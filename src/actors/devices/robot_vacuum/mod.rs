use crate::actors::system::battery::BatteryActor;
use crate::settings::RoborockField;
use crate::state::SharedActorState;
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use serde::Deserialize;
use uuid::Uuid;

pub mod spawn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaf {
    State,
    Attributes,
}

pub struct ValetudoEvent {
    pub event_id: Uuid,
    pub device_id: String,
    pub leaf: Leaf,
    pub payload: bytes::Bytes,
}

pub struct RoborockUpdate {
    pub event_id: Uuid,
    pub device_id: String,
    pub field: RoborockField,
    pub value: String,
}

pub enum Message {
    Valetudo(ValetudoEvent),
    Roborock(RoborockUpdate),
}

#[derive(Debug, Deserialize)]
struct ValetudoState {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    battery_level: Option<i32>,
    #[serde(default)]
    fan_speed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValetudoAttributes {
    #[serde(default, rename = "currentCleanArea", deserialize_with = "de_opt_f64")]
    current_clean_area: Option<f64>,
    #[serde(default, rename = "cleanCount")]
    clean_count: Option<i32>,
}

fn de_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrString {
        Num(f64),
        Str(String),
    }

    Ok(match Option::<NumOrString>::deserialize(deserializer)? {
        None => None,
        Some(NumOrString::Num(n)) => Some(n),
        Some(NumOrString::Str(s)) => s.parse::<f64>().ok(),
    })
}

pub struct RobotVacuumHandler {
    shared_actor_state: SharedActorState,
}

impl RobotVacuumHandler {
    pub const NAME: &str = "robot_vacuum";

    fn device_name(&self, device_id: &str) -> String {
        let devices = &self.shared_actor_state.devices;
        let address = devices.address_or_self(device_id);

        devices
            .valetudo(address)
            .map(|v| v.name.clone())
            .or_else(|| devices.roborock(address).map(|r| r.name.clone()))
            .unwrap_or_else(|| device_id.to_owned())
    }

    async fn handle_valetudo_state(
        &self,
        event_id: Uuid,
        device_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoState>(payload)?;

        sqlx::query!(
            "INSERT INTO robot_vacuum_events (event_id, device_id, source, state, battery_level) \
             VALUES ($1, $2, 'valetudo', $3, $4)",
            event_id,
            device_id,
            parsed.state,
            parsed.battery_level,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, state, battery_level, fan_speed) \
             VALUES ($1, 'valetudo', $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET \
               source = EXCLUDED.source, \
               state = COALESCE(EXCLUDED.state, latest_robot_vacuum_state.state), \
               battery_level = COALESCE(EXCLUDED.battery_level, latest_robot_vacuum_state.battery_level), \
               fan_speed = COALESCE(EXCLUDED.fan_speed, latest_robot_vacuum_state.fan_speed), \
               updated_at = now()",
            device_id,
            parsed.state,
            parsed.battery_level,
            parsed.fan_speed,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        if let Some(level) = parsed.battery_level {
            self.report_battery(device_id, level);
        }

        Ok(())
    }

    async fn handle_valetudo_attributes(
        &self,
        device_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoAttributes>(payload)?;
        let attributes = serde_json::from_slice::<serde_json::Value>(payload)?;

        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, current_clean_area, clean_count, attributes) \
             VALUES ($1, 'valetudo', $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET \
               current_clean_area = COALESCE(EXCLUDED.current_clean_area, latest_robot_vacuum_state.current_clean_area), \
               clean_count = COALESCE(EXCLUDED.clean_count, latest_robot_vacuum_state.clean_count), \
               attributes = EXCLUDED.attributes, \
               updated_at = now()",
            device_id,
            parsed.current_clean_area,
            parsed.clean_count,
            attributes,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn handle_roborock(&self, update: RoborockUpdate) -> Result<(), anyhow::Error> {
        let RoborockUpdate {
            event_id,
            device_id,
            field,
            value,
        } = update;

        match field {
            RoborockField::Status => {
                sqlx::query!(
                    "INSERT INTO robot_vacuum_events (event_id, device_id, source, state) \
                     VALUES ($1, $2, 'roborock', $3)",
                    event_id,
                    device_id,
                    value,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                sqlx::query!(
                    "INSERT INTO latest_robot_vacuum_state (device_id, source, state) \
                     VALUES ($1, 'roborock', $2) \
                     ON CONFLICT (device_id) DO UPDATE SET \
                       source = EXCLUDED.source, state = EXCLUDED.state, updated_at = now()",
                    device_id,
                    value,
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }

            RoborockField::Battery => {
                let level = value.parse::<f64>().ok().map(|v| v as i32);

                sqlx::query!(
                    "INSERT INTO robot_vacuum_events (event_id, device_id, source, battery_level) \
                     VALUES ($1, $2, 'roborock', $3)",
                    event_id,
                    device_id,
                    level,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                sqlx::query!(
                    "INSERT INTO latest_robot_vacuum_state (device_id, source, battery_level) \
                     VALUES ($1, 'roborock', $2) \
                     ON CONFLICT (device_id) DO UPDATE SET \
                       source = EXCLUDED.source, battery_level = EXCLUDED.battery_level, updated_at = now()",
                    device_id,
                    level,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                if let Some(level) = level {
                    self.report_battery(&device_id, level);
                }
            }

            RoborockField::Room => {
                sqlx::query!(
                    "INSERT INTO latest_robot_vacuum_state (device_id, source, room) \
                     VALUES ($1, 'roborock', $2) \
                     ON CONFLICT (device_id) DO UPDATE SET \
                       source = EXCLUDED.source, room = EXCLUDED.room, updated_at = now()",
                    device_id,
                    value,
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }
        }

        Ok(())
    }

    fn report_battery(&self, device_id: &str, level: i32) {
        let name = self.device_name(device_id);

        BatteryActor::report(
            device_id.to_owned(),
            name,
            Self::NAME.to_owned(),
            None,
            Some(level as f64),
            None,
        );
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::Valetudo(event) => match event.leaf {
                Leaf::State => {
                    self.handle_valetudo_state(event.event_id, &event.device_id, &event.payload)
                        .await?
                }
                Leaf::Attributes => {
                    self.handle_valetudo_attributes(&event.device_id, &event.payload)
                        .await?
                }
            },
            Message::Roborock(update) => self.handle_roborock(update).await?,
        }

        Ok(())
    }
}

impl Worker for RobotVacuumHandler {
    type Key = ();
    type Message = Message;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct RobotVacuumHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}

impl WorkerBuilder<RobotVacuumHandler, ()> for RobotVacuumHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (RobotVacuumHandler, ()) {
        (
            RobotVacuumHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
