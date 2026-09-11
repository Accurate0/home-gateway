use prometheus::Registry;
use rustls::crypto::aws_lc_rs;

use crate::tracing_setup::{self, SamplingControl};

pub struct Telemetry {
    pub sampling: SamplingControl,
    pub metrics_registry: Registry,
}

pub fn init() -> Telemetry {
    aws_lc_rs::default_provider().install_default().unwrap();

    let sampling = tracing_setup::init();
    let metrics_registry = tracing_setup::init_metrics();

    Telemetry {
        sampling,
        metrics_registry,
    }
}
