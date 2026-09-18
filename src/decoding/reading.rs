use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer};

use crate::device_metric::MetricValue;
use crate::settings::Metric;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceReading {
    #[serde(default, deserialize_with = "integer")]
    pub battery: Option<i64>,
    #[serde(default)]
    pub door: Option<DoorFields>,
    #[serde(default)]
    pub environment: Option<HashMap<Metric, f64>>,
    #[serde(default)]
    pub light: Option<LightFields>,
    #[serde(default)]
    pub smart_switch: Option<SmartSwitchFields>,
    #[serde(default)]
    pub presence: Option<PresenceFields>,
    #[serde(default)]
    pub control_switch: Option<ControlSwitchFields>,
    #[serde(default)]
    pub robot_vacuum: Option<RobotVacuumFields>,
    #[serde(default)]
    pub media_player: Option<MediaPlayerFields>,
    #[serde(default)]
    pub metrics: BTreeMap<String, ReadingMetric>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotVacuumFields {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub room: Option<String>,
    #[serde(default, deserialize_with = "integer")]
    pub battery: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaPlayerFields {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoorFields {
    #[serde(default)]
    pub contact: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightFields {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default, deserialize_with = "integer")]
    pub brightness: Option<i32>,
    #[serde(default, deserialize_with = "integer")]
    pub color_temp: Option<i32>,
    #[serde(default)]
    pub color: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SmartSwitchFields {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default, deserialize_with = "integer")]
    pub voltage: Option<i64>,
    #[serde(default, deserialize_with = "integer")]
    pub power: Option<i64>,
    #[serde(default)]
    pub current: Option<f64>,
    #[serde(default)]
    pub energy: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresenceFields {
    #[serde(default)]
    pub presence: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlSwitchFields {
    #[serde(default)]
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ReadingMetric {
    Flag(bool),
    Number(f64),
    Text(String),
}

impl From<ReadingMetric> for MetricValue {
    fn from(metric: ReadingMetric) -> Self {
        match metric {
            ReadingMetric::Flag(flag) => MetricValue::Text(flag.to_string()),
            ReadingMetric::Number(number) => MetricValue::Numeric(number),
            ReadingMetric::Text(text) => MetricValue::Text(text),
        }
    }
}

fn integer<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<i64>,
{
    let number = Option::<f64>::deserialize(deserializer)?;

    Ok(number
        .filter(|number| number.fract() == 0.0)
        .and_then(|number| T::try_from(number as i64).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_integral_float_is_accepted_as_an_integer() {
        let reading: DeviceReading = serde_json::from_str(r#"{"battery": 21.0}"#).expect("reading");

        assert_eq!(reading.battery, Some(21));
    }

    #[test]
    fn a_fractional_integer_field_is_dropped() {
        let reading: DeviceReading =
            serde_json::from_str(r#"{"smart_switch": {"voltage": 243.5}}"#).expect("reading");

        assert_eq!(reading.smart_switch.and_then(|fields| fields.voltage), None);
    }

    #[test]
    fn a_numeric_metric_does_not_coerce_from_a_string() {
        let reading: DeviceReading =
            serde_json::from_str(r#"{"metrics": {"power": "98"}}"#).expect("reading");

        assert_eq!(
            MetricValue::from(reading.metrics["power"].clone()),
            MetricValue::Text("98".to_owned())
        );
    }

    #[test]
    fn a_bool_metric_lands_as_text() {
        assert_eq!(
            MetricValue::from(ReadingMetric::Flag(false)),
            MetricValue::Text("false".to_owned())
        );
    }

    #[test]
    fn unknown_environment_metrics_are_rejected() {
        let error =
            serde_json::from_str::<DeviceReading>(r#"{"environment": {"wind_speed": 3.0}}"#)
                .expect_err("unknown metric");

        assert!(error.to_string().contains("wind_speed"), "{error}");
    }
}
