use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{LuaType, LuaTyped, schema};

pub struct Json<T>(pub T);

impl<T: JsonSchema> LuaTyped for Json<T> {
    const TYPE: LuaType = LuaType::Schema(schema::<T>);
}

impl<T: DeserializeOwned> FromLua for Json<T> {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value).map(Json)
    }
}

impl<T: Serialize> IntoLua for Json<T> {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use mlua::{FromLua, IntoLua, Lua};

    use super::Json;
    use crate::workflows::mode::Mode;

    #[test]
    fn a_schema_type_round_trips_as_its_serde_form() {
        let lua = Lua::new();

        let value = Json(Mode::Vacation)
            .into_lua(&lua)
            .expect("expected a lua value");

        assert_eq!(value.to_string().expect("expected a string"), "vacation");

        let Json(mode) = Json::<Mode>::from_lua(value, &lua).expect("expected a mode");

        assert_eq!(mode, Mode::Vacation);
    }

    #[test]
    fn an_unknown_variant_is_rejected() {
        let lua = Lua::new();
        let value = "holiday".into_lua(&lua).expect("expected a lua value");

        assert!(Json::<Mode>::from_lua(value, &lua).is_err());
    }
}
