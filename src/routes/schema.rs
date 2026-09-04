use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    state::ApiState,
};
use axum::extract::State;

pub async fn schema(
    State(ApiState { schema, .. }): State<ApiState>,
    Auth(auth): Auth,
) -> Result<String, AppError> {
    auth.require(&Scope::new(Resource::Schema, Action::Read))
        .map_err(AppError::StatusCode)?;

    Ok(schema.sdl())
}
