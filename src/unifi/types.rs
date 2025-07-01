use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct UnifiConnectedClients {
    pub clients: Vec<UnifiClient>,
}

#[derive(Serialize, Deserialize)]
pub struct UnifiClient {
    pub id: String,
    pub name: String,
    pub connected_at: String,
    pub ip_address: String,
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UnifiConnectedClientsResponse {
    pub offset: i64,
    pub limit: i64,
    pub count: i64,
    pub total_count: i64,
    pub data: Vec<ConnectedClients>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConnectedClients {
    pub id: String,
    pub name: String,
    pub connected_at: String,
    pub ip_address: String,
    pub access: Access,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Access {
    #[serde(rename = "type")]
    pub type_field: String,
}
