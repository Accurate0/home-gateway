use mlua::{Lua, Table};

use super::{LuaCallContext, LuaFunction};

pub trait LuaModule: Send + Sync + 'static {
    fn namespace(&self) -> &'static str;

    fn functions(&self) -> &'static [LuaFunction];

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()>;
}

#[cfg(test)]
mod tests {
    use mlua::{FromLua, IntoLua, Lua, Table};

    use super::LuaModule;
    use crate::auth::scope::{Action, Resource, Scope};
    use crate::lua::{Json, LuaCallContext, LuaClass, LuaType, lua_module};
    use crate::workflows::mode::Mode;

    #[derive(LuaClass, Debug, PartialEq)]
    #[lua(name = "SampleReading")]
    struct Reading {
        room: String,
        #[lua(rename = "celsius")]
        temperature: Option<f64>,
    }

    struct SampleLua;

    #[lua_module(namespace = "sample")]
    impl SampleLua {
        #[lua(scope = Device::Read)]
        async fn reading(_cx: &LuaCallContext, room: String) -> mlua::Result<Option<Reading>> {
            Ok(Some(Reading {
                room,
                temperature: None,
            }))
        }

        #[lua(name = "set")]
        fn store(_lua: &Lua, mode: Json<Mode>, rooms: Vec<String>) -> mlua::Result<()> {
            let _ = (mode.0, rooms);

            Ok(())
        }

        fn helper() -> usize {
            Self::FUNCTIONS.len()
        }
    }

    #[test]
    fn functions_are_declared_from_their_signatures() {
        assert_eq!(SampleLua.namespace(), "sample");
        assert_eq!(SampleLua::helper(), 2);
        assert_eq!(SampleLua::READING, "sample.reading");
        assert_eq!(SampleLua::STORE, "sample.set");

        let [reading, set] = SampleLua.functions() else {
            panic!("expected two functions");
        };

        assert_eq!(reading.name, "reading");
        assert_eq!(
            reading.scope,
            Some(Scope::new(Resource::Device, Action::Read))
        );

        let [room] = reading.params else {
            panic!("expected the context parameter to be left out");
        };

        assert_eq!(room.name, "room");
        assert!(matches!(room.ty, LuaType::String));

        let Some(LuaType::Optional(LuaType::Class(class))) = reading.returns else {
            panic!("expected an optional class");
        };

        assert_eq!(class.name, "SampleReading");

        assert_eq!(set.name, "set");
        assert_eq!(set.scope, None);
        assert!(set.returns.is_none());

        let [mode, rooms] = set.params else {
            panic!("expected the lua parameter to be left out");
        };

        assert_eq!(mode.name, "mode");
        assert!(matches!(mode.ty, LuaType::Schema(_)));
        assert_eq!(rooms.name, "rooms");
        assert!(matches!(rooms.ty, LuaType::Array(LuaType::String)));
    }

    #[test]
    fn a_class_declares_its_fields_and_round_trips() {
        let names: Vec<&str> = Reading::CLASS.fields.iter().map(|f| f.name).collect();

        assert_eq!(names, vec!["room", "celsius"]);
        assert!(matches!(
            Reading::CLASS.fields[1].ty,
            LuaType::Optional(LuaType::Number)
        ));

        let lua = Lua::new();

        let reading = Reading {
            room: "lounge".to_owned(),
            temperature: Some(21.5),
        };

        let value = reading.into_lua(&lua).expect("expected a table");
        let table = Table::from_lua(value.clone(), &lua).expect("expected a table");

        assert_eq!(table.get::<f64>("celsius").expect("celsius"), 21.5);

        let restored = Reading::from_lua(value, &lua).expect("expected a reading");

        assert_eq!(
            restored,
            Reading {
                room: "lounge".to_owned(),
                temperature: Some(21.5),
            }
        );
    }
}
