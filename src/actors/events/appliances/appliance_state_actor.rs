use crate::{
    settings::{ApplianceSettings, IEEEAddress},
    types::SharedActorState,
};
use ractor::Actor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ApplianceEvents;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "appliance_state", rename_all = "lowercase")]
pub enum ApplianceStateType {
    On,
    Off,
}

pub struct ApplianceStateState {
    pub map: HashMap<IEEEAddress, ApplianceStateType>,
}

pub struct ApplianceState {
    pub shared_actor_state: SharedActorState,
    pub appliance_settings: HashMap<IEEEAddress, ApplianceSettings>,
}

impl ApplianceState {
    pub const NAME: &str = "appliance-state";
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

        Ok(ApplianceStateState { map })
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            ApplianceEvents::PowerUsage {
                ieee_addr,
                power,
                event_id,
                ..
            } => {
                if let Some(appliance_settings) = self.appliance_settings.get(&ieee_addr) {
                    let last_state = state.map.get(&ieee_addr);
                    if let Some(last_state) = last_state {
                        match last_state {
                            ApplianceStateType::On => {
                                if power <= appliance_settings.power.off_threshold {
                                    tracing::info!(
                                        "threshold reached for {ieee_addr} - {}, turning on",
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
                                }
                            }
                            ApplianceStateType::Off => {
                                if power > appliance_settings.power.on_threshold {
                                    tracing::info!(
                                        "threshold reached for {ieee_addr} - {}, turning off",
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
                                }
                            }
                        }
                    } else {
                        let on_or_off = if power > appliance_settings.power.on_threshold {
                            ApplianceStateType::On
                        } else if power <= appliance_settings.power.off_threshold {
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
                    }
                }
            }
        }

        Ok(())
    }
}
