use std::time::Duration;

use crate::actors::system::rpc;
use crate::device_command::CommandOutcome;
use crate::workflows::definition::GarageDoorCommand;

use super::command_request::CommandRequest;
use super::garage_door_command_error::GarageDoorCommandError;
use super::{GarageDoorHandler, Message};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn send(
    address: &str,
    command: GarageDoorCommand,
) -> Result<CommandOutcome, GarageDoorCommandError> {
    let outcome = rpc::query_factory(GarageDoorHandler::NAME, COMMAND_TIMEOUT, |reply| {
        Message::Command(CommandRequest {
            address: address.to_owned(),
            command,
            traceparent: crate::telemetry::context::inject_current(),
            reply,
        })
    })
    .await?;

    outcome.map_err(GarageDoorCommandError::Rejected)
}
