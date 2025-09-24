use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    settings::workflow::WorkflowSettings,
    types::{ApiState, AppError},
};
use axum::{Json, extract::State};
use http::StatusCode;
use ractor::factory::{FactoryMessage, Job, JobOptions};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WorkflowExecutePayload {
    pub workflow: WorkflowSettings,
}

pub async fn workflow_execute(
    State(ApiState { .. }): State<ApiState>,
    Json(payload): Json<WorkflowExecutePayload>,
) -> Result<StatusCode, AppError> {
    let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
        tracing::warn!("could not find workflow actor");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let message = WorkflowWorkerMessage::Execute {
        event_id: uuid::Uuid::new_v4(),
        workflow: payload.workflow,
    };

    let message = FactoryMessage::Dispatch(Job {
        key: (),
        msg: message,
        options: JobOptions::default(),
        accepted: None,
    });

    actor.send_message(message)?;

    Ok(StatusCode::NO_CONTENT)
}
