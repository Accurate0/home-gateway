use crate::repo::intent::AirPurifierAttributes;
use crate::telemetry::context::TraceParent;

pub struct Reapply {
    pub address: String,
    pub attributes: AirPurifierAttributes,
    pub traceparent: TraceParent,
}
