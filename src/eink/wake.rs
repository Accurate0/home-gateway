use std::time::Duration;

use tracing::Instrument;

use super::EinkDisplayManager;
use super::manager::resolve::ResolvedDisplay;
use crate::actors::eink_display::{EInkDisplayActor, EInkDisplayMessage};
use crate::actors::system::rpc::{self, RpcError};
use crate::battery::BatteryChemistry;
use crate::device_registry::DeviceRegistry;
use crate::device_registry::last_seen::LastSeen;

#[derive(Debug, Clone, Copy)]
pub struct WakeReport<'a> {
    pub device_id: &'a str,
    pub battery_voltage: Option<f32>,
    pub is_charging: Option<bool>,
    pub battery_chemistry: Option<BatteryChemistry>,
    pub battery_kind: Option<&'a str>,
    pub firmware_version: Option<&'a str>,
    pub current_image_hash: Option<&'a str>,
}

pub struct WakeContext<'a> {
    pub devices: &'a DeviceRegistry,
    pub last_seen: &'a LastSeen,
    pub eink: &'a EinkDisplayManager,
    pub prepare_render_timeout: Duration,
}

#[tracing::instrument(name = "eink.wake", skip_all, fields(device_id = %report.device_id))]
pub async fn begin(
    context: &WakeContext<'_>,
    report: &WakeReport<'_>,
) -> Result<Option<ResolvedDisplay>, RpcError> {
    let device_id = report.device_id;
    let display = context.devices.eink_display(device_id);
    let registered = display.is_some();
    let configured_mode = display.map(|display| display.mode.name());
    let wake_drift_secs = wake_drift_secs(context.eink, device_id).await;

    tracing::info!(
        device_id = %device_id,
        registered,
        ?configured_mode,
        ?wake_drift_secs,
        current_image_hash = ?report.current_image_hash,
        battery_voltage = ?report.battery_voltage,
        is_charging = ?report.is_charging,
        battery_chemistry = ?report.battery_chemistry,
        battery_kind = ?report.battery_kind,
        firmware_version = ?report.firmware_version,
        "epd config requested"
    );

    if registered {
        context.last_seen.record(device_id).await;
    } else {
        tracing::warn!(
            device_id = %device_id,
            "epd config request from unregistered display, add it to config/devices/eink_display.yaml"
        );
    }

    report_to_actor(report)?;

    let prepare = rpc::query(
        EInkDisplayActor::NAME,
        context.prepare_render_timeout,
        |reply| EInkDisplayMessage::PrepareRender {
            device_id: device_id.to_owned(),
            reply,
        },
    )
    .instrument(tracing::info_span!(
        "eink.prepare_render",
        device_id = %device_id
    ));

    let (prepared, _) = tokio::join!(prepare, warm_displayed_frame(context.eink, report));

    if let Err(e) = prepared {
        tracing::warn!(
            device_id = %device_id,
            "could not prepare a fresh render ({e}), serving the last one"
        );
    }

    Ok(context.eink.resolve(device_id).await)
}

pub fn next_wake_at(
    now: chrono::DateTime<chrono::Utc>,
    wake_in_secs: u32,
) -> chrono::DateTime<chrono::Utc> {
    now + chrono::TimeDelta::seconds(wake_in_secs.into())
}

pub fn schedule_next_render(
    device_id: &str,
    next_wake_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), RpcError> {
    rpc::cast(
        EInkDisplayActor::NAME,
        EInkDisplayMessage::ScheduleNextRender {
            device_id: device_id.to_owned(),
            next_wake_at,
        },
    )
}

async fn warm_displayed_frame(eink: &EinkDisplayManager, report: &WakeReport<'_>) {
    let Some(hash) = report.current_image_hash else {
        return;
    };

    let Some(display) = eink.resolve(report.device_id).await else {
        return;
    };

    if !display.wants_partial() {
        return;
    }

    eink.packed_frame(hash).await;
}

async fn wake_drift_secs(eink: &EinkDisplayManager, device_id: &str) -> Option<i64> {
    let next_wake_at = eink
        .stored_next_wake(device_id)
        .await
        .inspect_err(|_| tracing::warn!(device_id, "could not read the stored wake"))
        .ok()
        .flatten()?;

    let drift = (chrono::Utc::now() - next_wake_at).num_seconds();

    crate::metrics::record_eink_wake_drift(device_id.to_owned(), drift as f64);

    Some(drift)
}

fn report_to_actor(report: &WakeReport<'_>) -> Result<(), RpcError> {
    rpc::cast(
        EInkDisplayActor::NAME,
        EInkDisplayMessage::ConfigRequest {
            device_id: report.device_id.to_owned(),
            trace_id: crate::tracing_context::current_trace_id(),
        },
    )?;

    let Some(voltage) = report.battery_voltage else {
        tracing::warn!(
            device_id = %report.device_id,
            "epd config request without a battery voltage, skipping battery report"
        );
        return Ok(());
    };

    rpc::cast(
        EInkDisplayActor::NAME,
        EInkDisplayMessage::BatteryReport {
            device_id: report.device_id.to_owned(),
            battery_voltage: voltage as f64,
            is_charging: report.is_charging,
            battery_chemistry: report.battery_chemistry,
            battery_kind: report.battery_kind.map(str::to_owned),
        },
    )
}
