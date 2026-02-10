use crate::types::ApiState;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub image_url: Option<String>,
}

pub async fn config(State(ApiState { .. }): State<ApiState>) -> Json<EpdConfig> {
    Json(EpdConfig {
        refresh_interval_mins: Some(15),
        image_url: Some("https://home.anurag.sh/v1/epd/latest".to_string()),
    })
}
