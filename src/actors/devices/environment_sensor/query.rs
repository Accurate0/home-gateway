use std::time::Duration;

use super::{EnvironmentSensorHandler, LatestReading, Message as EnvironmentMessage};
use crate::actors::system::rpc::{self, RpcError};
use crate::workflows::definition::EnvMetric;

pub async fn query_environment(
    sensor: &str,
    timeout: Duration,
) -> Result<Option<LatestReading>, RpcError> {
    rpc::query_factory(EnvironmentSensorHandler::NAME, timeout, |reply| {
        EnvironmentMessage::QueryLatest {
            entity_id: sensor.to_owned(),
            reply,
        }
    })
    .await
}

pub fn environment_metric(reading: &LatestReading, metric: EnvMetric) -> Option<f64> {
    match metric {
        EnvMetric::Temperature => Some(reading.temperature),
        EnvMetric::Humidity => reading.humidity,
        EnvMetric::Pressure => reading.pressure,
        EnvMetric::Lux => reading.lux,
        EnvMetric::UvIndex => reading.uv_index,
    }
}
