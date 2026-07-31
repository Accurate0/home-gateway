use crate::{battery::BatteryChemistry, event_bus::EventBusMessage, types::SharedActorState};
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
    pub shared_actor_state: SharedActorState,
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
        let Some(actor) = ractor::registry::where_is(Self::NAME.to_owned()) else {
            tracing::error!("battery actor not found, dropping report for {device_id}");
            return;
        };

        if let Err(e) = actor.send_message(BatteryMessage::Report {
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry,
        }) {
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

        sqlx::query!(
            "INSERT INTO device_battery (event_id, device_id, kind, battery_voltage, battery_percent) VALUES ($1, $2, $3, $4, $5)",
            event_id,
            device_id,
            kind,
            battery_voltage,
            battery_percent,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO device_battery_latest (device_id, name, kind, battery_voltage, battery_percent, battery_chemistry, updated_at) VALUES ($1, $2, $3, $4, $5, $6, now()) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, kind = EXCLUDED.kind, battery_voltage = EXCLUDED.battery_voltage, battery_percent = EXCLUDED.battery_percent, battery_chemistry = COALESCE(EXCLUDED.battery_chemistry, device_battery_latest.battery_chemistry), updated_at = EXCLUDED.updated_at",
            device_id,
            name,
            kind,
            battery_voltage,
            battery_percent,
            battery_chemistry as Option<BatteryChemistry>,
        )
        .execute(&self.shared_actor_state.db)
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
