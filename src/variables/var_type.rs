use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VarType {
    String,
    Int,
    Float,
    Bool,
}

impl VarType {
    pub fn accepts(&self, other: VarType) -> bool {
        match (self, other) {
            (VarType::Float, VarType::Int) => true,
            (VarType::String, _) => true,
            _ => *self == other,
        }
    }
}

impl std::fmt::Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            VarType::String => "string",
            VarType::Int => "int",
            VarType::Float => "float",
            VarType::Bool => "bool",
        };

        f.write_str(name)
    }
}
