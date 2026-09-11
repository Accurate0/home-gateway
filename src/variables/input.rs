use std::collections::BTreeMap;

use super::{Node, Shape, Value, VarType};

pub fn input_shape(declared: &BTreeMap<String, VarType>) -> Shape {
    let mut shape = Shape::empty();

    for (key, ty) in declared {
        shape.insert(key.clone(), Shape::required(*ty));
    }

    shape
}

pub fn input_node(
    declared: &BTreeMap<String, VarType>,
    given: &BTreeMap<String, serde_json::Value>,
) -> Result<Node, String> {
    let expected = || {
        declared
            .iter()
            .map(|(key, ty)| format!("{key}: {ty}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if let Some(unknown) = given.keys().find(|key| !declared.contains_key(*key)) {
        return Err(format!(
            "unknown input `{unknown}`; expected: [{}]",
            expected()
        ));
    }

    let mut node = Node::empty();

    for (key, ty) in declared {
        let raw = given
            .get(key)
            .ok_or_else(|| format!("missing input `{key}`; expected: [{}]", expected()))?;

        let value = json_value(raw)
            .and_then(|value| value.coerce(*ty))
            .ok_or_else(|| format!("input `{key}` must be a {ty}, got {raw}"))?;

        node.insert(key.clone(), Node::Value(Some(value)));
    }

    Ok(node)
}

fn json_value(raw: &serde_json::Value) -> Option<Value> {
    match raw {
        serde_json::Value::String(value) => Some(Value::String(value.clone())),
        serde_json::Value::Bool(value) => Some(Value::Bool(*value)),
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(Value::Int)
            .or_else(|| number.as_f64().map(Value::Float)),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            None
        }
    }
}
