use serde::Serialize;
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
