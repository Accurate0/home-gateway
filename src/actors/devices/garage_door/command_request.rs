use ractor::RpcReplyPort;

use crate::device_command::CommandOutcome;
use crate::workflows::definition::GarageDoorCommand;

pub struct CommandRequest {
    pub address: String,
    pub command: GarageDoorCommand,
    pub traceparent: crate::telemetry::context::TraceParent,
    pub reply: RpcReplyPort<Result<CommandOutcome, String>>,
}
