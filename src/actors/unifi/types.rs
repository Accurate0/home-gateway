use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiWebhookEvents {
    pub events: Vec<UnifiWebhookEvent>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiWebhookEvent {
    #[serde(rename = "alert_id")]
    pub alert_id: String,
    #[serde(rename = "alert_key")]
    pub alert_key: String,
    pub id: String,
    pub scope: UnifiWebHookScope,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiWebHookScope {
    #[serde(rename = "client_device_id")]
    pub client_mac_address: String,
    #[serde(rename = "site_id")]
    pub site_id: String,
}
