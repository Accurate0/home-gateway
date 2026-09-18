use std::collections::HashMap;
use std::time::Instant;

use crate::lua::LuaDecoder;

pub struct WorkerState {
    pub decoder: LuaDecoder,
    pub last_latest_state_write: HashMap<String, Instant>,
}
