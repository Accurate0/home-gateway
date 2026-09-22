use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct NotifyAndroidSettings {
    pub fcm_project_id: String,
    #[serde(default)]
    pub fcm_service_account_json: String,
}
