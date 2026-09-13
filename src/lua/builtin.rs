use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use mlua::{ExternalError, ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};
use reqwest_middleware::ClientWithMiddleware;

use crate::auth::scope::{Action, Resource, Scope};

use super::LuaCallContext;
use super::signature::{LuaClass, LuaField, LuaFunction, LuaParam, LuaType};

pub const GW_FIELDS: &[LuaField] = &[
    LuaField {
        name: "event_id",
        ty: LuaType::String,
    },
    LuaField {
        name: "origin",
        ty: LuaType::String,
    },
    LuaField {
        name: "dry_run",
        ty: LuaType::Boolean,
    },
];

const HTTP_REQUEST: LuaClass = LuaClass {
    name: "HttpRequest",
    fields: &[
        LuaField {
            name: "url",
            ty: LuaType::String,
        },
        LuaField {
            name: "method",
            ty: LuaType::Optional(&LuaType::String),
        },
        LuaField {
            name: "headers",
            ty: LuaType::Optional(&LuaType::Map(&LuaType::String)),
        },
        LuaField {
            name: "body",
            ty: LuaType::Optional(&LuaType::String),
        },
    ],
};

const HTTP_RESPONSE: LuaClass = LuaClass {
    name: "HttpResponse",
    fields: &[
        LuaField {
            name: "status",
            ty: LuaType::Integer,
        },
        LuaField {
            name: "ok",
            ty: LuaType::Boolean,
        },
        LuaField {
            name: "body",
            ty: LuaType::String,
        },
    ],
};

const LOG: LuaFunction = LuaFunction {
    name: "log",
    params: &[LuaParam {
        name: "message",
        ty: LuaType::String,
    }],
    returns: None,
    scope: None,
};

const SLEEP: LuaFunction = LuaFunction {
    name: "sleep",
    params: &[LuaParam {
        name: "seconds",
        ty: LuaType::Number,
    }],
    returns: None,
    scope: None,
};

const HAS: LuaFunction = LuaFunction {
    name: "has",
    params: &[LuaParam {
        name: "scope",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Boolean),
    scope: None,
};

const REQUIRE: LuaFunction = LuaFunction {
    name: "require",
    params: &[LuaParam {
        name: "scope",
        ty: LuaType::String,
    }],
    returns: None,
    scope: None,
};

const HTTP: LuaFunction = LuaFunction {
    name: "http",
    params: &[LuaParam {
        name: "request",
        ty: LuaType::Class(&HTTP_REQUEST),
    }],
    returns: Some(LuaType::Class(&HTTP_RESPONSE)),
    scope: Some(Scope::new(Resource::Http, Action::Write)),
};

const LIB: LuaFunction = LuaFunction {
    name: "lib",
    params: &[LuaParam {
        name: "name",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Any),
    scope: None,
};

pub const GW_FUNCTIONS: &[LuaFunction] = &[LOG, SLEEP, HAS, REQUIRE, HTTP, LIB];

const DECODE: LuaFunction = LuaFunction {
    name: "decode",
    params: &[LuaParam {
        name: "raw",
        ty: LuaType::String,
    }],
    returns: Some(LuaType::Any),
    scope: None,
};

const ENCODE: LuaFunction = LuaFunction {
    name: "encode",
    params: &[LuaParam {
        name: "value",
        ty: LuaType::Any,
    }],
    returns: Some(LuaType::String),
    scope: None,
};

pub const JSON_FUNCTIONS: &[LuaFunction] = &[DECODE, ENCODE];

pub fn install(
    lua: &Lua,
    cx: &LuaCallContext,
    library: &Arc<BTreeMap<String, String>>,
) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("gw", gateway_table(lua, cx, library)?)?;
    globals.set("json", json_table(lua, cx)?)?;

    Ok(())
}

fn gateway_table(
    lua: &Lua,
    cx: &LuaCallContext,
    library: &Arc<BTreeMap<String, String>>,
) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set("event_id", cx.event_id.to_string())?;
    table.set("origin", cx.origin.clone())?;
    table.set("dry_run", cx.dry_run)?;

    let log_id = cx.event_id;
    cx.expose(&table, &LOG, || {
        lua.create_function(move |_, message: String| {
            tracing::info!("[{log_id}] lua: {message}");

            Ok(())
        })
    })?;

    cx.expose(&table, &SLEEP, || {
        lua.create_async_function(|_, seconds: f64| async move {
            tokio::time::sleep(Duration::from_secs_f64(seconds.max(0.0))).await;

            Ok(())
        })
    })?;

    let has_cx = cx.clone();
    cx.expose(&table, &HAS, || {
        lua.create_function(move |_, raw: String| {
            let scope = Scope::parse(&raw).into_lua_err()?;

            Ok(has_cx.allows(scope.resource, scope.action))
        })
    })?;

    let require_cx = cx.clone();
    cx.expose(&table, &REQUIRE, || {
        lua.create_function(move |_, raw: String| {
            let scope = Scope::parse(&raw).into_lua_err()?;

            if require_cx.allows(scope.resource, scope.action) {
                return Ok(());
            }

            tracing::info!(
                "[{}] lua scope check denied {} for {}",
                require_cx.event_id,
                scope,
                require_cx.authority.describe()
            );

            Err(format!("missing scope `{scope}`").into_lua_err())
        })
    })?;

    let http_cx = cx.clone();
    cx.expose(&table, &HTTP, || {
        lua.create_async_function(move |lua, request: Table| {
            let cx = http_cx.clone();

            async move { http(&lua, &cx, request).await }
        })
    })?;

    let library = library.clone();
    cx.expose(&table, &LIB, || {
        lua.create_function(move |lua, name: String| {
            let source = library.get(&name).ok_or_else(|| {
                format!(
                    "unknown lua library `{name}`; available: [{}]",
                    library.keys().cloned().collect::<Vec<_>>().join(", ")
                )
                .into_lua_err()
            })?;

            lua.load(source.as_str())
                .set_name(name.as_str())
                .eval::<LuaValue>()
        })
    })?;

    Ok(table)
}

async fn http(lua: &Lua, cx: &LuaCallContext, request: Table) -> mlua::Result<Table> {
    let url: String = request.get("url")?;
    let method: Option<String> = request.get("method")?;
    let body: Option<String> = request.get("body")?;
    let headers: Option<Table> = request.get("headers")?;

    let method = method.unwrap_or_else(|| "GET".to_owned());
    let method = reqwest::Method::from_bytes(method.to_uppercase().as_bytes()).into_lua_err()?;

    let client = cx.state.handles.expect::<ClientWithMiddleware>();
    let mut outgoing = client.request(method.clone(), &url);

    if let Some(headers) = headers {
        for pair in headers.pairs::<String, String>() {
            let (name, value) = pair?;
            outgoing = outgoing.header(name, value);
        }
    }

    if let Some(body) = body {
        outgoing = outgoing.body(body);
    }

    let detail = format!("{method} {url}");
    let response = cx
        .query("gw.http", || async { outgoing.send().await })
        .await?;

    let status = response.status();
    let text = response.text().await.into_lua_err()?;

    tracing::info!("[{}] lua http {detail} returned {status}", cx.event_id);

    let result = lua.create_table()?;
    result.set("status", status.as_u16())?;
    result.set("ok", status.is_success())?;
    result.set("body", text)?;

    Ok(result)
}

fn json_table(lua: &Lua, cx: &LuaCallContext) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    cx.expose(&table, &DECODE, || {
        lua.create_function(|lua, raw: String| {
            let parsed: serde_json::Value = serde_json::from_str(&raw).into_lua_err()?;

            lua.to_value(&parsed)
        })
    })?;

    cx.expose(&table, &ENCODE, || {
        lua.create_function(|lua, value: LuaValue| {
            let parsed: serde_json::Value = lua.from_value(value)?;

            serde_json::to_string(&parsed).into_lua_err()
        })
    })?;

    Ok(table)
}
