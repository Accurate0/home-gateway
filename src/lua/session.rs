use mlua::Lua;

use super::{LuaCallContext, LuaEngine, LuaError, Script};

pub struct LuaSession {
    engine: LuaEngine,
    lua: Lua,
    cx: LuaCallContext,
}

impl LuaSession {
    pub(super) fn new(engine: LuaEngine, lua: Lua, cx: LuaCallContext) -> Self {
        LuaSession { engine, lua, cx }
    }

    pub fn context(&self) -> &LuaCallContext {
        &self.cx
    }

    pub async fn eval(&self, script: &Script) -> Result<serde_json::Value, LuaError> {
        self.engine.eval_session(&self.cx, &self.lua, script).await
    }
}
