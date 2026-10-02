pub mod eink_display;
pub mod proto;

use axum::Router;
use tonic::service::Routes;
use tonic_web::GrpcWebLayer;
use tower::Layer;

use crate::state::AppState;
use eink_display::EinkDisplayService;
use proto::eink_display_server::EinkDisplayServer;

pub const PREFIX: &str = "/grpc";

pub fn router(state: AppState) -> Router {
    let eink_display = EinkDisplayServer::new(EinkDisplayService::new(state));

    Routes::new(GrpcWebLayer::new().layer(eink_display)).into_axum_router()
}
