use async_graphql::Enum;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize, Enum, Eq, Copy,
)]
#[sqlx(type_name = "appliance_state", rename_all = "lowercase")]
pub enum ApplianceStateType {
    On,
    Off,
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize, Enum, Eq, Copy,
)]
#[sqlx(type_name = "unifi_state", rename_all = "lowercase")]
pub enum UnifiState {
    Connected,
    Disconnected,
}
