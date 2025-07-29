use crate::{
    actors::{alarm::types::AndroidAppAlarmPayload, event_handler},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};
use ractor::factory::JobOptions;

pub async fn alarm(
    State(ApiState {
        ref event_handler,
        ref settings,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<AndroidAppAlarmPayload>,
) -> StatusCode {
    let secret_header = headers.get("X-Webhook-Secret");
    match secret_header {
        Some(secret_value) if *secret_value == settings.android_app_webhook_secret => {
            if let Err(e) = event_handler.send_message(ractor::factory::FactoryMessage::Dispatch(
                ractor::factory::Job {
                    key: (),
                    msg: event_handler::Message::AlarmChangeIngest { payload },
                    options: JobOptions::default(),
                    accepted: None,
                },
            )) {
                tracing::error!("error forwarding alarm event {e}")
            };

            StatusCode::NO_CONTENT
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}
