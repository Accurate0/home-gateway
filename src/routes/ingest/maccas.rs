use crate::{actors::event_handler, maccas::MaccasOfferIngest, types::ApiState};
use axum::{Json, extract::State};
use http::StatusCode;
use ractor::factory::JobOptions;

pub async fn maccas(
    State(ApiState {
        ref event_handler, ..
    }): State<ApiState>,
    Json(maccas_offer): Json<MaccasOfferIngest>,
) -> StatusCode {
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
