use crate::types::SharedActorState;
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

pub struct NewEvent {
    pub event_id: Uuid,
    pub identifier: String,
    pub leaf: Leaf,
    pub payload: bytes::Bytes,
}

pub enum Message {
    NewEvent(NewEvent),
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

pub struct ValetudoHandler {
    shared_actor_state: SharedActorState,
}

impl ValetudoHandler {
    pub const NAME: &str = "valetudo";

    async fn handle_state(
        &self,
        event_id: Uuid,
        identifier: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoState>(payload)?;

        sqlx::query!(
            "INSERT INTO valetudo_events (event_id, identifier, state, battery_level) \
             VALUES ($1, $2, $3, $4)",
            event_id,
            identifier,
            parsed.state,
            parsed.battery_level,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_valetudo_state (identifier, state, battery_level, fan_speed) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (identifier) DO UPDATE SET \
               state = COALESCE(EXCLUDED.state, latest_valetudo_state.state), \
               battery_level = COALESCE(EXCLUDED.battery_level, latest_valetudo_state.battery_level), \
               fan_speed = COALESCE(EXCLUDED.fan_speed, latest_valetudo_state.fan_speed), \
               updated_at = now()",
            identifier,
            parsed.state,
            parsed.battery_level,
            parsed.fan_speed,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn handle_attributes(
        &self,
        identifier: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoAttributes>(payload)?;
        let attributes = serde_json::from_slice::<serde_json::Value>(payload)?;

        sqlx::query!(
            "INSERT INTO latest_valetudo_state (identifier, current_clean_area, clean_count, attributes) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (identifier) DO UPDATE SET \
               current_clean_area = COALESCE(EXCLUDED.current_clean_area, latest_valetudo_state.current_clean_area), \
               clean_count = COALESCE(EXCLUDED.clean_count, latest_valetudo_state.clean_count), \
               attributes = EXCLUDED.attributes, \
               updated_at = now()",
            identifier,
            parsed.current_clean_area,
            parsed.clean_count,
            attributes,
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        Ok(())
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.leaf {
                Leaf::State => {
                    self.handle_state(event.event_id, &event.identifier, &event.payload)
                        .await?
                }
                Leaf::Attributes => {
                    self.handle_attributes(&event.identifier, &event.payload)
                        .await?
                }
            },
        }

        Ok(())
    }
}

impl Worker for ValetudoHandler {
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

pub struct ValetudoHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<ValetudoHandler, ()> for ValetudoHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (ValetudoHandler, ()) {
        (
            ValetudoHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
