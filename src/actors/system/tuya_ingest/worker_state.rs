use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::lua::LuaDecoder;

pub struct WorkerState {
    pub decoder: LuaDecoder,
    pub dps: HashMap<String, Map<String, Value>>,
}
