use std::collections::BTreeMap;

use mlua::{ExternalError, Lua, LuaSerdeExt, Table, Value as LuaValue};

use crate::auth::scope::{Action, Resource, Scope};
use crate::lua::bridge::lua_to_value;
use crate::lua::{LuaCallContext, LuaFunction, LuaModule, LuaParam, LuaType, schema};
use crate::mode::Mode;
use crate::variables::Vars;

use super::manager::WorkflowManager;
use super::{ReusableCall, WorkflowWorker};

const RUN: LuaFunction = LuaFunction {
    name: "run",
    params: &[
        LuaParam {
            name: "name",
            ty: LuaType::String,
        },
        LuaParam {
            name: "with",
            ty: LuaType::Optional(&LuaType::Map(&LuaType::Any)),
        },
    ],
    returns: None,
    scope: Some(Scope::new(Resource::Workflow, Action::Run)),
};

const MODE: LuaFunction = LuaFunction {
    name: "mode",
    params: &[],
    returns: Some(LuaType::Schema(schema::<Mode>)),
    scope: Some(Scope::new(Resource::Workflow, Action::Read)),
};

const SET_MODE: LuaFunction = LuaFunction {
    name: "set_mode",
    params: &[LuaParam {
        name: "mode",
        ty: LuaType::Schema(schema::<Mode>),
    }],
    returns: None,
    scope: Some(Scope::new(Resource::Workflow, Action::Write)),
};

const FUNCTIONS: &[LuaFunction] = &[RUN, MODE, SET_MODE];

pub struct WorkflowLua;

impl LuaModule for WorkflowLua {
    fn namespace(&self) -> &'static str {
        "workflow"
    }

    fn functions(&self) -> &'static [LuaFunction] {
        FUNCTIONS
    }

    fn register(&self, lua: &Lua, table: &Table, cx: &LuaCallContext) -> mlua::Result<()> {
        let run_cx = cx.clone();
        cx.expose(table, &RUN, || {
            lua.create_async_function(move |_, (name, with): (String, Option<Table>)| {
                let cx = run_cx.clone();

                async move {
                    let mut inputs = BTreeMap::new();

                    if let Some(with) = with {
                        for pair in with.pairs::<String, LuaValue>() {
                            let (key, value) = pair?;

                            let value = lua_to_value(&value).ok_or_else(|| {
                                format!("input `{key}` must be a string, number or boolean")
                                    .into_lua_err()
                            })?;

                            inputs.insert(key, value);
                        }
                    }

                    let worker = WorkflowWorker::new(cx.state.clone());
                    let origin = cx.origin.clone();

                    let call = ReusableCall {
                        event_id: cx.event_id,
                        depth: cx.depth + 1,
                        dry_run: cx.dry_run,
                        origin_slug: &origin,
                    };

                    cx.command("workflow.run", &name, || async {
                        worker
                            .run_reusable(call, &name, &Vars::default(), inputs)
                            .await
                    })
                    .await
                }
            })
        })?;

        let mode_cx = cx.clone();
        cx.expose(table, &MODE, || {
            lua.create_async_function(move |_, ()| {
                let cx = mode_cx.clone();

                async move {
                    let mode = cx
                        .state
                        .handles
                        .expect::<WorkflowManager>()
                        .current_mode()
                        .await;

                    Ok(mode.as_str().to_owned())
                }
            })
        })?;

        let set_mode_cx = cx.clone();
        cx.expose(table, &SET_MODE, || {
            lua.create_async_function(move |lua, mode: LuaValue| {
                let cx = set_mode_cx.clone();

                async move {
                    let mode: Mode = lua.from_value(mode)?;
                    let worker = WorkflowWorker::new(cx.state.clone());

                    cx.command("workflow.set_mode", mode.as_str(), || async {
                        worker.run_set_mode(mode).await
                    })
                    .await
                }
            })
        })
    }
}
