use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};
use crate::lua::LuaCallContext;
use crate::routes::vars::{header_node, string_node};
use crate::state::AppState;
use crate::variables::{Node, Value, Vars};

pub async fn lua_ingest(
    Auth(auth): Auth,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<BTreeMap<String, String>>,
    headers: HeaderMap,
    body: String,
) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::IngestLua, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let Some(source) = state.settings.ingest.source(&name) else {
        tracing::warn!("no scripted ingest source named `{name}`");
        return StatusCode::NOT_FOUND;
    };

    let event_id = Uuid::new_v4();
    let vars = Vars::default()
        .with("raw", Node::Value(Some(Value::String(body))))
        .with("query", string_node(query))
        .with("headers", header_node(&headers));

    let cx = LuaCallContext::new(state.clone(), event_id, format!("ingest:{name}"));

    match state.lua.run_unit(&cx, &source.script, &vars).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(error) => {
            tracing::error!("[{event_id}] scripted ingest `{name}` failed: {error}");

            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
