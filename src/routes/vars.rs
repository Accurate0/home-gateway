use std::collections::BTreeMap;

use axum::http::HeaderMap;
use http::header::HeaderName;

use crate::variables::{Node, Value};

pub fn string_node(values: BTreeMap<String, String>) -> Node {
    let mut node = Node::empty();

    for (key, value) in values {
        node.insert(key, Node::Value(Some(Value::String(value))));
    }

    node
}

pub fn header_node(headers: &HeaderMap) -> Node {
    let mut node = Node::empty();

    for (name, value) in headers {
        let Ok(value) = value.to_str() else {
            continue;
        };

        node.insert(
            HeaderName::as_str(name).to_owned(),
            Node::Value(Some(Value::String(value.to_owned()))),
        );
    }

    node
}

pub fn header_map(headers: &HeaderMap) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();

    for (name, value) in headers {
        let Ok(value) = value.to_str() else {
            continue;
        };

        map.insert(
            HeaderName::as_str(name).to_owned(),
            serde_json::Value::String(value.to_owned()),
        );
    }

    map
}
