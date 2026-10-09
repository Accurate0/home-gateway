use std::time::Duration;

use crate::actors::system::rpc;
use crate::device_command::CommandOutcome;
use crate::workflows::definition::AirPurifierCommand;

use super::air_purifier_command_error::AirPurifierCommandError;
use super::command_request::CommandRequest;
use super::{AirPurifierHandler, Message};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn send(
    address: &str,
    command: AirPurifierCommand,
) -> Result<CommandOutcome, AirPurifierCommandError> {
    let outcome = rpc::query_factory(AirPurifierHandler::NAME, COMMAND_TIMEOUT, |reply| {
        Message::Command(CommandRequest {
            address: address.to_owned(),
            command,
            traceparent: crate::telemetry::context::inject_current(),
            reply,
        })
    })
    .await?;

    outcome.map_err(AirPurifierCommandError::Rejected)
}
