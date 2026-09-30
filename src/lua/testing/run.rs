use crate::lua::LuaError;

pub struct LuaTestRun {
    pub result: Result<(), LuaError>,
    pub logs: Vec<String>,
}
