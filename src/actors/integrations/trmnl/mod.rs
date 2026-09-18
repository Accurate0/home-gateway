use crate::{
    integrations::trmnl::{Trmnl, types::TrmnlDevice},
    settings::TrmnlDeviceSettings,
    state::AppState,
};
use ractor::Actor;
use std::time::Duration;

pub enum TrmnlMessage {
    CheckBattery,
}

pub struct TrmnlActor {
    pub shared_actor_state: AppState,
    pub trmnl: Trmnl,
}

impl TrmnlActor {
    pub const NAME: &str = "trmnl";

    async fn check_battery(&self) -> Result<(), ractor::ActorProcessingErr> {
        let registry = self.shared_actor_state.devices.trmnl_devices();
        let devices = self.trmnl.list_devices().await?;

        for device in devices {
            let Some((address, settings)) = match_device(&device, registry) else {
                continue;
            };

            crate::device_registry::last_seen::record(
                &self.shared_actor_state.devices,
                self.shared_actor_state.repos.device(),
                address,
            )
            .await;

            let Some(voltage) = device.battery_voltage else {
                tracing::debug!(
                    "trmnl device '{}' reported no battery voltage, skipping",
                    settings.id
                );
                continue;
            };

            let device_id = settings.id.clone();
            let name = settings.name.clone();
            let kind = "trmnl";

            self.shared_actor_state
                .repos
                .eink()
                .store_battery(&device_id, &name, voltage, None)
                .await?;

            crate::actors::system::battery::BatteryActor::report(
                device_id,
                name,
                kind.to_owned(),
                Some(voltage),
                None,
                None,
            );
        }

        Ok(())
    }
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

    #[tracing::instrument(
        parent = None,
        name = "actor.trmnl",
        skip(self, _myself, message, _state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            TrmnlMessage::CheckBattery => {
                let started = std::time::Instant::now();

                match self.check_battery().await {
                    Ok(()) => {
                        crate::metrics::record_integration_poll(
                            "trmnl",
                            "success",
                            started.elapsed(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("error checking trmnl batteries: {e}");
                        crate::tracing_context::record_current_error(&e.to_string());
                        crate::metrics::record_integration_poll(
                            "trmnl",
                            "error",
                            started.elapsed(),
                        );
                    }
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
