use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::auth::{
    AuthContext,
    scope::{Action, Resource},
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
    auth: AuthContext,
    Json(payload): Json<LuaExecutePayload>,
) -> Result<Json<LuaExecuteResponse>, AppError> {
    auth.require(Resource::Lua, Action::Write)?;

    let script = Script::parse(&payload.script).map_err(AppError::bad_request)?;

    let result = execute::execute(&state, auth, &script, payload.vars, payload.dry_run)
        .await
        .map_err(|error| AppError::bad_request(error.to_string()))?;

    Ok(Json(LuaExecuteResponse { result }))
}
