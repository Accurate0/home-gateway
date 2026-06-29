use crate::{
    auth::{Auth, scope::required},
    types::{ApiState, AppError},
};
use axum::extract::State;

pub async fn schema(
    State(ApiState { schema, .. }): State<ApiState>,
    Auth(auth): Auth,
) -> Result<String, AppError> {
    auth.require(&required::REST_SCHEMA_READ)
        .map_err(AppError::StatusCode)?;

    Ok(schema.sdl())
}
