use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    state::AppState,
};
use axum::extract::State;

pub async fn schema(State(state): State<AppState>, Auth(auth): Auth) -> Result<String, AppError> {
    auth.require(&Scope::new(Resource::Schema, Action::Read))
        .map_err(AppError::StatusCode)?;

    Ok(state.schema.sdl())
}
