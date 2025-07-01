use crate::{types::SharedActorState, unifi::types::UnifiConnectedClients};
use ractor::Actor;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "unifi_state", rename_all = "lowercase")]
pub enum UnifiState {
    Connected,
    Disconnected,
}

pub enum Message {
    NewEvent {
        clients: UnifiConnectedClients,
        event_id: Uuid,
    },
}

pub struct UnifiClientState {
    pub map: HashMap<String, UnifiState>,
    pub client_id_to_name_map: HashMap<String, String>,
}

pub struct UnifiConnectedClientHandler {
    pub shared_actor_state: SharedActorState,
}

impl UnifiConnectedClientHandler {
    pub const NAME: &str = "unifi-connected-clients";
}

impl Actor for UnifiConnectedClientHandler {
    type Msg = Message;
    type State = UnifiClientState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let last_state = sqlx::query!(
            r#"
        SELECT unifi_clients.id, unifi_clients.name, state as "state: UnifiState"
        FROM
            (SELECT id, max(time) FROM unifi_clients GROUP BY id) AS latest_state
            INNER JOIN unifi_clients ON unifi_clients.id = latest_state.id
        "#
        )
        .fetch_all(&self.shared_actor_state.db)
        .await?;

        let mut map = HashMap::new();
        let mut client_id_to_name_map = HashMap::new();

        for client in last_state {
            map.insert(client.id.clone(), client.state);
            client_id_to_name_map.insert(client.id, client.name);
        }

        Ok(UnifiClientState {
            map,
            client_id_to_name_map,
        })
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            Message::NewEvent {
                clients: unifi_clients,
                event_id,
            } => {
                for client in &unifi_clients.clients {
                    let already_exists = state.map.get(&client.id).is_some();

                    if !already_exists {
                        sqlx::query!(r#"
                        INSERT INTO unifi_clients (event_id, name, id, state) VALUES ($1, $2, $3, $4)
                        "#,
                            event_id,
                            client.name,
                            client.id,
                            UnifiState::Connected as UnifiState)
                        .execute(&self.shared_actor_state.db).await?;

                        state.map.insert(client.id.clone(), UnifiState::Connected);
                        state
                            .client_id_to_name_map
                            .insert(client.id.clone(), client.name.clone());
                    }
                }

                let all_known_connected = state
                    .map
                    .iter()
                    .filter(|(_, v)| **v == UnifiState::Connected)
                    .map(|(k, _)| k.clone())
                    .collect::<HashSet<_>>();

                let all_currently_connected = unifi_clients
                    .clients
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<HashSet<_>>();

                let clients_to_mark_connected = all_currently_connected
                    .difference(&all_known_connected)
                    .collect::<HashSet<_>>();

                for client_to_mark_connected in clients_to_mark_connected {
                    let client = unifi_clients
                        .clients
                        .iter()
                        .find(|c| c.id == **client_to_mark_connected);
                    match client {
                        Some(client) => {
                            sqlx::query!(
                                "INSERT INTO unifi_clients (event_id, name, id, state) VALUES ($1, $2, $3, $4)",
                                event_id,
                                client.name,
                                client.id,
                                UnifiState::Connected as UnifiState
                            ).execute(&self.shared_actor_state.db).await?;
                            state.map.insert(client.id.clone(), UnifiState::Connected);
                        }
                        None => tracing::warn!("client {client_to_mark_connected} not found"),
                    }
                }

                let clients_to_mark_disconnected = all_known_connected
                    .difference(&all_currently_connected)
                    .collect::<HashSet<_>>();

                for client_to_mark_disconnected in clients_to_mark_disconnected {
                    let client_name = state.client_id_to_name_map.get(client_to_mark_disconnected);
                    match client_name {
                        Some(name) => {
                            sqlx::query!(
                                "INSERT INTO unifi_clients (event_id, name, id, state) VALUES ($1, $2, $3, $4)",
                                event_id,
                                name,
                                client_to_mark_disconnected,
                                UnifiState::Disconnected as UnifiState
                            ).execute(&self.shared_actor_state.db).await?;
                            state.map.insert(
                                client_to_mark_disconnected.clone(),
                                UnifiState::Disconnected,
                            );
                        }
                        None => {
                            tracing::warn!("client name {client_to_mark_disconnected} not found")
                        }
                    };
                }
            }
        }

        Ok(())
    }
}
