use std::time::Duration;

use super::{Message as PresenceMessage, PresenceSensorHandler};
use crate::actors::system::rpc::{self, RpcError};

pub async fn query_presence(sensor: &str, timeout: Duration) -> Result<bool, RpcError> {
    let present: Option<bool> = rpc::query_factory(PresenceSensorHandler::NAME, timeout, |reply| {
        PresenceMessage::QueryLatest {
            sensor: sensor.to_owned(),
            traceparent: crate::telemetry::context::inject_current(),
            reply,
        }
    })
    .await?;

    match present {
        Some(present) => Ok(present),
        None => {
            tracing::warn!("no presence reading for sensor {sensor}");
            Ok(false)
        }
    }
}
