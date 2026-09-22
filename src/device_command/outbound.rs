#[derive(Debug, Clone, PartialEq)]
pub enum Outbound {
    Json(serde_json::Value),
    Text(String),
}
