use crate::{
    actors::{event_handler, maccas::types::MaccasOfferIngest},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};
use ractor::factory::JobOptions;

pub async fn maccas(
    State(ApiState {
        ref event_handler,
        ref settings,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(maccas_offer): Json<MaccasOfferIngest>,
) -> StatusCode {
    let maccas_secret_header = headers.get("X-Maccas-External-Secret");
    match maccas_secret_header {
        Some(secret_value) if *secret_value == settings.maccas.webhook_secret => {
            if let Err(e) = event_handler.send_message(ractor::factory::FactoryMessage::Dispatch(
                ractor::factory::Job {
                    key: (),
                    msg: event_handler::Message::MaccasOfferIngest {
                        payload: maccas_offer,
                    },
                    options: JobOptions::default(),
                    accepted: None,
                },
            )) {
                tracing::error!("error forwarding maccas event {e}")
            };

            StatusCode::NO_CONTENT
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}
