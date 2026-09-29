use serde_json::{Map, Value};

pub enum Command {
    SetDps(Map<String, Value>),
}
