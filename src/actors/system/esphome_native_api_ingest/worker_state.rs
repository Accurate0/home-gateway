use std::collections::HashMap;

use serde_json::Value;

use crate::lua::LuaDecoder;

pub struct WorkerState {
    pub decoder: LuaDecoder,
    pub entities: HashMap<String, HashMap<String, Value>>,
}
