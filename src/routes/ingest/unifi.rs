use crate::{
    actors::{event_handler, unifi::types::UnifiWebhookEvents},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};
use ractor::factory::JobOptions;

pub async fn unifi(
    State(ApiState {
        ref event_handler,
        ref settings,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(unifi_event): Json<UnifiWebhookEvents>,
) -> StatusCode {
    let unifi_secret_header = headers.get("X-Webhook-Secret");
    match unifi_secret_header {
        Some(secret_value) if *secret_value == settings.unifi_webhook_secret => {
            if let Err(e) = event_handler.send_message(ractor::factory::FactoryMessage::Dispatch(
                ractor::factory::Job {
                    key: (),
                    msg: event_handler::Message::UnifiWebhook {
                        payload: unifi_event,
                    },
                    options: JobOptions::default(),
                    accepted: None,
                },
            )) {
                tracing::error!("error forwarding unifi event {e}")
            };

            StatusCode::NO_CONTENT
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}
