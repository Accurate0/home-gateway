use schemars::JsonSchema;
use serde::Deserialize;

use crate::variables::{Value, VarType};

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl Literal {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();

        for quote in ['"', '\''] {
            if let Some(inner) = input
                .strip_prefix(quote)
                .and_then(|rest| rest.strip_suffix(quote))
            {
                return Ok(Literal::String(inner.to_owned()));
            }
        }

        match input {
            "true" => return Ok(Literal::Bool(true)),
            "false" => return Ok(Literal::Bool(false)),
            _ => {}
        }

        if let Ok(value) = input.parse::<i64>() {
            return Ok(Literal::Int(value));
        }

        if let Ok(value) = input.parse::<f64>() {
            return Ok(Literal::Float(value));
        }

        Err(format!(
            "invalid literal `{input}`; expected a quoted string, number or bool"
        ))
    }

    pub fn var_type(&self) -> VarType {
        match self {
            Literal::String(_) => VarType::String,
            Literal::Int(_) => VarType::Int,
            Literal::Float(_) => VarType::Float,
            Literal::Bool(_) => VarType::Bool,
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            Literal::String(value) => Value::String(value.clone()),
            Literal::Int(value) => Value::Int(*value),
            Literal::Float(value) => Value::Float(*value),
            Literal::Bool(value) => Value::Bool(*value),
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(value) => write!(f, "\"{value}\""),
            Literal::Int(value) => write!(f, "{value}"),
            Literal::Float(value) => write!(f, "{value}"),
            Literal::Bool(value) => write!(f, "{value}"),
        }
    }
}
