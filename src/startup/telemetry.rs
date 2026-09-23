use prometheus::Registry;
use rustls::crypto::aws_lc_rs;

use crate::tracing_setup::{self, SamplingControl};

pub struct Telemetry {
    pub sampling: SamplingControl,
    pub metrics_registry: Registry,
}

pub fn init() -> Telemetry {
    if aws_lc_rs::default_provider().install_default().is_err() {
        tracing::debug!("a rustls crypto provider was already installed");
    }

    let sampling = tracing_setup::init();
    let metrics_registry = tracing_setup::init_metrics();

    Telemetry {
        sampling,
        metrics_registry,
    }
}
