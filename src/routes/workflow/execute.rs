use crate::{
    actors::system::rpc,
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    settings::{ReusableWorkflow, workflow::scope as workflow_scope},
    state::AppState,
    variables::{Node, Vars, input::input_node},
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
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
    Auth(auth): Auth,
    Json(payload): Json<WorkflowExecutePayload>,
) -> Result<Response, AppError> {
    auth.require(&Scope::new(Resource::Workflow, Action::Write))
        .map_err(AppError::StatusCode)?;

    let WorkflowExecutePayload { workflow, inputs } = payload;

    let input = match validate(&state, &workflow, &inputs) {
        Ok(input) => input,
        Err(error) => {
            tracing::warn!("rejected ad-hoc workflow: {error}");
            return Ok((StatusCode::BAD_REQUEST, error).into_response());
        }
    };

    let message = WorkflowWorkerMessage::Execute {
        event_id: uuid::Uuid::new_v4(),
        workflow,
        vars: Vars::default().with("input", input),
        traceparent: crate::tracing_context::inject_current(),
    };

    if let Err(e) = rpc::cast_factory(WorkflowWorker::NAME, message) {
        tracing::error!("error dispatching workflow: {e}");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    Ok(StatusCode::NO_CONTENT.into_response())
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
