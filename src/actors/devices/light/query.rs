use std::time::Duration;

use super::{LightHandler, LightHandlerMessage};
use crate::actors::system::rpc::{self, RpcError};

pub async fn query_light_on(ieee_addr: &str, timeout: Duration) -> Result<bool, RpcError> {
    rpc::query_factory(LightHandler::NAME, timeout, |reply| {
        LightHandlerMessage::QueryPowerState {
            ieee_addr: ieee_addr.to_owned(),
            traceparent: crate::telemetry::context::inject_current(),
            reply,
        }
    })
    .await
}
