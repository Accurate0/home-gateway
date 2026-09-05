use crate::{
    actors::system::rpc,
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    error::AppError,
    settings::workflow::Workflow,
    state::ApiState,
};
use axum::{Json, extract::State};
use http::StatusCode;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct WorkflowExecutePayload {
    pub workflow: Workflow,
}

pub async fn workflow_execute(
    State(ApiState { .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(payload): Json<WorkflowExecutePayload>,
) -> Result<StatusCode, AppError> {
    auth.require(&Scope::new(Resource::Workflow, Action::Write))
        .map_err(AppError::StatusCode)?;

    let message = WorkflowWorkerMessage::Execute {
        event_id: uuid::Uuid::new_v4(),
        workflow: payload.workflow,
        vars: HashMap::new(),
    };

    if let Err(e) = rpc::cast_factory(WorkflowWorker::NAME, message) {
        tracing::error!("error dispatching workflow: {e}");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(StatusCode::NO_CONTENT)
}
