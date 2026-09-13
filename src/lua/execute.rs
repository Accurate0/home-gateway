use std::collections::BTreeMap;

use uuid::Uuid;

use crate::auth::AuthContext;
use crate::state::AppState;
use crate::variables::{Node, Value, Vars};

use super::{LuaAuthority, LuaCallContext, LuaError, Script};

pub async fn execute(
    state: &AppState,
    auth: AuthContext,
    script: &Script,
    vars: BTreeMap<String, serde_json::Value>,
    dry_run: bool,
) -> Result<serde_json::Value, LuaError> {
    let event_id = Uuid::new_v4();
    let authority = LuaAuthority::delegated(auth);

    let cx = LuaCallContext::new(state.clone(), event_id, "adhoc")
        .with_dry_run(dry_run)
        .with_authority(authority);

    let mut namespaces = Vars::default();

    for (namespace, value) in vars {
        namespaces.insert(namespace, json_node(&value));
    }

    tracing::info!(
        "[{event_id}] executing an ad-hoc lua script for {} (dry_run: {dry_run})",
        cx.authority.describe()
    );

    state.lua.run_json(&cx, script, &namespaces).await
}

fn json_node(raw: &serde_json::Value) -> Node {
    match raw {
        serde_json::Value::Object(fields) => {
            let mut node = Node::empty();

            for (key, field) in fields {
                node.insert(key.clone(), json_node(field));
            }

            node
        }
        serde_json::Value::String(value) => Node::Value(Some(Value::String(value.clone()))),
        serde_json::Value::Bool(value) => Node::Value(Some(Value::Bool(*value))),
        serde_json::Value::Number(number) => Node::Value(
            number
                .as_i64()
                .map(Value::Int)
                .or_else(|| number.as_f64().map(Value::Float)),
        ),
        serde_json::Value::Null | serde_json::Value::Array(_) => Node::Value(None),
    }
}
