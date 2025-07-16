use crate::{
    actors::{event_handler, synergy::types::S3BucketEvent},
    types::{ApiState, AppError},
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};
use ractor::factory::JobOptions;

pub async fn synergy(
    State(ApiState {
        ref event_handler,
        ref settings,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(s3_bucket_event): Json<S3BucketEvent>,
) -> Result<StatusCode, AppError> {
    tracing::info!("{headers:?}");
    tracing::info!("{s3_bucket_event:?}");
    let synergy_secret = headers.get("Authorization");
    match synergy_secret {
        Some(secret_value)
            if *secret_value.to_str()?.replace("Bearer ", "") == settings.s3_webhook_secret =>
        {
            if let Err(e) = event_handler.send_message(ractor::factory::FactoryMessage::Dispatch(
                ractor::factory::Job {
                    key: (),
                    msg: event_handler::Message::SynergyDataIngest {
                        payload: s3_bucket_event,
                    },
                    options: JobOptions::default(),
                    accepted: None,
                },
            )) {
                tracing::error!("error forwarding synergy event {e}")
            };

            Ok(StatusCode::NO_CONTENT)
        }
        _ => Ok(StatusCode::UNAUTHORIZED),
    }
}
