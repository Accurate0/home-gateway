use std::sync::Arc;

use mlua::Lua;

use super::{LuaCallContext, LuaModule, LuaNamespace};

pub const RESERVED_NAMESPACES: [&str; 5] = ["event", "input", "lua", "fuelwatch", "willyweather"];

#[derive(Default)]
pub struct LuaApiRegistryBuilder {
    modules: Vec<Arc<dyn LuaModule>>,
}

impl LuaApiRegistryBuilder {
    pub fn insert<M: LuaModule>(mut self, module: M) -> Self {
        self.modules.push(Arc::new(module));

        self
    }

    pub fn insert_optional<M: LuaModule>(self, module: Option<M>) -> Self {
        match module {
            Some(module) => self.insert(module),
            None => self,
        }
    }

    pub fn build(self) -> LuaApiRegistry {
        assert_namespaces(&self.modules);

        LuaApiRegistry {
            modules: Arc::new(self.modules),
        }
    }
}

#[derive(Clone, Default)]
pub struct LuaApiRegistry {
    modules: Arc<Vec<Arc<dyn LuaModule>>>,
}

impl LuaApiRegistry {
    pub fn builder() -> LuaApiRegistryBuilder {
        LuaApiRegistryBuilder::default()
    }

    pub fn namespaces(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.namespace())
            .collect()
    }

    pub fn api(&self) -> Vec<LuaNamespace> {
        self.modules
            .iter()
            .map(|module| LuaNamespace {
                name: module.namespace(),
                fields: &[],
                functions: module.functions(),
            })
            .collect()
    }

    pub fn install(&self, lua: &Lua, cx: &LuaCallContext) -> mlua::Result<()> {
        let globals = lua.globals();

        for module in self.modules.iter() {
            let table = lua.create_table()?;

            module.register(lua, &table, cx)?;

            if table.is_empty() {
                tracing::debug!(
                    "withholding lua namespace `{}` from {}",
                    module.namespace(),
                    cx.authority.describe()
                );

                continue;
            }

            globals.set(module.namespace(), table)?;
        }

        Ok(())
    }
}

fn assert_namespaces(modules: &[Arc<dyn LuaModule>]) {
    let mut seen: Vec<&'static str> = Vec::with_capacity(modules.len());

    for module in modules {
        let namespace = module.namespace();

        if RESERVED_NAMESPACES.contains(&namespace) {
            panic!("lua namespace `{namespace}` is reserved for workflow variables");
        }

        if seen.contains(&namespace) {
            panic!("lua namespace `{namespace}` was registered twice");
        }

        seen.push(namespace);
    }
}

#[cfg(test)]
mod tests {
    use super::{LuaApiRegistry, LuaCallContext, LuaModule};
    use crate::lua::LuaFunction;

    struct Marker;

    impl LuaModule for Marker {
        fn namespace(&self) -> &'static str {
            "marker"
        }

        fn functions(&self) -> &'static [LuaFunction] {
            &[]
        }

        fn register(&self, _: &mlua::Lua, _: &mlua::Table, _: &LuaCallContext) -> mlua::Result<()> {
            Ok(())
        }
    }

    struct Reserved;

    impl LuaModule for Reserved {
        fn namespace(&self) -> &'static str {
            "event"
        }

        fn functions(&self) -> &'static [LuaFunction] {
            &[]
        }

        fn register(&self, _: &mlua::Lua, _: &mlua::Table, _: &LuaCallContext) -> mlua::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn an_absent_optional_module_leaves_its_namespace_unregistered() {
        let registry = LuaApiRegistry::builder()
            .insert(Marker)
            .insert_optional(None::<Marker>)
            .build();

        assert_eq!(registry.namespaces(), vec!["marker"]);
    }

    #[test]
    #[should_panic(expected = "is reserved for workflow variables")]
    fn a_module_may_not_claim_a_reserved_namespace() {
        LuaApiRegistry::builder().insert(Reserved).build();
    }

    #[test]
    #[should_panic(expected = "was registered twice")]
    fn a_namespace_may_not_be_registered_twice() {
        LuaApiRegistry::builder()
            .insert(Marker)
            .insert(Marker)
            .build();
    }
}
