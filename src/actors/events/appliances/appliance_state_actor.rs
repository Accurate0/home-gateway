use crate::{
    notify::notify,
    settings::{ApplianceSettings, IEEEAddress},
    timed_average::TimedAverage,
    types::{SharedActorState, db::ApplianceStateType},
};
use ractor::Actor;
use std::{collections::HashMap, time::Duration};
use tracing::Level;

use super::ApplianceEvents;

pub struct ApplianceStateState {
    pub average_running: HashMap<IEEEAddress, TimedAverage>,
    pub map: HashMap<IEEEAddress, ApplianceStateType>,
}

pub struct ApplianceState {
    pub shared_actor_state: SharedActorState,
}

impl ApplianceState {
    pub const NAME: &str = "appliance-state";

    async fn turn_off_appliance(
        &self,
        state: &mut ApplianceStateState,
        ieee_addr: &String,
        event_id: uuid::Uuid,
        average_current: f64,
        appliance_settings: &ApplianceSettings,
    ) -> Result<(), ractor::ActorProcessingErr> {
        tracing::info!(
            "threshold reached for {ieee_addr} - {}, turning off, avg current {average_current}",
            appliance_settings.id
        );

        sqlx::query!(
                "INSERT INTO appliances (event_id, name, ieee_addr, id, state) VALUES ($1, $2, $3, $4, $5)",
                event_id,
                appliance_settings.name,
                ieee_addr,
                appliance_settings.id,
                ApplianceStateType::Off as ApplianceStateType
            ).execute(&self.shared_actor_state.db).await?;

        state.map.insert(ieee_addr.clone(), ApplianceStateType::Off);

        let message = format!("{} has turned off.", appliance_settings.name);
        notify(&appliance_settings.notify, message, true);

        Ok(())
    }

    async fn setup_initial_state(
        &self,
        state: &mut ApplianceStateState,
        ieee_addr: String,
        event_id: uuid::Uuid,
        average_current: f64,
        appliance_settings: &ApplianceSettings,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let on_or_off = if average_current >= appliance_settings.current.threshold {
            ApplianceStateType::On
        } else if average_current <= appliance_settings.current.threshold {
            ApplianceStateType::Off
        } else {
            // shouldn't happen
            ApplianceStateType::Off
        };
        tracing::info!(
            "initial state for {ieee_addr} - {}, turning {:?}",
            appliance_settings.id,
            on_or_off
        );
        sqlx::query!(
            "INSERT INTO appliances (event_id, name, ieee_addr, id, state) VALUES ($1, $2, $3, $4, $5)",
            event_id,
            appliance_settings.name,
            ieee_addr,
            appliance_settings.id,
            &on_or_off as &ApplianceStateType
        ).execute(&self.shared_actor_state.db).await?;
        state.map.insert(ieee_addr.clone(), on_or_off);
        Ok(())
    }

    async fn turn_on_appliance(
        &self,
        state: &mut ApplianceStateState,
        ieee_addr: &String,
        event_id: uuid::Uuid,
        average_current: f64,
        appliance_settings: &ApplianceSettings,
    ) -> Result<(), ractor::ActorProcessingErr> {
        tracing::info!(
            "threshold reached for {ieee_addr} - {}, turning on, avg current {average_current}",
            appliance_settings.id
        );
        sqlx::query!(
                "INSERT INTO appliances (event_id, name, ieee_addr, id, state) VALUES ($1, $2, $3, $4, $5)",
                event_id,
                appliance_settings.name,
                ieee_addr,
                appliance_settings.id,
                ApplianceStateType::On as ApplianceStateType
            ).execute(&self.shared_actor_state.db).await?;
        state.map.insert(ieee_addr.clone(), ApplianceStateType::On);

        Ok(())
    }
}

impl Actor for ApplianceState {
    type Msg = ApplianceEvents;
    type State = ApplianceStateState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let last_state = sqlx::query!(
            r#"
        SELECT appliances.ieee_addr, state as "state: ApplianceStateType"
        FROM
            (SELECT id, max(time) FROM appliances GROUP BY id) AS latest_state
            INNER JOIN appliances ON appliances.id = latest_state.id
        "#
        )
        .fetch_all(&self.shared_actor_state.db)
        .await?;

        let mut map = HashMap::new();

        for client in last_state {
            map.insert(client.ieee_addr.clone(), client.state);
        }

        Ok(ApplianceStateState {
            map,
            average_running: Default::default(),
        })
    }

    #[tracing::instrument(name = "appliance-state-actor", skip(self, _myself, message, state), level = Level::TRACE)]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ApplianceEvents::PowerUsage {
                ieee_addr,
                event_id,
                current,
                ..
            } => {
                let settings = self.shared_actor_state.settings.load();
                if let Some(appliance_settings) = settings.appliances.get(&ieee_addr) {
                    let avg = state
                        .average_running
                        .entry(ieee_addr.clone())
                        .and_modify(|v| {
                            v.push(current);
                        })
                        .or_insert_with(|| {
                            let mut avg = TimedAverage::new(Duration::from_secs(60 * 5));
                            avg.push(current);
                            avg
                        });

                    let average_current = avg.value();

                    let last_state = state.map.get(&ieee_addr);
                    if let Some(last_state) = last_state {
                        match last_state {
                            ApplianceStateType::On => {
                                if average_current < appliance_settings.current.threshold {
                                    self.turn_off_appliance(
                                        state,
                                        &ieee_addr,
                                        event_id,
                                        average_current,
                                        appliance_settings,
                                    )
                                    .await?;
                                }
                            }
                            ApplianceStateType::Off => {
                                if average_current >= appliance_settings.current.threshold {
                                    self.turn_on_appliance(
                                        state,
                                        &ieee_addr,
                                        event_id,
                                        average_current,
                                        appliance_settings,
                                    )
                                    .await?;
                                }
                            }
                        }
                    } else {
                        self.setup_initial_state(
                            state,
                            ieee_addr,
                            event_id,
                            average_current,
                            appliance_settings,
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
