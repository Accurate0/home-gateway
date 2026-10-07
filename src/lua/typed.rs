use std::collections::BTreeMap;

use mlua::{Function, LuaString, MultiValue, Table, Value};

use super::LuaType;

pub trait LuaTyped {
    const TYPE: LuaType;
}

impl LuaTyped for String {
    const TYPE: LuaType = LuaType::String;
}

impl LuaTyped for LuaString {
    const TYPE: LuaType = LuaType::String;
}

impl LuaTyped for i32 {
    const TYPE: LuaType = LuaType::Integer;
}

impl LuaTyped for i64 {
    const TYPE: LuaType = LuaType::Integer;
}

impl LuaTyped for u32 {
    const TYPE: LuaType = LuaType::Integer;
}

impl LuaTyped for u64 {
    const TYPE: LuaType = LuaType::Integer;
}

impl LuaTyped for usize {
    const TYPE: LuaType = LuaType::Integer;
}

impl LuaTyped for f64 {
    const TYPE: LuaType = LuaType::Number;
}

impl LuaTyped for bool {
    const TYPE: LuaType = LuaType::Boolean;
}

impl LuaTyped for Value {
    const TYPE: LuaType = LuaType::Any;
}

impl LuaTyped for MultiValue {
    const TYPE: LuaType = LuaType::Any;
}

impl LuaTyped for Table {
    const TYPE: LuaType = LuaType::Table;
}

impl LuaTyped for Function {
    const TYPE: LuaType = LuaType::Function;
}

impl<T: LuaTyped> LuaTyped for Option<T> {
    const TYPE: LuaType = LuaType::Optional(&T::TYPE);
}

impl<T: LuaTyped> LuaTyped for Vec<T> {
    const TYPE: LuaType = LuaType::Array(&T::TYPE);
}

impl<T: LuaTyped> LuaTyped for BTreeMap<String, T> {
    const TYPE: LuaType = LuaType::Map(&T::TYPE);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{LuaType, LuaTyped};

    #[test]
    fn nested_containers_keep_their_shape() {
        let LuaType::Optional(LuaType::Array(LuaType::String)) = <Option<Vec<String>>>::TYPE else {
            panic!("expected an optional array of strings");
        };

        let LuaType::Map(LuaType::Integer) = <BTreeMap<String, i64>>::TYPE else {
            panic!("expected a map of integers");
        };
    }
}
