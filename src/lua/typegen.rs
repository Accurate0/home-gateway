use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::signature::{LuaClass, LuaField, LuaNamespace, LuaType};

#[derive(Default)]
struct Definitions {
    aliases: BTreeMap<String, String>,
    classes: BTreeMap<&'static str, String>,
}

pub fn render(namespaces: &[LuaNamespace], globals: &[LuaField]) -> String {
    let mut definitions = Definitions::default();
    let mut body = String::new();

    for global in globals {
        let ty = definitions.lua_type(&global.ty);
        body.push_str(&format!("---@type {ty}\n{} = {{}}\n\n", global.name));
    }

    for namespace in namespaces {
        body.push_str(&format!("---@class gw.api.{}\n", namespace.name));

        for field in namespace.fields {
            let field = definitions.field(field.name, &field.ty);
            body.push_str(&format!("---@field {field}\n"));
        }

        body.push_str(&format!("{} = {{}}\n\n", namespace.name));

        for function in namespace.functions {
            if let Some(scope) = function.scope {
                body.push_str(&format!("---Requires the `{scope}` scope.\n"));
            }

            for param in function.params {
                let param = definitions.field(param.name, &param.ty);
                body.push_str(&format!("---@param {param}\n"));
            }

            if let Some(returns) = &function.returns {
                let returns = definitions.lua_type(returns);
                body.push_str(&format!("---@return {returns}\n"));
            }

            let params = function
                .params
                .iter()
                .map(|param| param.name)
                .collect::<Vec<_>>()
                .join(", ");

            body.push_str(&format!(
                "function {}.{}({params}) end\n\n",
                namespace.name, function.name
            ));
        }
    }

    let mut out = String::from("---@meta\n\n");

    for (name, rendered) in &definitions.aliases {
        out.push_str(&format!("---@alias gw.{name} {rendered}\n\n"));
    }

    for rendered in definitions.classes.values() {
        out.push_str(rendered);
        out.push('\n');
    }

    out.push_str(&body);

    format!("{}\n", out.trim_end())
}

impl Definitions {
    fn field(&mut self, name: &str, ty: &LuaType) -> String {
        match ty {
            LuaType::Optional(inner) => format!("{name}? {}", self.lua_type(inner)),
            other => format!("{name} {}", self.lua_type(other)),
        }
    }

    fn lua_type(&mut self, ty: &LuaType) -> String {
        match ty {
            LuaType::String => "string".to_owned(),
            LuaType::Integer => "integer".to_owned(),
            LuaType::Number => "number".to_owned(),
            LuaType::Boolean => "boolean".to_owned(),
            LuaType::Any => "any".to_owned(),
            LuaType::Table => "table".to_owned(),
            LuaType::Optional(inner) => format!("{}?", wrap(self.lua_type(inner))),
            LuaType::Array(inner) => format!("{}[]", wrap(self.lua_type(inner))),
            LuaType::Map(inner) => format!("table<string, {}>", self.lua_type(inner)),
            LuaType::Schema(build) => self.schema(build().as_value()),
            LuaType::Class(class) => self.class(class),
        }
    }

    fn class(&mut self, class: &'static LuaClass) -> String {
        if !self.classes.contains_key(class.name) {
            self.classes.insert(class.name, String::new());

            let mut rendered = format!("---@class gw.{}\n", class.name);
            for field in class.fields {
                let field = self.field(field.name, &field.ty);
                rendered.push_str(&format!("---@field {field}\n"));
            }

            self.classes.insert(class.name, rendered);
        }

        format!("gw.{}", class.name)
    }

    fn schema(&mut self, root: &Value) -> String {
        let name = root
            .get("title")
            .and_then(Value::as_str)
            .expect("a derived schema always carries its type name as a title");
        let defs = root.get("$defs").cloned().unwrap_or(Value::Null);

        self.alias(name, root, &defs)
    }

    fn alias(&mut self, name: &str, schema: &Value, defs: &Value) -> String {
        if !self.aliases.contains_key(name) {
            self.aliases.insert(name.to_owned(), "any".to_owned());

            let rendered = self.convert(schema, defs);
            self.aliases.insert(name.to_owned(), rendered);
        }

        format!("gw.{name}")
    }

    fn convert(&mut self, schema: &Value, defs: &Value) -> String {
        let Value::Object(map) = schema else {
            return "any".to_owned();
        };

        if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
            let name = reference.rsplit('/').next().unwrap_or(reference);
            let target = defs.get(name).cloned().unwrap_or(Value::Bool(true));

            return self.alias(name, &target, defs);
        }

        if let Some(constant) = map.get("const") {
            return literal(constant);
        }

        if let Some(Value::Array(values)) = map.get("enum") {
            return union(values.iter().map(literal));
        }

        for key in ["oneOf", "anyOf"] {
            if let Some(Value::Array(variants)) = map.get(key) {
                let variants: Vec<String> = variants
                    .iter()
                    .map(|variant| self.convert(variant, defs))
                    .collect();

                return union(variants);
            }
        }

        if let Some(Value::Array(parts)) = map.get("allOf")
            && let [single] = parts.as_slice()
        {
            return self.convert(single, defs);
        }

        match map.get("type") {
            Some(Value::String(ty)) => self.convert_typed(ty, map, defs),
            Some(Value::Array(types)) => {
                let types: Vec<String> = types
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|ty| self.convert_typed(ty, map, defs))
                    .collect();

                union(types)
            }
            _ => "any".to_owned(),
        }
    }

    fn convert_typed(&mut self, ty: &str, map: &Map<String, Value>, defs: &Value) -> String {
        match ty {
            "string" => "string".to_owned(),
            "integer" => "integer".to_owned(),
            "number" => "number".to_owned(),
            "boolean" => "boolean".to_owned(),
            "null" => "nil".to_owned(),
            "array" => {
                let items = match map.get("items") {
                    Some(items) => self.convert(items, defs),
                    None => "any".to_owned(),
                };

                format!("{}[]", wrap(items))
            }
            "object" => {
                if let Some(Value::Object(properties)) = map.get("properties") {
                    let required: BTreeSet<&str> = map
                        .get("required")
                        .and_then(Value::as_array)
                        .map(|required| required.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();

                    let fields = properties
                        .iter()
                        .map(|(name, property)| {
                            let marker = if required.contains(name.as_str()) {
                                ""
                            } else {
                                "?"
                            };

                            format!("{name}{marker}: {}", self.convert(property, defs))
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    return format!("{{ {fields} }}");
                }

                match map.get("additionalProperties") {
                    Some(additional @ Value::Object(_)) => {
                        format!("table<string, {}>", self.convert(additional, defs))
                    }
                    _ => "table".to_owned(),
                }
            }
            _ => "any".to_owned(),
        }
    }
}

fn literal(value: &Value) -> String {
    match value {
        Value::String(text) => format!("{text:?}"),
        other => other.to_string(),
    }
}

fn union(parts: impl IntoIterator<Item = String>) -> String {
    let mut seen = BTreeSet::new();

    parts
        .into_iter()
        .filter(|part| seen.insert(part.clone()))
        .collect::<Vec<_>>()
        .join("|")
}

fn wrap(ty: String) -> String {
    if ty.contains('|') || ty.ends_with('?') {
        format!("({ty})")
    } else {
        ty
    }
}

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;

    use crate::auth::scope::{Action, Resource, Scope};
    use crate::lua::signature::{
        LuaClass, LuaField, LuaFunction, LuaNamespace, LuaParam, LuaType, schema,
    };

    use super::render;

    #[derive(JsonSchema)]
    #[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
    #[allow(dead_code)]
    enum Command {
        On,
        Dim { value: u64 },
    }

    const READING: LuaClass = LuaClass {
        name: "Reading",
        fields: &[LuaField {
            name: "lux",
            ty: LuaType::Optional(&LuaType::Number),
        }],
    };

    const FUNCTIONS: &[LuaFunction] = &[
        LuaFunction {
            name: "set",
            params: &[
                LuaParam {
                    name: "device",
                    ty: LuaType::String,
                },
                LuaParam {
                    name: "command",
                    ty: LuaType::Schema(schema::<Command>),
                },
            ],
            returns: None,
            scope: Some(Scope::new(Resource::Light, Action::Write)),
        },
        LuaFunction {
            name: "reading",
            params: &[LuaParam {
                name: "retain",
                ty: LuaType::Optional(&LuaType::Boolean),
            }],
            returns: Some(LuaType::Optional(&LuaType::Class(&READING))),
            scope: None,
        },
    ];

    #[test]
    fn a_namespace_renders_as_luals_definitions() {
        let rendered = render(
            &[LuaNamespace {
                name: "demo",
                fields: &[],
                functions: FUNCTIONS,
            }],
            &[LuaField {
                name: "event",
                ty: LuaType::Map(&LuaType::Any),
            }],
        );

        assert_eq!(
            rendered,
            "---@meta

---@alias gw.Command { state: \"ON\" }|{ value: integer, state: \"DIM\" }

---@class gw.Reading
---@field lux? number

---@type table<string, any>
event = {}

---@class gw.api.demo
demo = {}

---Requires the `light:write` scope.
---@param device string
---@param command gw.Command
function demo.set(device, command) end

---@param retain? boolean
---@return gw.Reading?
function demo.reading(retain) end
"
        );
    }
}
