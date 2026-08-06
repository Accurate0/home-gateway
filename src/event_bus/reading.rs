use serde::Deserialize;

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
    SoilMoisture,
    Other(String),
}

impl From<String> for SensorMetric {
    fn from(s: String) -> Self {
        match s.as_str() {
            "temperature" => SensorMetric::Temperature,
            "humidity" => SensorMetric::Humidity,
            "pressure" => SensorMetric::Pressure,
            "lux" => SensorMetric::Lux,
            "uv_index" => SensorMetric::UvIndex,
            "soil_moisture" => SensorMetric::SoilMoisture,
            _ => SensorMetric::Other(s),
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
    SoilMoisture { value: f64 },
    Other { name: String, value: f64 },
}

/// Template variable name for a metric, matching the config metric strings
/// (`temperature`, `soil_moisture`, …) so `${temperature}` works in a notify.
pub fn metric_var_name(metric: &SensorMetric) -> String {
    match metric {
        SensorMetric::Temperature => "temperature".to_owned(),
        SensorMetric::Humidity => "humidity".to_owned(),
        SensorMetric::Pressure => "pressure".to_owned(),
        SensorMetric::Lux => "lux".to_owned(),
        SensorMetric::UvIndex => "uv_index".to_owned(),
        SensorMetric::SoilMoisture => "soil_moisture".to_owned(),
        SensorMetric::Other(name) => name.clone(),
    }
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
            | SensorReading::SoilMoisture { value }
            | SensorReading::Other { value, .. } => *value,
        }
    }
}
