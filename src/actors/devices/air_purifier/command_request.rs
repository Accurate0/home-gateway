use ractor::RpcReplyPort;

use crate::device_command::CommandOutcome;
use crate::settings::workflow::AirPurifierCommand;

pub struct CommandRequest {
    pub address: String,
    pub command: AirPurifierCommand,
    pub traceparent: crate::tracing_context::TraceParent,
    pub reply: RpcReplyPort<Result<CommandOutcome, String>>,
}
