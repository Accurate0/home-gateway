use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SamplingSettings {
    pub default: f64,
    pub spans: HashMap<String, f64>,
}

impl SamplingSettings {
    pub fn validate(&self) -> Result<(), String> {
        let in_range = |ratio: f64| (0.0..=1.0).contains(&ratio);

        if !in_range(self.default) {
            return Err(format!(
                "tracing.sampling.default must be between 0 and 1, got {}",
                self.default
            ));
        }

        for (span, ratio) in &self.spans {
            if !in_range(*ratio) {
                return Err(format!(
                    "tracing.sampling.spans.{span} must be between 0 and 1, got {ratio}"
                ));
            }
        }

        Ok(())
    }
}
