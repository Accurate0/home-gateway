use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};
use crate::error::AppError;
use crate::lua::{Script, execute};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LuaExecutePayload {
    pub script: String,
    #[serde(default)]
    pub vars: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct LuaExecuteResponse {
    pub result: serde_json::Value,
}

pub async fn lua_execute(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Json(payload): Json<LuaExecutePayload>,
) -> Result<Response, AppError> {
    auth.require(&Scope::new(Resource::Lua, Action::Write))
        .map_err(AppError::StatusCode)?;

    let script = match Script::parse(&payload.script) {
        Ok(script) => script,
        Err(error) => return Ok((StatusCode::BAD_REQUEST, error).into_response()),
    };

    match execute::execute(&state, auth, &script, payload.vars, payload.dry_run).await {
        Ok(result) => Ok(Json(LuaExecuteResponse { result }).into_response()),
        Err(error) => Ok((StatusCode::BAD_REQUEST, error.to_string()).into_response()),
    }
}
