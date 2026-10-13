use ractor::RpcReplyPort;

use crate::device_command::{CommandOrigin, CommandOutcome};
use crate::workflows::definition::AirPurifierCommand;

pub struct CommandRequest {
    pub address: String,
    pub command: AirPurifierCommand,
    pub origin: CommandOrigin,
    pub traceparent: crate::telemetry::context::TraceParent,
    pub reply: RpcReplyPort<Result<CommandOutcome, String>>,
}
