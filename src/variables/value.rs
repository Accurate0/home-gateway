use serde::{Deserialize, Serialize};

use super::VarType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Value {
    pub fn var_type(&self) -> VarType {
        match self {
            Value::String(_) => VarType::String,
            Value::Int(_) => VarType::Int,
            Value::Float(_) => VarType::Float,
            Value::Bool(_) => VarType::Bool,
        }
    }

    pub fn render(&self) -> String {
        match self {
            Value::String(value) => value.clone(),
            Value::Int(value) => value.to_string(),
            Value::Float(value) => render_float(*value),
            Value::Bool(value) => value.to_string(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(value) => Some(*value as f64),
            Value::Float(value) => Some(*value),
            Value::String(_) | Value::Bool(_) => None,
        }
    }

    pub fn coerce(self, target: VarType) -> Option<Value> {
        match (self, target) {
            (Value::Int(value), VarType::Float) => Some(Value::Float(value as f64)),
            (value, VarType::String) => Some(Value::String(value.render())),
            (value, target) if value.var_type() == target => Some(value),
            _ => None,
        }
    }
}

fn render_float(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}
