use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct FcmSendRequest {
    pub message: FcmMessage,
}

#[derive(Serialize)]
pub struct FcmMessage {
    pub token: String,
    pub data: HashMap<String, String>,
    pub android: FcmAndroidConfig,
}

#[derive(Serialize)]
pub struct FcmAndroidConfig {
    pub priority: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushAction {
    pub label: String,
    #[serde(flatten)]
    pub kind: PushActionKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PushActionKind {
    RunWorkflow { slug: String },
    Snooze { seconds: u64 },
    Dismiss,
    Acknowledge,
}
