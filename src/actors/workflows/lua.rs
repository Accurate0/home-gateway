use std::collections::BTreeMap;
use std::time::Duration;

use chrono::Utc;
use mlua::{ExternalError, Function, MultiValue, Value as LuaValue};

use crate::lua::bridge::lua_to_value;
use crate::lua::{Json, LuaCallContext, lua_module};
use crate::workflows::definition::EnableState;
use crate::workflows::mode::Mode;
use crate::workflows::trace::{StepOutcome, StepTrace};

use super::{ReusableCall, WorkflowWorker};
use crate::workflows::manager::WorkflowManager;

pub struct WorkflowLua;

#[lua_module(namespace = "workflow")]
impl WorkflowLua {
    #[lua(scope = Workflow::Run)]
    async fn run(
        cx: &LuaCallContext,
        name: String,
        with: Option<BTreeMap<String, LuaValue>>,
    ) -> mlua::Result<()> {
        let mut inputs = BTreeMap::new();

        for (key, value) in with.unwrap_or_default() {
            let value = lua_to_value(&value).ok_or_else(|| {
                format!("input `{key}` must be a string, number or boolean").into_lua_err()
            })?;

            inputs.insert(key, value);
        }

        let worker = WorkflowWorker::new(cx.state.clone());
        let origin = cx.origin.clone();

        let call = ReusableCall {
            event_id: cx.event_id,
            depth: cx.depth + 1,
            dry_run: cx.dry_run,
            origin_slug: &origin,
            trace: &cx.trace,
        };

        cx.command(Self::RUN, &name, || async {
            worker.run_reusable(call, &name, inputs).await
        })
        .await
    }

    #[lua(scope = Workflow::Read)]
    async fn mode(cx: &LuaCallContext) -> mlua::Result<Json<Mode>> {
        let mode = cx
            .state
            .handles
            .expect::<WorkflowManager>()
            .current_mode()
            .await;

        Ok(Json(mode))
    }

    #[lua(scope = Workflow::Write)]
    async fn set_mode(cx: &LuaCallContext, mode: Json<Mode>) -> mlua::Result<()> {
        let mode = mode.0;
        let worker = WorkflowWorker::new(cx.state.clone());

        cx.command(Self::SET_MODE, mode.as_str(), || async {
            worker.run_set_mode(mode).await
        })
        .await
    }

    #[lua(scope = Workflow::Write)]
    async fn set_enabled(
        cx: &LuaCallContext,
        tag: String,
        state: Json<EnableState>,
    ) -> mlua::Result<()> {
        let state = state.0;
        let worker = WorkflowWorker::new(cx.state.clone());
        let origin = cx.origin.clone();

        cx.command(
            Self::SET_ENABLED,
            format!("#{tag} -> {state:?}"),
            || async {
                worker
                    .run_set_workflows_enabled(cx.event_id, &origin, &tag, state)
                    .await
            },
        )
        .await
    }

    #[lua(scope = Workflow::Read)]
    fn record_step(
        cx: &LuaCallContext,
        kind: String,
        detail: Option<String>,
        error: Option<String>,
    ) -> mlua::Result<()> {
        let outcome = if error.is_some() {
            StepOutcome::Error
        } else {
            StepOutcome::Ran
        };

        cx.trace.record(StepTrace {
            depth: cx.trace_depth(),
            kind,
            outcome,
            guard: None,
            detail,
            error,
            duration: Duration::ZERO,
            at: Utc::now(),
        });

        Ok(())
    }

    #[lua(scope = Workflow::Read)]
    async fn step(
        cx: &LuaCallContext,
        kind: String,
        run: Function,
        detail: Option<String>,
    ) -> mlua::Result<MultiValue> {
        let handle = cx.trace.start(cx.trace_depth(), kind, detail);

        let result = {
            let _nested = cx.trace.nest();

            run.call_async::<MultiValue>(()).await
        };

        cx.trace
            .finish_with(handle, result.as_ref().err().map(ToString::to_string));

        result
    }
}
