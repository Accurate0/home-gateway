use std::collections::BTreeMap;

use crate::device_registry::DeviceRegistry;
use crate::event_bus::SensorMetric;
use crate::event_bus::variables::{
    CommandFailedVariables, CronVariables, DeviceBatteryVariables, DeviceConnectionVariables,
    DoorVariables, EnvironmentVariables, FeatureFlagVariables, FuelWatchVariables,
    HomeAssistantVariables, LightVariables, MediaPlayerVariables, ModeVariables, PlantVariables,
    PresenceVariables, SolarVariables, SunVariables, SwitchVariables, UnifiVariables,
    WeatherVariables, WoolworthsVariables,
};
use crate::variables::{Shape, VarType, WorkflowContextVariables};

use super::TriggerMatcher;

impl TriggerMatcher {
    pub fn event_shape(&self, registry: &DeviceRegistry) -> Result<Shape, String> {
        let shape = match self {
            TriggerMatcher::Presence { .. } => PresenceVariables::shape(),
            TriggerMatcher::Door { .. } => DoorVariables::shape(),
            TriggerMatcher::Switch { .. } => SwitchVariables::shape(),
            TriggerMatcher::Cron { .. } => CronVariables::shape(),
            TriggerMatcher::Sun { .. } => SunVariables::shape(),
            TriggerMatcher::Mode { .. } => ModeVariables::shape(),
            TriggerMatcher::HomeAssistant { .. } => HomeAssistantVariables::shape(),
            TriggerMatcher::Woolworths { .. } => WoolworthsVariables::shape(),
            TriggerMatcher::FuelWatch { .. } => FuelWatchVariables::shape(),
            TriggerMatcher::DeviceBattery { .. } => DeviceBatteryVariables::shape(),
            TriggerMatcher::DeviceConnection { .. } => DeviceConnectionVariables::shape(),
            TriggerMatcher::MediaPlayer { .. } => MediaPlayerVariables::shape(),
            TriggerMatcher::Unifi { .. } => UnifiVariables::shape(),
            TriggerMatcher::Light { .. } => LightVariables::shape(),
            TriggerMatcher::CommandFailed { .. } => CommandFailedVariables::shape(),
            TriggerMatcher::FeatureFlag { .. } => FeatureFlagVariables::shape(),
            TriggerMatcher::Plant { sensor, .. } => {
                let metrics = registry.sensor_metrics(registry.address_or_self(sensor));

                if !metrics.contains(&SensorMetric::SoilMoisture) {
                    return Err(format!(
                        "plant trigger on `{sensor}`: sensor does not report soil moisture"
                    ));
                }

                PlantVariables::shape()
            }
            TriggerMatcher::Environment { sensor, metric, .. } => {
                let metrics = registry.sensor_metrics(registry.address_or_self(sensor));

                EnvironmentVariables::shape(&metrics, metric)
                    .map_err(|error| format!("environment trigger on `{sensor}`: {error}"))?
            }
            TriggerMatcher::Solar { metric, .. } => {
                let mut shape = SolarVariables::shape();
                shape.require(&[metric.to_string().as_str()]);
                shape
            }
            TriggerMatcher::Weather {
                source,
                metric,
                day,
                ..
            } => WeatherVariables::shape(*source, *metric, *day)?,
            TriggerMatcher::Custom { payload, .. } => {
                let payload = payload
                    .iter()
                    .map(|(key, ty)| (key.clone(), Shape::optional(*ty)))
                    .collect();

                Shape::Object(BTreeMap::from([
                    ("name".to_owned(), Shape::required(VarType::String)),
                    ("source".to_owned(), Shape::required(VarType::String)),
                    ("payload".to_owned(), Shape::Object(payload)),
                ]))
            }
        };

        Ok(shape)
    }
}
