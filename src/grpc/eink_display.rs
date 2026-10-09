use tonic::{Code, Request, Response, Status};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::proto::eink_display_server::EinkDisplay;
use super::proto::wake_response::Refresh;
use super::proto::{Clear, FirmwareUpdate, FullFrame, PartialFrame, WakeRequest, WakeResponse};
use crate::auth::AuthContext;
use crate::auth::scope::{Action, Resource};
use crate::device_registry::last_seen::LastSeen;
use crate::eink::EinkDisplayManager;
use crate::eink::manager::config::firmware_url;
use crate::eink::refresh::{PlannedRefresh, planned_refresh};
use crate::eink::rtc::rtc_sync;
use crate::eink::wake::{self, WakeContext, WakeReport, battery_chemistry};
use crate::repo::eink::{RtcReportRecord, RtcSyncRecord, WakeRecord};
use crate::routes::epd::DeviceReport;
use crate::state::AppState;

const GRPC_STATUS_ATTRIBUTE: &str = "rpc.grpc.status_code";

pub struct EinkDisplayService {
    state: AppState,
}

impl EinkDisplayService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn handle(&self, request: Request<WakeRequest>) -> Result<WakeResponse, Status> {
        let Some(auth) = request.extensions().get::<AuthContext>() else {
            tracing::warn!("wake reached the service without an auth context");
            return Err(Status::unauthenticated("missing credentials"));
        };

        if let Err(missing) = auth.require(Resource::Epd, Action::Read) {
            tracing::warn!("wake denied, missing scope {}", missing.scope);
            return Err(Status::permission_denied("missing scope"));
        }

        let request = request.into_inner();
        let device_id = request.device_id.as_str();
        let eink = self.state.handles.expect::<EinkDisplayManager>();

        let displayed = if request.previous_refresh_failed {
            tracing::warn!(
                device_id = %device_id,
                "display reported a failed refresh, forgetting what it shows"
            );

            None
        } else {
            eink.displayed_hash(device_id).await
        };

        let context = WakeContext {
            devices: &self.state.devices,
            last_seen: self.state.handles.expect::<LastSeen>(),
            eink,
            prepare_render_timeout: self.state.settings.eink_display.prepare_render_timeout(),
        };

        let firmware_version = non_empty(&request.firmware_version);

        let report = WakeReport {
            device_id,
            battery_voltage: request.battery_voltage,
            is_charging: Some(request.is_charging),
            battery_chemistry: battery_chemistry(&request.battery_chemistry),
            battery_kind: non_empty(&request.battery_kind),
            firmware_version,
            current_image_hash: displayed.as_deref(),
        };

        let resolved = wake::begin(&context, &report)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let Some(resolved) = resolved else {
            return Err(Status::not_found("display is not registered"));
        };

        let decision = eink
            .wake_decision(
                &resolved,
                DeviceReport {
                    running_firmware_version: firmware_version,
                    current_image_hash: displayed.as_deref(),
                },
            )
            .await;

        let planned = planned_refresh(&decision, displayed.as_deref())
            .map_err(|e| Status::internal(e.to_string()))?;

        let now_displayed = match planned {
            PlannedRefresh::Unchanged => None,
            PlannedRefresh::Clear => Some(None),
            PlannedRefresh::Full { .. } | PlannedRefresh::Partial { .. } => {
                Some(decision.frame.as_ref().map(|frame| frame.hash.as_str()))
            }
        };

        let reported = request
            .rtc_unix_ms
            .and_then(chrono::DateTime::from_timestamp_millis);

        let synced_at = eink.rtc_synced_at(device_id).await;
        let now = chrono::Utc::now();
        let interval = self.state.settings.eink_display.rtc_sync_interval;
        let next_wake_at = wake::next_wake_at(now, decision.refresh_secs);
        let sync = rtc_sync(reported, synced_at, now, interval);

        let drift_ms = sync
            .and_then(|sync| sync.drift)
            .map(|drift| drift.num_milliseconds());

        let record = WakeRecord {
            next_wake_at,
            displayed_hash: now_displayed,
            rtc_report: reported.map(|reported_at| RtcReportRecord {
                reported_at,
                offset_ms: (reported_at - now).num_milliseconds(),
            }),
            rtc_sync: sync.map(|_| RtcSyncRecord {
                synced_at: now,
                drift_ms,
            }),
        };

        eink.store_wake(device_id, &resolved.name, &record)
            .await
            .map_err(|e| Status::internal(e.message().to_string()))?;

        wake::schedule_next_render(device_id, next_wake_at)
            .map_err(|e| Status::internal(e.to_string()))?;

        let rtc_synced = sync.is_some();

        if rtc_synced {
            if let Some(drift_ms) = drift_ms {
                crate::metrics::record_eink_rtc_drift(
                    device_id.to_owned(),
                    drift_ms as f64 / 1000.0,
                );
            }

            tracing::info!(
                device_id = %device_id,
                ?reported,
                ?synced_at,
                ?drift_ms,
                "setting the display clock"
            );
        }

        tracing::info!(
            device_id = %device_id,
            outcome = %planned,
            image_bytes = planned.image_bytes(),
            sleep_secs = decision.refresh_secs,
            firmware_update = ?decision.firmware_version,
            rtc_unix_ms = ?request.rtc_unix_ms,
            rtc_synced,
            "wake answered"
        );

        Ok(WakeResponse {
            sleep_secs: decision.refresh_secs,
            firmware: decision.firmware_version.map(|version| FirmwareUpdate {
                version,
                url: firmware_url(device_id),
            }),
            refresh: refresh_message(planned),
            set_rtc_unix_ms: rtc_synced.then(|| chrono::Utc::now().timestamp_millis()),
        })
    }
}

#[tonic::async_trait]
impl EinkDisplay for EinkDisplayService {
    async fn wake(&self, request: Request<WakeRequest>) -> Result<Response<WakeResponse>, Status> {
        let span = tracing::Span::current();

        let result = self.handle(request).await;

        let code = match &result {
            Ok(_) => Code::Ok,
            Err(status) => status.code(),
        };

        span.set_attribute(GRPC_STATUS_ATTRIBUTE, code as i64);

        if let Err(status) = &result {
            crate::telemetry::context::record_error(&span, status.message());
            tracing::warn!(code = ?status.code(), "wake failed: {}", status.message());
        }

        result.map(Response::new)
    }
}

fn refresh_message(planned: PlannedRefresh) -> Option<Refresh> {
    match planned {
        PlannedRefresh::Unchanged => None,
        PlannedRefresh::Clear => Some(Refresh::Clear(Clear {})),
        PlannedRefresh::Full { image } => Some(Refresh::Full(FullFrame { image })),
        PlannedRefresh::Partial { window, image } => Some(Refresh::Partial(PartialFrame {
            x: window.x,
            y: window.y,
            width: window.width,
            height: window.height,
            image,
        })),
    }
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}
