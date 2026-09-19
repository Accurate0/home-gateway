use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node {
    Value(Option<Value>),
    Object(BTreeMap<String, Node>),
}

impl Node {
    pub fn empty() -> Self {
        Node::Object(BTreeMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, node: Node) {
        if let Node::Object(fields) = self {
            fields.insert(key.into(), node);
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Node::Value(None) => serde_json::Value::Null,
            Node::Value(Some(Value::String(value))) => serde_json::Value::from(value.clone()),
            Node::Value(Some(Value::Int(value))) => serde_json::Value::from(*value),
            Node::Value(Some(Value::Float(value))) => serde_json::Value::from(*value),
            Node::Value(Some(Value::Bool(value))) => serde_json::Value::from(*value),
            Node::Object(fields) => fields
                .iter()
                .map(|(key, node)| (key.clone(), node.to_json()))
                .collect::<serde_json::Map<_, _>>()
                .into(),
        }
    }

    pub fn lookup(&self, segments: &[String]) -> Option<&Node> {
        let Some((first, rest)) = segments.split_first() else {
            return Some(self);
        };

        match self {
            Node::Object(fields) => fields.get(first)?.lookup(rest),
            Node::Value(_) => None,
        }
    }
}
