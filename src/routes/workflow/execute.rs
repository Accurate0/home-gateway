use crate::{
    actors::system::rpc,
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
    lua::LuaAuthority,
    settings::{ReusableWorkflow, workflow::scope as workflow_scope},
    state::AppState,
    variables::{Node, Vars, input::input_node},
};
use anyhow::Context;
use axum::{Json, extract::State};
use http::StatusCode;

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct WorkflowExecutePayload {
    pub workflow: ReusableWorkflow,
    pub inputs: BTreeMap<String, serde_json::Value>,
}

pub async fn workflow_execute(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<WorkflowExecutePayload>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::Workflow, Action::Write)?;

    let WorkflowExecutePayload { workflow, inputs } = payload;

    let input = match validate(&state, &workflow, &inputs) {
        Ok(input) => input,
        Err(error) => {
            tracing::warn!("rejected ad-hoc workflow: {error}");
            return Err(AppError::bad_request(error));
        }
    };

    let missing = workflow.steps().into_iter().find_map(|step| {
        step.scope()
            .filter(|scope| !auth.has(scope))
            .map(|scope| format!("step `{}` needs scope `{scope}`", step.kind()))
    });

    if let Some(error) = missing {
        tracing::warn!("rejected ad-hoc workflow: {error}");
        return Err(AppError::forbidden(error));
    }

    let message = WorkflowWorkerMessage::Execute {
        event_id: uuid::Uuid::new_v4(),
        workflow,
        vars: Vars::default().with("input", input),
        authority: LuaAuthority::delegated(auth),
        traceparent: crate::tracing_context::inject_current(),
    };

    rpc::cast_factory(WorkflowWorker::NAME, message).context("dispatching workflow")?;

    Ok(StatusCode::NO_CONTENT)
}

fn validate(
    state: &AppState,
    workflow: &ReusableWorkflow,
    inputs: &BTreeMap<String, serde_json::Value>,
) -> Result<Node, String> {
    let scope = workflow_scope::reusable_scope(workflow)?;

    workflow_scope::check_steps(workflow, &scope)?;
    workflow_scope::check_calls(workflow, &scope, &state.settings.workflows)?;

    input_node(&workflow.inputs.clone().unwrap_or_default(), inputs)
}
