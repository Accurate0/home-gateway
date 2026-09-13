use schemars::{JsonSchema, Schema};

use crate::auth::scope::Scope;

#[derive(Clone, Copy)]
pub enum LuaType {
    String,
    Integer,
    Number,
    Boolean,
    Any,
    Table,
    Optional(&'static LuaType),
    Array(&'static LuaType),
    Map(&'static LuaType),
    Schema(fn() -> Schema),
    Class(&'static LuaClass),
}

pub struct LuaClass {
    pub name: &'static str,
    pub fields: &'static [LuaField],
}

pub struct LuaField {
    pub name: &'static str,
    pub ty: LuaType,
}

pub struct LuaParam {
    pub name: &'static str,
    pub ty: LuaType,
}

pub struct LuaFunction {
    pub name: &'static str,
    pub params: &'static [LuaParam],
    pub returns: Option<LuaType>,
    pub scope: Option<Scope>,
}

pub struct LuaNamespace {
    pub name: &'static str,
    pub fields: &'static [LuaField],
    pub functions: &'static [LuaFunction],
}

pub fn schema<T: JsonSchema>() -> Schema {
    schemars::schema_for!(T)
}
