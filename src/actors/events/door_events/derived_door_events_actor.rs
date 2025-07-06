use super::{DoorEvents, DoorEventsType};
use crate::types::db::DoorState;
use crate::{
    settings::{DoorSettings, IEEEAddress},
    types::SharedActorState,
};
use chrono::{DateTime, Utc};
use ractor::Actor;
use std::collections::HashMap;

pub struct DerivedDoorEventsState {
    pub map: HashMap<IEEEAddress, DoorState>,
    pub last_trigger: HashMap<IEEEAddress, DateTime<Utc>>,
}

pub struct DerivedDoorEvents {
    pub shared_actor_state: SharedActorState,
    pub door_settings: HashMap<IEEEAddress, DoorSettings>,
}

impl DerivedDoorEvents {
    pub const NAME: &str = "derived-door-events";

    async fn change_door_state(
        &self,
        message: &DoorEvents,
        state: &mut DerivedDoorEventsState,
        now: DateTime<Utc>,
        door_settings: &DoorSettings,
        door_state: DoorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        sqlx::query!(
            "INSERT INTO derived_door_events (event_id, name, id, ieee_addr, state) VALUES ($1, $2, $3, $4, $5)",
            message.event_id,
            door_settings.name,
            door_settings.id,
            &message.ieee_addr,
            door_state as DoorState
        ).execute(&self.shared_actor_state.db).await?;

        state.map.insert(message.ieee_addr.clone(), door_state);

        state
            .last_trigger
            .insert(message.ieee_addr.clone(), now.clone());

        Ok(())
    }
}

impl Actor for DerivedDoorEvents {
    type Msg = DoorEvents;
    type State = DerivedDoorEventsState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let last_state = sqlx::query!(
            r#"
        SELECT derived_door_events.id, derived_door_events.name, derived_door_events.ieee_addr, state as "state: DoorState"
        FROM
            (SELECT id, max(time) FROM derived_door_events GROUP BY id) AS latest_state
            INNER JOIN derived_door_events ON derived_door_events.id = latest_state.id
        "#
        )
        .fetch_all(&self.shared_actor_state.db)
        .await?;

        let mut map = HashMap::new();

        for door in last_state {
            map.insert(door.ieee_addr.clone(), door.state);
        }

        Ok(DerivedDoorEventsState {
            map,
            last_trigger: Default::default(),
        })
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        if let Some(door_settings) = self.door_settings.get(&message.ieee_addr) {
            let last_state = state.map.get(&message.ieee_addr);
            let now = chrono::offset::Utc::now();
            let last_event_is_too_soon = state
                .last_trigger
                .get(&message.ieee_addr)
                .map(|d| {
                    let difference = now - d;
                    difference.as_seconds_f64() < 1.0
                })
                .unwrap_or(false);

            if last_event_is_too_soon {
                return Ok(());
            }

            match message.event {
                DoorEventsType::Opened => match last_state {
                    Some(last_state) => match last_state {
                        DoorState::Open => {
                            // do nothing
                        }
                        DoorState::Closed => {
                            self.change_door_state(
                                &message,
                                state,
                                now,
                                door_settings,
                                DoorState::Open,
                            )
                            .await?;
                        }
                    },
                    None => {
                        self.change_door_state(
                            &message,
                            state,
                            now,
                            door_settings,
                            DoorState::Open,
                        )
                        .await?;
                    }
                },
                DoorEventsType::Closed => match last_state {
                    Some(door_state) => match door_state {
                        DoorState::Open => {
                            self.change_door_state(
                                &message,
                                state,
                                now,
                                door_settings,
                                DoorState::Closed,
                            )
                            .await?;
                        }
                        DoorState::Closed => {
                            // do nothing
                        }
                    },
                    None => {
                        self.change_door_state(
                            &message,
                            state,
                            now,
                            door_settings,
                            DoorState::Closed,
                        )
                        .await?
                    }
                },
                DoorEventsType::Trigger { .. } => {}
            }
        };

        Ok(())
    }
}
