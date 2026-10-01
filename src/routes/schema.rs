use crate::{
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
    state::AppState,
};
use axum::extract::State;

pub async fn schema(State(state): State<AppState>, auth: AuthContext) -> Result<String, AppError> {
    auth.require(Resource::Schema, Action::Read)?;

    Ok(state.schema.sdl())
}
