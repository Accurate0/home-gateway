use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer};

use crate::device_metric::MetricValue;
use crate::settings::Metric;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZigbeeReading {
    #[serde(default, deserialize_with = "integer")]
    pub battery: Option<i64>,
    #[serde(default)]
    pub door: Option<ZigbeeDoorReading>,
    #[serde(default)]
    pub environment: Option<HashMap<Metric, f64>>,
    #[serde(default)]
    pub light: Option<ZigbeeLightReading>,
    #[serde(default)]
    pub smart_switch: Option<ZigbeeSmartSwitchReading>,
    #[serde(default)]
    pub presence: Option<ZigbeePresenceReading>,
    #[serde(default)]
    pub control_switch: Option<ZigbeeControlSwitchReading>,
    #[serde(default)]
    pub metrics: BTreeMap<String, ZigbeeMetric>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZigbeeDoorReading {
    #[serde(default)]
    pub contact: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZigbeeLightReading {
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
pub struct ZigbeeSmartSwitchReading {
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
pub struct ZigbeePresenceReading {
    #[serde(default)]
    pub presence: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZigbeeControlSwitchReading {
    #[serde(default)]
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ZigbeeMetric {
    Flag(bool),
    Number(f64),
    Text(String),
}

impl From<ZigbeeMetric> for MetricValue {
    fn from(metric: ZigbeeMetric) -> Self {
        match metric {
            ZigbeeMetric::Flag(flag) => MetricValue::Text(flag.to_string()),
            ZigbeeMetric::Number(number) => MetricValue::Numeric(number),
            ZigbeeMetric::Text(text) => MetricValue::Text(text),
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
        let reading: ZigbeeReading = serde_json::from_str(r#"{"battery": 21.0}"#).expect("reading");

        assert_eq!(reading.battery, Some(21));
    }

    #[test]
    fn a_fractional_integer_field_is_dropped() {
        let reading: ZigbeeReading =
            serde_json::from_str(r#"{"smart_switch": {"voltage": 243.5}}"#).expect("reading");

        assert_eq!(reading.smart_switch.and_then(|fields| fields.voltage), None);
    }

    #[test]
    fn a_numeric_metric_does_not_coerce_from_a_string() {
        let reading: ZigbeeReading =
            serde_json::from_str(r#"{"metrics": {"power": "98"}}"#).expect("reading");

        assert_eq!(
            MetricValue::from(reading.metrics["power"].clone()),
            MetricValue::Text("98".to_owned())
        );
    }

    #[test]
    fn a_bool_metric_lands_as_text() {
        assert_eq!(
            MetricValue::from(ZigbeeMetric::Flag(false)),
            MetricValue::Text("false".to_owned())
        );
    }

    #[test]
    fn unknown_environment_metrics_are_rejected() {
        let error =
            serde_json::from_str::<ZigbeeReading>(r#"{"environment": {"wind_speed": 3.0}}"#)
                .expect_err("unknown metric");

        assert!(error.to_string().contains("wind_speed"), "{error}");
    }
}
