use crate::device_registry::DeviceRegistry;
use crate::event_bus::variables::{
    CronVariables, DeviceBatteryVariables, DoorVariables, EnvironmentVariables, FuelWatchVariables,
    HomeAssistantVariables, JellyfinVariables, MediaPlayerVariables, ModeVariables,
    PresenceVariables, SolarVariables, SunVariables, SwitchVariables, WeatherVariables,
    WoolworthsVariables,
};
use crate::variables::{Shape, WorkflowContextVariables};

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
            TriggerMatcher::Jellyfin { .. } => JellyfinVariables::shape(),
            TriggerMatcher::MediaPlayer { .. } => MediaPlayerVariables::shape(),
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
        };

        Ok(shape)
    }
}
