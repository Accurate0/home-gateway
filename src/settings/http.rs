use schemars::JsonSchema;
use serde::Deserialize;
use std::net::SocketAddr;

use super::HttpClientsSettings;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct HttpSettings {
    pub listen_address: SocketAddr,
    pub clients: HttpClientsSettings,
}
