use ractor::RpcReplyPort;

use crate::settings::workflow::GarageDoorCommand;

use super::command_outcome::CommandOutcome;

pub struct CommandRequest {
    pub address: String,
    pub command: GarageDoorCommand,
    pub traceparent: crate::tracing_context::TraceParent,
    pub reply: RpcReplyPort<Result<CommandOutcome, String>>,
}
