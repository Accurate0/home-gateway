use crate::integrations::solar::queries::{self, SolarQueryError};
use crate::integrations::solar::types::{
    SolarCurrentResponse, SolarHistoryResponse, SolarHistoryTwoDayResponse,
};
use crate::state::ApiState;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Query, extract::State};
use chrono::{NaiveDateTime, Utc};
use http::StatusCode;
use serde::Deserialize;

pub struct SolarError(SolarQueryError);

impl IntoResponse for SolarError {
    fn into_response(self) -> Response {
        tracing::error!("solar api error: {:?}", self.0);

        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

impl From<SolarQueryError> for SolarError {
    fn from(e: SolarQueryError) -> Self {
        Self(e)
    }
}

pub async fn current(
    State(ApiState { db, .. }): State<ApiState>,
) -> Result<Json<SolarCurrentResponse>, SolarError> {
    Ok(Json(queries::current(&db).await?))
}

pub async fn history(
    State(ApiState { db, .. }): State<ApiState>,
) -> Result<Json<SolarHistoryTwoDayResponse>, SolarError> {
    let (today, yesterday) = queries::history_last_two_days(&db).await?;

    Ok(Json(SolarHistoryTwoDayResponse { today, yesterday }))
}

#[derive(Deserialize)]
pub struct SolarHistoryQueryParams {
    since: NaiveDateTime,
}

pub async fn history_since(
    State(ApiState { db, .. }): State<ApiState>,
    params: Query<SolarHistoryQueryParams>,
) -> Result<Json<SolarHistoryResponse>, SolarError> {
    let since = queries::clamp_since(params.since.and_utc(), Utc::now());
    let history = queries::history_since(&db, since).await?;

    Ok(Json(SolarHistoryResponse { history }))
}

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}
