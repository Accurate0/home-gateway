use std::collections::HashMap;

use serde_json::Value;

use crate::lua::LuaDecoder;

/// The node reports one entity per message, so the worker keeps the last value
/// of every entity: a model needs the whole snapshot to build a role block that
/// spans several entities (media player metadata lives in its own text sensors).
pub struct WorkerState {
    pub decoder: LuaDecoder,
    pub entities: HashMap<String, HashMap<String, Value>>,
}
