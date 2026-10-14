use std::time::Duration;

use chrono::TimeDelta;

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

pub async fn query_presence_seen_within(
    sensor: &str,
    window: TimeDelta,
    timeout: Duration,
) -> Result<bool, RpcError> {
    let seen: Option<bool> = rpc::query_factory(PresenceSensorHandler::NAME, timeout, |reply| {
        PresenceMessage::QuerySeenWithin {
            sensor: sensor.to_owned(),
            window,
            traceparent: crate::telemetry::context::inject_current(),
            reply,
        }
    })
    .await?;

    match seen {
        Some(seen) => Ok(seen),
        None => {
            tracing::warn!("no presence reading for sensor {sensor}");
            Ok(false)
        }
    }
}
