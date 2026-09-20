use async_graphql::SimpleObject;

use crate::lua::description::{ApiAlias, ApiClass, ApiDescription, ApiField, ApiFunction};

#[derive(SimpleObject)]
pub struct LuaApiObject {
    pub namespaces: Vec<LuaNamespaceObject>,
    pub globals: Vec<LuaFieldObject>,
    pub classes: Vec<LuaClassObject>,
    pub aliases: Vec<LuaAliasObject>,
    pub definitions: String,
}

#[derive(SimpleObject)]
pub struct LuaNamespaceObject {
    pub name: String,
    pub fields: Vec<LuaFieldObject>,
    pub functions: Vec<LuaFunctionObject>,
}

#[derive(SimpleObject)]
pub struct LuaFunctionObject {
    pub name: String,
    pub params: Vec<LuaFieldObject>,
    pub returns: Option<String>,
    pub scope: Option<String>,
    pub signature: String,
}

#[derive(SimpleObject)]
pub struct LuaFieldObject {
    pub name: String,
    pub r#type: String,
    pub optional: bool,
    pub declaration: String,
}

#[derive(SimpleObject)]
pub struct LuaClassObject {
    pub name: String,
    pub fields: Vec<LuaFieldObject>,
}

#[derive(SimpleObject)]
pub struct LuaAliasObject {
    pub name: String,
    pub r#type: String,
}

impl LuaApiObject {
    pub fn new(description: ApiDescription, definitions: String) -> Self {
        Self {
            namespaces: description
                .namespaces
                .into_iter()
                .map(|namespace| LuaNamespaceObject {
                    name: namespace.name,
                    fields: fields(namespace.fields),
                    functions: namespace.functions.into_iter().map(function).collect(),
                })
                .collect(),
            globals: fields(description.globals),
            classes: description
                .classes
                .into_iter()
                .map(|ApiClass { name, fields: own }| LuaClassObject {
                    name,
                    fields: fields(own),
                })
                .collect(),
            aliases: description
                .aliases
                .into_iter()
                .map(|ApiAlias { name, ty }| LuaAliasObject { name, r#type: ty })
                .collect(),
            definitions,
        }
    }
}

fn fields(fields: Vec<ApiField>) -> Vec<LuaFieldObject> {
    fields.into_iter().map(field).collect()
}

fn field(field: ApiField) -> LuaFieldObject {
    LuaFieldObject {
        declaration: field.declaration(),
        name: field.name,
        r#type: field.ty,
        optional: field.optional,
    }
}

fn function(function: ApiFunction) -> LuaFunctionObject {
    LuaFunctionObject {
        name: function.name,
        params: fields(function.params),
        returns: function.returns,
        scope: function.scope,
        signature: function.signature,
    }
}
