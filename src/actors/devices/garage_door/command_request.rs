use ractor::RpcReplyPort;

use crate::device_command::CommandOutcome;
use crate::settings::workflow::GarageDoorCommand;

pub struct CommandRequest {
    pub address: String,
    pub command: GarageDoorCommand,
    pub traceparent: crate::tracing_context::TraceParent,
    pub reply: RpcReplyPort<Result<CommandOutcome, String>>,
}
