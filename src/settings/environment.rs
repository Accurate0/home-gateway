use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EnvironmentSensorType {
    #[default]
    Zigbee,
    Esphome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_form_maps_metric_to_object_id() {
        let map: HashMap<Metric, String> =
            serde_yaml::from_str("temperature: my_custom_temp\nlux: my_lux").expect("map yaml");
        assert_eq!(
            map.get(&Metric::Temperature).map(String::as_str),
            Some("my_custom_temp")
        );
        assert_eq!(map.get(&Metric::Lux).map(String::as_str), Some("my_lux"));
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentSensorSettings {
    pub id: String,
    pub name: String,
    #[allow(unused)]
    pub sensor_type: EnvironmentSensorType,
    /// esphome sensor object_id → logical metric. Keys are the topics to
    /// subscribe to on discovery; the value tells the actor which reading column
    /// each object_id feeds. Empty for zigbee sources (their readings arrive as
    /// typed device structs, not by object_id).
    pub entities: HashMap<String, Metric>,
}
