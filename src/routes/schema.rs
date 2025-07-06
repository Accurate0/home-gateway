use crate::types::ApiState;
use axum::extract::State;

pub async fn schema(State(ApiState { schema, .. }): State<ApiState>) -> String {
    schema.sdl()
}
