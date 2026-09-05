use crate::repo::battery::BatteryReading;
use crate::{battery::BatteryChemistry, event_bus::EventBusMessage, state::AppState};
use ractor::Actor;
use uuid::Uuid;

pub enum BatteryMessage {
    Report {
        device_id: String,
        name: String,
        kind: String,
        battery_voltage: Option<f64>,
        battery_percent: Option<f64>,
        battery_chemistry: Option<BatteryChemistry>,
    },
}

pub struct BatteryActor {
    pub shared_actor_state: AppState,
}

impl BatteryActor {
    pub const NAME: &str = "battery";

    pub fn report(
        device_id: String,
        name: String,
        kind: String,
        battery_voltage: Option<f64>,
        battery_percent: Option<f64>,
        battery_chemistry: Option<BatteryChemistry>,
    ) {
        let message = BatteryMessage::Report {
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry,
        };

        if let Err(e) = crate::actors::system::rpc::cast(Self::NAME, message) {
            tracing::error!("failed to send battery report: {e}");
        }
    }
}

impl Actor for BatteryActor {
    type Msg = BatteryMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let BatteryMessage::Report {
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry,
        } = message;
        let devices = &self.shared_actor_state.devices;
        let address = devices.address_or_self(&device_id);

        if devices.battery(address).is_none() {
            tracing::warn!("battery report for '{device_id}' without a battery kind, dropping");
            return Ok(());
        }

        let event_id = Uuid::new_v4();

        self.shared_actor_state
            .repos
            .battery()
            .record(BatteryReading {
                event_id,
                device_id: &device_id,
                name: &name,
                kind: &kind,
                battery_voltage,
                battery_percent,
                battery_chemistry,
            })
            .await?;

        if let Some(percent) = battery_percent {
            crate::metrics::record_device_battery_percent(device_id.clone(), kind.clone(), percent);
        }
        if let Some(voltage) = battery_voltage {
            crate::metrics::record_device_battery_voltage(device_id.clone(), kind.clone(), voltage);
        }

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::DeviceBattery {
                event_id,
                device_id,
                kind,
                name,
                battery_voltage,
                battery_percent,
            });

        Ok(())
    }
}
