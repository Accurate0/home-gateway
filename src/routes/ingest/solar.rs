use crate::{
    actors::solar::{SolarIngestActor, SolarMessage},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::StatusCode;

#[allow(unused)]
#[derive(serde::Deserialize)]
pub struct SolarIngestAvgPayload {
    pub mins_15: Option<f64>,
    pub mins_60: Option<f64>,
    pub mins_180: Option<f64>,
}

#[allow(unused)]
#[derive(serde::Deserialize)]
pub struct SolarIngestPayload {
    pub current_kwh: f64,
    pub average_kwh: SolarIngestAvgPayload,
    pub uv_level: Option<f64>,
}

pub async fn solar(
    State(ApiState { .. }): State<ApiState>,
    Json(solar_payload): Json<SolarIngestPayload>,
) -> StatusCode {
    if let Some(solar_actor) = ractor::registry::where_is(SolarIngestActor::NAME.to_string())
        && let Err(e) = solar_actor.send_message(SolarMessage::NewData(solar_payload)) {
            tracing::error!("error sending solar actor message: {e}");
        };

    StatusCode::NO_CONTENT
}
