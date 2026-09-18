use uuid::Uuid;

use crate::actors::devices::{
    control_switch, door_sensor, environment_sensor, light, media_player, presence_sensor,
    robot_vacuum, smart_switch,
};
use crate::device_metric::DeviceMetric;
use crate::state::AppState;

use super::decoded_device::DecodedDevice;
use super::reading::DeviceReading;
use super::role;

pub async fn dispatch(
    state: &AppState,
    event_id: Uuid,
    device: &DecodedDevice,
    friendly_name: &str,
    reading: DeviceReading,
) {
    let devices = &state.devices;
    let address = device.address.as_str();

    if let Some(percent) = reading.battery {
        record_battery(state, address, percent as f64);
    }

    role::run::<door_sensor::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<environment_sensor::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<light::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<smart_switch::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<presence_sensor::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<control_switch::Entity>(event_id, devices, device, friendly_name, &reading);
    role::run::<robot_vacuum::RoborockReading>(event_id, devices, device, friendly_name, &reading);
    role::run::<media_player::MediaPlayerReading>(
        event_id,
        devices,
        device,
        friendly_name,
        &reading,
    );

    let device_id = devices.id_for_address(address).map(str::to_owned);

    for (metric, value) in reading.metrics {
        let record = DeviceMetric {
            event_id,
            address: address.to_owned(),
            device_id: device_id.clone(),
            metric,
            value: value.into(),
        };

        if let Err(e) = state.repos.metric().record(&record).await {
            tracing::error!("failed to save device metric for {address}: {e}");
            crate::tracing_context::record_current_error(&e.to_string());
        }
    }
}

fn record_battery(state: &AppState, address: &str, percent: f64) {
    let devices = &state.devices;

    let Some(settings) = devices.battery(address) else {
        tracing::debug!("{address} reported a battery level but declares no battery role");
        return;
    };

    let device_id = devices
        .id_for_address(address)
        .unwrap_or(address)
        .to_owned();

    crate::actors::system::battery::BatteryActor::report(
        device_id,
        settings.name.clone(),
        "battery".to_owned(),
        None,
        Some(percent),
        None,
    );
}
