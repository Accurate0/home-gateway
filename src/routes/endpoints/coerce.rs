use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::settings::ParamType;
use crate::variables::VarType;

pub fn coerce(
    contract: &BTreeMap<String, ParamType>,
    query: BTreeMap<String, String>,
) -> Result<Map<String, Value>, String> {
    let mut coerced = Map::new();

    for (name, raw) in query {
        let value = match contract.get(&name) {
            Some(param) => scalar(&name, &raw, param.ty)?,
            None => Value::String(raw),
        };

        coerced.insert(name, value);
    }

    for (name, param) in contract {
        if param.required && !coerced.contains_key(name) {
            return Err(format!("`{name}` is a required query parameter"));
        }
    }

    Ok(coerced)
}

fn scalar(name: &str, raw: &str, ty: VarType) -> Result<Value, String> {
    let value = match ty {
        VarType::String => Value::String(raw.to_owned()),
        VarType::Int => raw
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| format!("`{name}` must be a {ty}, got `{raw}`"))?,
        VarType::Float => raw
            .parse::<f64>()
            .map(Value::from)
            .map_err(|_| format!("`{name}` must be a {ty}, got `{raw}`"))?,
        VarType::Bool => match raw {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => return Err(format!("`{name}` must be a {ty}, got `{raw}`")),
        },
    };

    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::coerce;
    use crate::settings::ParamType;

    fn contract(pairs: &[(&str, &str)]) -> BTreeMap<String, ParamType> {
        pairs
            .iter()
            .map(|(name, ty)| {
                (
                    (*name).to_owned(),
                    ParamType::try_from((*ty).to_owned()).expect("expected a param type"),
                )
            })
            .collect()
    }

    fn query(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn a_declared_parameter_is_coerced_to_its_type() {
        let coerced = coerce(
            &contract(&[("hours", "int"), ("ratio", "float"), ("on", "bool")]),
            query(&[("hours", "3"), ("ratio", "1.5"), ("on", "true")]),
        )
        .expect("expected the query to coerce");

        assert_eq!(coerced["hours"], serde_json::json!(3));
        assert_eq!(coerced["ratio"], serde_json::json!(1.5));
        assert_eq!(coerced["on"], serde_json::json!(true));
    }

    #[test]
    fn an_undeclared_parameter_stays_a_string() {
        let coerced = coerce(&contract(&[]), query(&[("room", "kitchen")]))
            .expect("expected the query to coerce");

        assert_eq!(coerced["room"], serde_json::json!("kitchen"));
    }

    #[test]
    fn a_missing_required_parameter_is_named() {
        let error =
            coerce(&contract(&[("room", "string")]), query(&[])).expect_err("expected a rejection");

        assert!(error.contains("`room` is a required query parameter"));
    }

    #[test]
    fn a_missing_optional_parameter_is_allowed() {
        let coerced =
            coerce(&contract(&[("room", "string?")]), query(&[])).expect("expected no parameters");

        assert!(coerced.is_empty());
    }

    #[test]
    fn an_uncoercible_parameter_is_named() {
        let error = coerce(&contract(&[("hours", "int")]), query(&[("hours", "soon")]))
            .expect_err("expected a rejection");

        assert!(error.contains("`hours` must be a int"));
    }
}
