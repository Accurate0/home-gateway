use mlua::{Lua, Table};

use super::{LuaCallContext, LuaFunction};

pub trait LuaModule: Send + Sync + 'static {
    fn namespace(&self) -> &'static str;

    fn functions(&self) -> &'static [LuaFunction];

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()>;
}
