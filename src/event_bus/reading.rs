use serde::Deserialize;

use crate::settings::Metric;

/// A named scalar sensor reading. Known metrics are typed; anything else (an
/// esphome object_id we don't model) falls back to [`SensorMetric::Other`] so
/// producers and config stay honest without an exhaustive list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(from = "String")]
#[schemars(with = "String")]
pub enum SensorMetric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
    Pm25,
    VocIndex,
    SoilMoisture,
    Other(String),
}

impl std::fmt::Display for SensorMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SensorMetric::Temperature => "temperature",
            SensorMetric::Humidity => "humidity",
            SensorMetric::Pressure => "pressure",
            SensorMetric::Lux => "lux",
            SensorMetric::UvIndex => "uv_index",
            SensorMetric::Pm25 => "pm25",
            SensorMetric::VocIndex => "voc_index",
            SensorMetric::SoilMoisture => "soil_moisture",
            SensorMetric::Other(name) => name,
        };

        f.write_str(name)
    }
}

impl From<String> for SensorMetric {
    fn from(s: String) -> Self {
        match s.as_str() {
            "temperature" => SensorMetric::Temperature,
            "humidity" => SensorMetric::Humidity,
            "pressure" => SensorMetric::Pressure,
            "lux" => SensorMetric::Lux,
            "uv_index" => SensorMetric::UvIndex,
            "pm25" => SensorMetric::Pm25,
            "voc_index" => SensorMetric::VocIndex,
            "soil_moisture" => SensorMetric::SoilMoisture,
            _ => SensorMetric::Other(s),
        }
    }
}

impl From<Metric> for SensorMetric {
    fn from(metric: Metric) -> Self {
        match metric {
            Metric::Temperature => SensorMetric::Temperature,
            Metric::Humidity => SensorMetric::Humidity,
            Metric::Pressure => SensorMetric::Pressure,
            Metric::Lux => SensorMetric::Lux,
            Metric::UvIndex => SensorMetric::UvIndex,
            Metric::Pm25 => SensorMetric::Pm25,
            Metric::VocIndex => SensorMetric::VocIndex,
        }
    }
}

/// A sensor reading: which metric, and its scalar value. The runtime companion
/// to [`SensorMetric`] (which is the value-less discriminant used in config).
#[derive(Debug, Clone, PartialEq)]
pub enum SensorReading {
    Temperature { value: f64 },
    Humidity { value: f64 },
    Pressure { value: f64 },
    Lux { value: f64 },
    UvIndex { value: f64 },
    Pm25 { value: f64 },
    VocIndex { value: f64 },
    SoilMoisture { value: f64 },
    Other { name: String, value: f64 },
}

impl SensorReading {
    /// Build a reading from a metric discriminant and value (e.g. mapping a raw
    /// esphome object_id through [`SensorMetric::from`]).
    pub fn new(metric: SensorMetric, value: f64) -> Self {
        match metric {
            SensorMetric::Temperature => SensorReading::Temperature { value },
            SensorMetric::Humidity => SensorReading::Humidity { value },
            SensorMetric::Pressure => SensorReading::Pressure { value },
            SensorMetric::Lux => SensorReading::Lux { value },
            SensorMetric::UvIndex => SensorReading::UvIndex { value },
            SensorMetric::Pm25 => SensorReading::Pm25 { value },
            SensorMetric::VocIndex => SensorReading::VocIndex { value },
            SensorMetric::SoilMoisture => SensorReading::SoilMoisture { value },
            SensorMetric::Other(name) => SensorReading::Other { name, value },
        }
    }

    pub fn metric(&self) -> SensorMetric {
        match self {
            SensorReading::Temperature { .. } => SensorMetric::Temperature,
            SensorReading::Humidity { .. } => SensorMetric::Humidity,
            SensorReading::Pressure { .. } => SensorMetric::Pressure,
            SensorReading::Lux { .. } => SensorMetric::Lux,
            SensorReading::UvIndex { .. } => SensorMetric::UvIndex,
            SensorReading::Pm25 { .. } => SensorMetric::Pm25,
            SensorReading::VocIndex { .. } => SensorMetric::VocIndex,
            SensorReading::SoilMoisture { .. } => SensorMetric::SoilMoisture,
            SensorReading::Other { name, .. } => SensorMetric::Other(name.clone()),
        }
    }

    pub fn value(&self) -> f64 {
        match self {
            SensorReading::Temperature { value }
            | SensorReading::Humidity { value }
            | SensorReading::Pressure { value }
            | SensorReading::Lux { value }
            | SensorReading::UvIndex { value }
            | SensorReading::Pm25 { value }
            | SensorReading::VocIndex { value }
            | SensorReading::SoilMoisture { value }
            | SensorReading::Other { value, .. } => *value,
        }
    }
}
