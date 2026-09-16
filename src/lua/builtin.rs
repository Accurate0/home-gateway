use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;

use async_graphql::Variables;
use async_graphql::parser::parse_query;
use async_graphql::parser::types::{DocumentOperations, OperationType};

use crate::auth::AuthContext;
use crate::auth::scope::{Action, Resource, Scope};
use crate::event_bus::{CustomEventSource, EventBusMessage};
use crate::http::public_client::PublicHttpClient;
use crate::lua::bridge::lua_to_value;
use crate::variables::Node;
use mlua::{ExternalError, ExternalResult, Lua, LuaSerdeExt, Table, Value as LuaValue};

use super::signature::{LuaClass, LuaField, LuaFunction, LuaParam, LuaType};
use super::{LuaAuthority, LuaCallContext};

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

const COOLDOWN: LuaFunction = LuaFunction {
    name: "cooldown",
    params: &[
        LuaParam {
            name: "key",
            ty: LuaType::String,
        },
        LuaParam {
            name: "seconds",
            ty: LuaType::Integer,
        },
    ],
    returns: Some(LuaType::Boolean),
    scope: Some(Scope::new(Resource::Workflow, Action::Write)),
};

const EMIT: LuaFunction = LuaFunction {
    name: "emit",
    params: &[
        LuaParam {
            name: "name",
            ty: LuaType::String,
        },
        LuaParam {
            name: "payload",
            ty: LuaType::Optional(&LuaType::Map(&LuaType::Any)),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Workflow, Action::Run)),
};

const GRAPHQL: LuaFunction = LuaFunction {
    name: "graphql",
    params: &[
        LuaParam {
            name: "query",
            ty: LuaType::String,
        },
        LuaParam {
            name: "variables",
            ty: LuaType::Optional(&LuaType::Map(&LuaType::Any)),
        },
    ],
    returns: Some(LuaType::Any),
    scope: None,
};

pub const GW_FUNCTIONS: &[LuaFunction] =
    &[LOG, SLEEP, HAS, REQUIRE, HTTP, GRAPHQL, LIB, COOLDOWN, EMIT];

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
    library: &Arc<BTreeMap<String, Vec<u8>>>,
) -> mlua::Result<()> {
    let globals = lua.globals();

    globals.set("gw", gateway_table(lua, cx, library)?)?;
    globals.set("json", json_table(lua, cx)?)?;

    Ok(())
}

fn gateway_table(
    lua: &Lua,
    cx: &LuaCallContext,
    library: &Arc<BTreeMap<String, Vec<u8>>>,
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

    let graphql_cx = cx.clone();
    cx.expose(&table, &GRAPHQL, || {
        lua.create_async_function(move |lua, (query, variables): (String, Option<Table>)| {
            let cx = graphql_cx.clone();

            async move { graphql(&lua, &cx, query, variables).await }
        })
    })?;

    let cooldown_cx = cx.clone();
    cx.expose(&table, &COOLDOWN, || {
        lua.create_async_function(move |_, (key, seconds): (String, i64)| {
            let cx = cooldown_cx.clone();

            async move { cooldown(&cx, &key, seconds).await }
        })
    })?;

    let emit_cx = cx.clone();
    cx.expose(&table, &EMIT, || {
        lua.create_async_function(move |_, (name, payload): (String, Option<Table>)| {
            let cx = emit_cx.clone();

            async move { emit(&cx, name, payload).await }
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

            super::bytecode::load(lua, name.as_str(), source)?.call::<LuaValue>(())
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

    let client = cx.state.handles.expect::<PublicHttpClient>();
    let mut outgoing = client.request(method.clone(), &url).into_lua_err()?;

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

async fn graphql(
    lua: &Lua,
    cx: &LuaCallContext,
    query: String,
    variables: Option<Table>,
) -> mlua::Result<LuaValue> {
    reject_non_queries(&query)?;

    let variables = match variables {
        Some(table) => {
            let raw: serde_json::Value = lua.from_value(LuaValue::Table(table))?;

            Variables::from_json(raw)
        }
        None => Variables::default(),
    };

    let auth = match &cx.authority {
        LuaAuthority::Trusted => AuthContext::full_access(false),
        LuaAuthority::Delegated(auth) => auth.as_ref().clone(),
    };

    let request = async_graphql::Request::new(query)
        .variables(variables)
        .data(auth)
        .data(cx.state.clone());

    let response = cx
        .query("gw.graphql", || async {
            Ok::<_, Infallible>(cx.state.schema.execute(request).await)
        })
        .await?;

    if !response.errors.is_empty() {
        let joined = response
            .errors
            .iter()
            .map(|error| error.message.clone())
            .collect::<Vec<_>>()
            .join("; ");

        tracing::warn!("[{}] lua graphql failed: {joined}", cx.event_id);

        return Err(joined.into_lua_err());
    }

    lua.to_value(&response.data)
}

fn reject_non_queries(query: &str) -> mlua::Result<()> {
    let document = parse_query(query).map_err(|error| error.to_string().into_lua_err())?;

    let operations = match &document.operations {
        DocumentOperations::Single(operation) => vec![&operation.node],
        DocumentOperations::Multiple(operations) => operations
            .values()
            .map(|operation| &operation.node)
            .collect(),
    };

    for operation in operations {
        if operation.ty != OperationType::Query {
            return Err(format!(
                "gw.graphql only runs queries, got a {}",
                match operation.ty {
                    OperationType::Mutation => "mutation",
                    OperationType::Subscription => "subscription",
                    OperationType::Query => "query",
                }
            )
            .into_lua_err());
        }
    }

    Ok(())
}

async fn emit(cx: &LuaCallContext, name: String, payload: Option<Table>) -> mlua::Result<()> {
    let mut node = Node::empty();

    if let Some(payload) = payload {
        for pair in payload.pairs::<String, LuaValue>() {
            let (key, value) = pair?;

            let value = lua_to_value(&value).ok_or_else(|| {
                format!("payload `{key}` must be a string, number or boolean").into_lua_err()
            })?;

            node.insert(key, Node::Value(Some(value)));
        }
    }

    let message = EventBusMessage::Custom {
        event_id: cx.event_id,
        source: CustomEventSource::Lua,
        name: name.clone(),
        payload: node,
    };

    let bus = &cx.state.event_bus;

    cx.command("gw.emit", &name, move || async move {
        bus.publish(message);

        Ok::<_, Infallible>(())
    })
    .await
}

async fn cooldown(cx: &LuaCallContext, key: &str, seconds: i64) -> mlua::Result<bool> {
    let name = format!("lua:{key}");
    let window = TimeDelta::seconds(seconds.max(0));
    let repo = cx.state.repos.workflow();

    if cx.dry_run {
        let active = cx
            .query("gw.cooldown", || async {
                repo.cooldown_active(&name, window).await
            })
            .await?;

        tracing::info!(
            "[{}] dry-run lua cooldown {name} would {}",
            cx.event_id,
            if active { "block" } else { "pass" }
        );

        return Ok(!active);
    }

    let passed = cx
        .query("gw.cooldown", || async {
            repo.cooldown_ok(&name, window).await
        })
        .await?;

    tracing::info!(
        "[{}] lua cooldown {name} {}",
        cx.event_id,
        if passed { "passed" } else { "blocked" }
    );

    Ok(passed)
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
