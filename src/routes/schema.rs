use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    graphql::FinalSchema,
};
use axum::extract::State;

pub async fn schema(
    State(schema): State<FinalSchema>,
    Auth(auth): Auth,
) -> Result<String, AppError> {
    auth.require(&Scope::new(Resource::Schema, Action::Read))
        .map_err(AppError::StatusCode)?;

    Ok(schema.sdl())
}
