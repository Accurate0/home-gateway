use serde::Deserialize;
use serde::de::IntoDeserializer;
use tonic::{Code, Request, Response, Status};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::proto::eink_display_server::EinkDisplay;
use super::proto::wake_response::Refresh;
use super::proto::{Clear, FirmwareUpdate, FullFrame, PartialFrame, WakeRequest, WakeResponse};
use crate::auth::AuthContext;
use crate::auth::scope::{Action, Resource};
use crate::battery::BatteryChemistry;
use crate::device_registry::last_seen::LastSeen;
use crate::eink::EinkDisplayManager;
use crate::eink::manager::config::firmware_url;
use crate::eink::manager::decision::WakeDecision;
use crate::eink::panel::{PACKED_FRAME_SIZE, crop_packed};
use crate::eink::rtc::rtc_sync;
use crate::eink::wake::{self, WakeContext, WakeReport};
use crate::repo::eink::{RtcReportRecord, RtcSyncRecord, WakeRecord};
use crate::routes::epd::DeviceReport;
use crate::state::AppState;

const GRPC_STATUS_ATTRIBUTE: &str = "rpc.grpc.status_code";

pub struct EinkDisplayService {
    state: AppState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum Outcome {
    Unchanged,
    Clear,
    Full,
    Partial,
}

struct PlannedRefresh {
    outcome: Outcome,
    refresh: Option<Refresh>,
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

        let planned = planned_refresh(&decision, displayed.as_deref())?;

        let now_displayed = match planned.outcome {
            Outcome::Unchanged => None,
            Outcome::Clear => Some(None),
            Outcome::Full | Outcome::Partial => {
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
            outcome = %planned.outcome,
            image_bytes = image_bytes(planned.refresh.as_ref()),
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
            refresh: planned.refresh,
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
            crate::tracing_context::record_error(&span, status.message());
            tracing::warn!(code = ?status.code(), "wake failed: {}", status.message());
        }

        result.map(Response::new)
    }
}

fn planned_refresh(
    decision: &WakeDecision,
    displayed: Option<&str>,
) -> Result<PlannedRefresh, Status> {
    if decision.clear_screen {
        return Ok(PlannedRefresh {
            outcome: Outcome::Clear,
            refresh: Some(Refresh::Clear(Clear {})),
        });
    }

    let Some(frame) = &decision.frame else {
        return Ok(PlannedRefresh {
            outcome: Outcome::Unchanged,
            refresh: None,
        });
    };

    if displayed == Some(frame.hash.as_str()) {
        return Ok(PlannedRefresh {
            outcome: Outcome::Unchanged,
            refresh: None,
        });
    }

    if frame.packed.len() != PACKED_FRAME_SIZE {
        tracing::error!(
            hash = %frame.hash,
            len = frame.packed.len(),
            "planned frame is not {PACKED_FRAME_SIZE} bytes"
        );
        return Err(Status::internal("planned frame has the wrong size"));
    }

    Ok(match frame.partial {
        Some(window) => PlannedRefresh {
            outcome: Outcome::Partial,
            refresh: Some(Refresh::Partial(PartialFrame {
                x: window.x,
                y: window.y,
                width: window.width,
                height: window.height,
                image: bytes::Bytes::from(crop_packed(&frame.packed, window)),
            })),
        },
        None => PlannedRefresh {
            outcome: Outcome::Full,
            refresh: Some(Refresh::Full(FullFrame {
                image: frame.packed.clone(),
            })),
        },
    })
}

fn image_bytes(refresh: Option<&Refresh>) -> usize {
    match refresh {
        Some(Refresh::Full(frame)) => frame.image.len(),
        Some(Refresh::Partial(frame)) => frame.image.len(),
        Some(Refresh::Clear(_)) | None => 0,
    }
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

fn battery_chemistry(value: &str) -> Option<BatteryChemistry> {
    let value = non_empty(value)?;

    let parsed: Result<BatteryChemistry, serde::de::value::Error> =
        BatteryChemistry::deserialize(value.into_deserializer());

    match parsed {
        Ok(chemistry) => Some(chemistry),
        Err(_) => {
            tracing::warn!("unknown battery chemistry `{value}`, ignoring");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::decision::PlannedFrame;
    use crate::eink::panel::PartialWindow;
    use pretty_assertions::assert_eq;

    const HASH: &str = "aa00000000000000000000000000000000000000000000000000000000000001";
    const OTHER_HASH: &str = "bb00000000000000000000000000000000000000000000000000000000000002";

    fn decision(partial: Option<PartialWindow>) -> WakeDecision {
        WakeDecision {
            refresh_secs: 900,
            firmware_version: None,
            clear_screen: false,
            frame: Some(PlannedFrame {
                hash: HASH.to_owned(),
                packed: bytes::Bytes::from(vec![0x11; PACKED_FRAME_SIZE]),
                partial,
            }),
        }
    }

    #[test]
    fn a_display_already_showing_the_frame_gets_no_refresh() {
        let planned = planned_refresh(&decision(None), Some(HASH)).unwrap();

        assert_eq!(planned.outcome, Outcome::Unchanged);
        assert!(planned.refresh.is_none());
    }

    #[test]
    fn a_display_with_no_known_frame_gets_the_full_frame() {
        let planned = planned_refresh(&decision(None), None).unwrap();

        assert_eq!(planned.outcome, Outcome::Full);
        assert_eq!(image_bytes(planned.refresh.as_ref()), PACKED_FRAME_SIZE);
    }

    #[test]
    fn a_partial_window_sends_only_the_cropped_region() {
        let window = PartialWindow {
            x: 0,
            y: 0,
            width: 64,
            height: 8,
        };

        let planned = planned_refresh(&decision(Some(window)), Some(OTHER_HASH)).unwrap();

        assert_eq!(planned.outcome, Outcome::Partial);
        assert_eq!(image_bytes(planned.refresh.as_ref()), 64 / 2 * 8);
    }

    #[test]
    fn clear_screen_wins_over_the_frame() {
        let clearing = WakeDecision {
            clear_screen: true,
            ..decision(None)
        };

        let planned = planned_refresh(&clearing, Some(HASH)).unwrap();

        assert_eq!(planned.outcome, Outcome::Clear);
        assert!(matches!(planned.refresh, Some(Refresh::Clear(_))));
    }

    #[test]
    fn no_planned_frame_leaves_the_panel_alone() {
        let empty = WakeDecision {
            frame: None,
            ..decision(None)
        };

        let planned = planned_refresh(&empty, None).unwrap();

        assert_eq!(planned.outcome, Outcome::Unchanged);
    }

    #[test]
    fn a_truncated_frame_is_an_error_rather_than_a_corrupt_draw() {
        let truncated = WakeDecision {
            frame: Some(PlannedFrame {
                hash: HASH.to_owned(),
                packed: bytes::Bytes::from(vec![0x11; 10]),
                partial: None,
            }),
            ..decision(None)
        };

        let status = planned_refresh(&truncated, None).err().unwrap();

        assert_eq!(status.code(), Code::Internal);
    }

    #[test]
    fn battery_chemistry_parses_the_firmware_names() {
        assert_eq!(battery_chemistry("lipo"), Some(BatteryChemistry::Lipo));
        assert_eq!(battery_chemistry("li_ion"), Some(BatteryChemistry::LiIon));
        assert_eq!(battery_chemistry(""), None);
        assert_eq!(battery_chemistry("plutonium"), None);
    }
}
