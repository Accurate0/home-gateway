use super::{Value, VarType};

pub trait ScalarVariable {
    const TYPE: VarType;

    fn to_value(&self) -> Value;
}

impl ScalarVariable for String {
    const TYPE: VarType = VarType::String;

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ScalarVariable for bool {
    const TYPE: VarType = VarType::Bool;

    fn to_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl ScalarVariable for f64 {
    const TYPE: VarType = VarType::Float;

    fn to_value(&self) -> Value {
        Value::Float(*self)
    }
}

impl ScalarVariable for f32 {
    const TYPE: VarType = VarType::Float;

    fn to_value(&self) -> Value {
        Value::Float(f64::from(*self))
    }
}

macro_rules! int_scalar {
    ($($ty:ty),*) => {
        $(
            impl ScalarVariable for $ty {
                const TYPE: VarType = VarType::Int;

                fn to_value(&self) -> Value {
                    Value::Int(i64::from(*self))
                }
            }
        )*
    };
}

int_scalar!(i64, i32, i16, u32, u16);
