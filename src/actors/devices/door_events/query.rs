use std::time::Duration;

use super::{DerivedDoorEvents, DoorEventsMessage};
use crate::actors::system::rpc::{self, RpcError};
use crate::repo::door::DoorState;

pub async fn query_door_open(ieee_addr: &str, timeout: Duration) -> Result<bool, RpcError> {
    let state: Option<DoorState> = rpc::query(DerivedDoorEvents::NAME, timeout, |reply| {
        DoorEventsMessage::QueryState {
            ieee_addr: ieee_addr.to_owned(),
            reply,
        }
    })
    .await?;

    match state {
        Some(state) => Ok(matches!(state, DoorState::Open)),
        None => {
            tracing::warn!("no door state for {ieee_addr}");
            Ok(false)
        }
    }
}
