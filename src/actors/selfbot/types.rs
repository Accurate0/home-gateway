#[derive(serde::Serialize)]
pub struct SelfBotMessageRequest {
    pub message: String,
    pub channel_id: u64,
}
