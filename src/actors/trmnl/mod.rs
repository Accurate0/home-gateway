use crate::{
    device_registry::TrmnlDeviceSettings, event_bus::EventBusMessage, trmnl::Trmnl,
    trmnl::types::TrmnlDevice, types::SharedActorState,
};
use ractor::Actor;
use std::time::Duration;
use uuid::Uuid;

pub enum TrmnlMessage {
    CheckBattery,
}

pub struct TrmnlActor {
    pub shared_actor_state: SharedActorState,
    pub trmnl: Trmnl,
}

impl TrmnlActor {
    pub const NAME: &str = "trmnl";
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn match_device<'a>(
    device: &TrmnlDevice,
    registry: &'a std::collections::HashMap<String, TrmnlDeviceSettings>,
) -> Option<(&'a String, &'a TrmnlDeviceSettings)> {
    let friendly = normalize(&device.friendly_id);
    let mac = normalize(&device.mac_address);
    registry.iter().find(|(address, _)| {
        let key = normalize(address);
        key == friendly || key == mac
    })
}

impl Actor for TrmnlActor {
    type Msg = TrmnlMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let refresh = self
            .shared_actor_state
            .settings
            .trmnl
            .refresh
            .to_std()
            .unwrap_or(Duration::from_secs(3 * 3600));
        myself.send_interval(refresh, || TrmnlMessage::CheckBattery);
        myself.send_message(TrmnlMessage::CheckBattery)?;

        Ok(())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            TrmnlMessage::CheckBattery => {
                let registry = self.shared_actor_state.devices.trmnl_devices();
                let devices = self.trmnl.list_devices().await?;

                for device in devices {
                    let Some((_address, settings)) = match_device(&device, registry) else {
                        continue;
                    };
                    let Some(voltage) = device.battery_voltage else {
                        tracing::debug!(
                            "trmnl device '{}' reported no battery voltage, skipping",
                            settings.id
                        );
                        continue;
                    };

                    let device_id = settings.id.clone();
                    let name = settings.name.clone();
                    let event_id = Uuid::new_v4();
                    let kind = "trmnl";

                    sqlx::query!(
                        "INSERT INTO device_battery (event_id, device_id, kind, battery_voltage) VALUES ($1, $2, $3, $4)",
                        event_id,
                        device_id,
                        kind,
                        voltage,
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;

                    sqlx::query!(
                        "INSERT INTO eink_display (device_id, name, battery_voltage, updated_at) VALUES ($1, $2, $3, now()) \
                         ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, updated_at = EXCLUDED.updated_at",
                        device_id,
                        name,
                        voltage,
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;

                    crate::metrics::record_device_battery_voltage(
                        device_id.clone(),
                        kind.to_owned(),
                        voltage,
                    );

                    self.shared_actor_state
                        .event_bus
                        .publish(EventBusMessage::DeviceBattery {
                            event_id,
                            device_id,
                            kind: kind.to_owned(),
                            name,
                            battery_voltage: voltage,
                        });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn device(friendly_id: &str, mac: &str, voltage: Option<f64>) -> TrmnlDevice {
        TrmnlDevice {
            id: 1,
            name: "device".to_owned(),
            friendly_id: friendly_id.to_owned(),
            mac_address: mac.to_owned(),
            battery_voltage: voltage,
            percent_charged: None,
        }
    }

    fn registry(address: &str) -> HashMap<String, TrmnlDeviceSettings> {
        HashMap::from([(
            address.to_owned(),
            TrmnlDeviceSettings {
                id: "fridge-trmnl".to_owned(),
                name: "Fridge TRMNL".to_owned(),
            },
        )])
    }

    #[test]
    fn matches_by_friendly_id_case_insensitive() {
        let reg = registry("653VZN");
        let matched = match_device(&device("653vzn", "12:34:56:78:9A:BC", Some(3.7)), &reg);
        assert_eq!(matched.map(|(_, s)| s.id.as_str()), Some("fridge-trmnl"));
    }

    #[test]
    fn matches_by_mac_ignoring_colons_and_case() {
        let reg = registry("94a990cf8384");
        let matched = match_device(&device("XXX", "94:A9:90:CF:83:84", Some(3.7)), &reg);
        assert_eq!(matched.map(|(_, s)| s.id.as_str()), Some("fridge-trmnl"));
    }

    #[test]
    fn unmatched_device_returns_none() {
        let reg = registry("653VZN");
        assert!(match_device(&device("OTHER", "00:00:00:00:00:00", Some(3.7)), &reg).is_none());
    }
}
