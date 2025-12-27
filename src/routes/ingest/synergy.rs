use crate::{
    actors::event_handler,
    types::{ApiState, AppError},
};
use axum::extract::State;
use bytes::Bytes;
use http::StatusCode;
use ractor::factory::JobOptions;

pub async fn synergy(
    State(ApiState {
        ref event_handler, ..
    }): State<ApiState>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    match event_handler.send_message(ractor::factory::FactoryMessage::Dispatch(
        ractor::factory::Job {
            key: (),
            msg: event_handler::Message::SynergyDataIngest { payload: body },
            options: JobOptions::default(),
            accepted: None,
        },
    )) {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            tracing::error!("error forwarding synergy event {e}");
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
