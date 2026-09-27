use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

use crate::device_registry::IdOrAlias;

#[Scalar(name = "IdOrAlias")]
impl ScalarType for IdOrAlias {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(reference) => Ok(IdOrAlias(reference)),
            other => Err(InputValueError::expected_type(other)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.clone())
    }
}
