use uuid::Uuid;

use crate::device_registry::{DeviceRegistry, Transport};
use crate::event_bus::{EventBus, EventBusMessage};
use crate::repo::DeviceRepo;

pub async fn record(
    devices: &DeviceRegistry,
    repo: &DeviceRepo,
    event_bus: &EventBus,
    transport: Transport,
    address: &str,
    connected: bool,
) {
    let Some(device_id) = devices.id_for_address(address) else {
        tracing::warn!("connection change for unregistered {transport} device {address}");
        return;
    };

    match connected {
        true => tracing::info!("{transport} device {device_id} ({address}) is connected"),
        false => tracing::warn!("{transport} device {device_id} ({address}) is disconnected"),
    }

    if let Err(e) = repo.record_connection(device_id, connected).await {
        tracing::error!("failed to record connection state for {device_id}: {e}");
    }

    event_bus.publish(EventBusMessage::DeviceConnection {
        event_id: Uuid::new_v4(),
        device_id: device_id.to_owned(),
        transport: transport.to_string(),
        room: devices.room(address).map(str::to_owned),
        connected,
    });
}
