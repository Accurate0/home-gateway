use serde_json::{Map, Value};

pub struct DpsUpdate {
    pub address: String,
    pub dps: Map<String, Value>,
}
