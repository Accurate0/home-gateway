use crate::{device_metric::MetricValue, settings::Metric};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ZigbeeFieldType {
    Bool,
    Int,
    Float,
    String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ZigbeeField {
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: ZigbeeFieldType,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RawFieldBlock {
    Names(Vec<String>),
    Renames(HashMap<String, String>),
}

impl RawFieldBlock {
    fn normalize(self, slug: &str, block: &str) -> Result<HashMap<String, String>, String> {
        let mut fields = HashMap::new();

        let pairs = match self {
            RawFieldBlock::Names(names) => names.into_iter().map(|n| (n.clone(), n)).collect(),
            RawFieldBlock::Renames(renames) => renames.into_iter().collect::<Vec<_>>(),
        };

        for (logical, key) in pairs {
            if fields.insert(logical.clone(), key).is_some() {
                return Err(format!(
                    "zigbee model {slug}: `{block}` declares `{logical}` twice"
                ));
            }
        }

        Ok(fields)
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawZigbeeModelProfile {
    #[serde(default)]
    pub battery: Option<String>,
    #[serde(default)]
    pub door: Option<RawFieldBlock>,
    #[serde(default)]
    pub environment: Option<RawFieldBlock>,
    #[serde(default)]
    pub light: Option<RawFieldBlock>,
    #[serde(default)]
    pub smart_switch: Option<RawFieldBlock>,
    #[serde(default)]
    pub presence: Option<RawFieldBlock>,
    #[serde(default)]
    pub control_switch: Option<RawFieldBlock>,
    #[serde(default)]
    pub metrics: HashMap<String, ZigbeeField>,
}

#[derive(Debug, Clone)]
pub struct ZigbeeDoorFields {
    pub contact: String,
}

#[derive(Debug, Clone)]
pub struct ZigbeeLightFields {
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct ZigbeeSmartSwitchFields {
    pub voltage: String,
    pub power: String,
    pub current: String,
    pub energy: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ZigbeePresenceFields {
    pub presence: String,
}

#[derive(Debug, Clone)]
pub struct ZigbeeControlSwitchFields {
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct ZigbeeModelProfile {
    pub slug: String,
    pub battery: Option<String>,
    pub door: Option<ZigbeeDoorFields>,
    pub environment: Option<Vec<(Metric, String)>>,
    pub light: Option<ZigbeeLightFields>,
    pub smart_switch: Option<ZigbeeSmartSwitchFields>,
    pub presence: Option<ZigbeePresenceFields>,
    pub control_switch: Option<ZigbeeControlSwitchFields>,
    pub metrics: Vec<(String, ZigbeeField)>,
}

impl ZigbeeModelProfile {
    pub fn resolve(slug: String, raw: RawZigbeeModelProfile) -> Result<Self, String> {
        let RawZigbeeModelProfile {
            battery,
            door,
            environment,
            light,
            smart_switch,
            presence,
            control_switch,
            metrics,
        } = raw;

        let door = door
            .map(|block| {
                let mut fields = block.normalize(&slug, "door")?;
                let contact = take_required(&mut fields, &slug, "door", "contact")?;
                reject_unknown(fields, &slug, "door", &["contact"])?;

                Ok::<_, String>(ZigbeeDoorFields { contact })
            })
            .transpose()?;

        let light = light
            .map(|block| {
                let mut fields = block.normalize(&slug, "light")?;
                let state = take_required(&mut fields, &slug, "light", "state")?;
                reject_unknown(fields, &slug, "light", &["state"])?;

                Ok::<_, String>(ZigbeeLightFields { state })
            })
            .transpose()?;

        let smart_switch = smart_switch
            .map(|block| {
                let mut fields = block.normalize(&slug, "smart_switch")?;
                let voltage = take_required(&mut fields, &slug, "smart_switch", "voltage")?;
                let power = take_required(&mut fields, &slug, "smart_switch", "power")?;
                let current = take_required(&mut fields, &slug, "smart_switch", "current")?;
                let energy = take_required(&mut fields, &slug, "smart_switch", "energy")?;
                let state = fields.remove("state");
                reject_unknown(
                    fields,
                    &slug,
                    "smart_switch",
                    &["voltage", "power", "current", "energy", "state"],
                )?;

                Ok::<_, String>(ZigbeeSmartSwitchFields {
                    voltage,
                    power,
                    current,
                    energy,
                    state,
                })
            })
            .transpose()?;

        let presence = presence
            .map(|block| {
                let mut fields = block.normalize(&slug, "presence")?;
                let presence = take_required(&mut fields, &slug, "presence", "presence")?;
                reject_unknown(fields, &slug, "presence", &["presence"])?;

                Ok::<_, String>(ZigbeePresenceFields { presence })
            })
            .transpose()?;

        let control_switch = control_switch
            .map(|block| {
                let mut fields = block.normalize(&slug, "control_switch")?;
                let action = take_required(&mut fields, &slug, "control_switch", "action")?;
                reject_unknown(fields, &slug, "control_switch", &["action"])?;

                Ok::<_, String>(ZigbeeControlSwitchFields { action })
            })
            .transpose()?;

        let environment = environment
            .map(|block| resolve_environment(&slug, block))
            .transpose()?;

        let mut metrics: Vec<_> = metrics.into_iter().collect();
        metrics.sort_by(|(a, _), (b, _)| a.cmp(b));

        Ok(ZigbeeModelProfile {
            slug,
            battery,
            door,
            environment,
            light,
            smart_switch,
            presence,
            control_switch,
            metrics,
        })
    }
}

fn resolve_environment(slug: &str, block: RawFieldBlock) -> Result<Vec<(Metric, String)>, String> {
    let fields = block.normalize(slug, "environment")?;

    if fields.is_empty() {
        return Err(format!("zigbee model {slug}: `environment` is empty"));
    }

    let mut readings = Vec::new();
    for (logical, key) in fields {
        let metric = serde_yaml::from_str::<Metric>(&logical).map_err(|_| {
            format!("zigbee model {slug}: `environment` has unknown metric `{logical}`")
        })?;

        readings.push((metric, key));
    }

    if !readings.iter().any(|(m, _)| *m == Metric::Temperature) {
        return Err(format!(
            "zigbee model {slug}: `environment` must define `temperature`"
        ));
    }

    readings.sort_by(|(_, a), (_, b)| a.cmp(b));

    Ok(readings)
}

fn take_required(
    fields: &mut HashMap<String, String>,
    slug: &str,
    block: &str,
    logical: &str,
) -> Result<String, String> {
    fields
        .remove(logical)
        .ok_or_else(|| format!("zigbee model {slug}: `{block}` must define `{logical}`"))
}

fn reject_unknown(
    fields: HashMap<String, String>,
    slug: &str,
    block: &str,
    valid: &[&str],
) -> Result<(), String> {
    let Some(unknown) = fields.keys().min() else {
        return Ok(());
    };

    Err(format!(
        "zigbee model {slug}: `{block}` has unknown field `{unknown}`; valid fields are {}",
        valid.join(", ")
    ))
}

pub fn payload_bool(payload: &Map<String, Value>, key: &str) -> Option<bool> {
    payload.get(key)?.as_bool()
}

pub fn payload_i64(payload: &Map<String, Value>, key: &str) -> Option<i64> {
    let value = payload.get(key)?;

    if let Some(int) = value.as_i64() {
        return Some(int);
    }

    let float = value.as_f64()?;

    (float.fract() == 0.0).then_some(float as i64)
}

pub fn payload_f64(payload: &Map<String, Value>, key: &str) -> Option<f64> {
    payload.get(key)?.as_f64()
}

pub fn payload_string(payload: &Map<String, Value>, key: &str) -> Option<String> {
    payload.get(key)?.as_str().map(str::to_owned)
}

pub fn extract_metric(payload: &Map<String, Value>, field: &ZigbeeField) -> Option<MetricValue> {
    match field.field_type {
        ZigbeeFieldType::Bool => {
            payload_bool(payload, &field.key).map(|b| MetricValue::Text(b.to_string()))
        }
        ZigbeeFieldType::Int => {
            payload_i64(payload, &field.key).map(|i| MetricValue::Numeric(i as f64))
        }
        ZigbeeFieldType::Float => payload_f64(payload, &field.key).map(MetricValue::Numeric),
        ZigbeeFieldType::String => payload_string(payload, &field.key).map(MetricValue::Text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(yaml: &str) -> Result<ZigbeeModelProfile, String> {
        let raw = serde_yaml::from_str::<RawZigbeeModelProfile>(yaml).expect("profile yaml");

        ZigbeeModelProfile::resolve("test_model".to_owned(), raw)
    }

    fn payload(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("payload json")
    }

    #[test]
    fn list_and_identity_map_forms_resolve_the_same() {
        let from_list = profile("smart_switch: [state, voltage, power, current, energy]")
            .expect("list form")
            .smart_switch
            .expect("smart switch block");

        let from_map = profile(
            "smart_switch: { state: state, voltage: voltage, power: power, current: current, energy: energy }",
        )
        .expect("map form")
        .smart_switch
        .expect("smart switch block");

        assert_eq!(from_list.voltage, from_map.voltage);
        assert_eq!(from_list.energy, from_map.energy);
        assert_eq!(from_list.state, from_map.state);
    }

    #[test]
    fn rename_map_maps_logical_name_to_payload_key() {
        let fields = profile("smart_switch: { state: state, voltage: voltage, power: active_power, current: current, energy: total_energy }")
            .expect("rename form")
            .smart_switch
            .expect("smart switch block");

        assert_eq!(fields.power, "active_power");
        assert_eq!(fields.energy, "total_energy");
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let error = profile("door: []").expect_err("door without contact");

        assert!(error.contains("must define `contact`"), "{error}");
    }

    #[test]
    fn unknown_logical_name_is_rejected() {
        let error = profile("door: [contact, nonsense]").expect_err("unknown door field");

        assert!(error.contains("unknown field `nonsense`"), "{error}");
        assert!(error.contains("valid fields are contact"), "{error}");
    }

    #[test]
    fn environment_requires_temperature_and_rejects_unknown_metrics() {
        let missing = profile("environment: [humidity]").expect_err("no temperature");
        assert!(missing.contains("must define `temperature`"), "{missing}");

        let unknown = profile("environment: [temperature, wind_speed]").expect_err("bad metric");
        assert!(unknown.contains("unknown metric `wind_speed`"), "{unknown}");
    }

    #[test]
    fn environment_resolves_metric_names_to_payload_keys() {
        let readings = profile("environment: [temperature, humidity, pm25, voc_index]")
            .expect("environment")
            .environment
            .expect("environment block");

        assert!(readings.contains(&(Metric::Temperature, "temperature".to_owned())));
        assert!(readings.contains(&(Metric::Pm25, "pm25".to_owned())));
        assert!(readings.contains(&(Metric::VocIndex, "voc_index".to_owned())));
    }

    #[test]
    fn payload_accessors_reject_mismatched_types() {
        let payload = payload(r#"{"battery": 97, "contact": false, "state": "ON", "temp": 18.4}"#);

        assert_eq!(payload_i64(&payload, "battery"), Some(97));
        assert_eq!(payload_bool(&payload, "contact"), Some(false));
        assert_eq!(payload_string(&payload, "state"), Some("ON".to_owned()));
        assert_eq!(payload_f64(&payload, "temp"), Some(18.4));

        assert_eq!(payload_bool(&payload, "state"), None);
        assert_eq!(payload_i64(&payload, "state"), None);
        assert_eq!(payload_i64(&payload, "temp"), None);
        assert_eq!(payload_string(&payload, "battery"), None);
        assert_eq!(payload_f64(&payload, "missing"), None);
    }

    #[test]
    fn payload_i64_accepts_an_integral_float() {
        let payload = payload(r#"{"device_temperature": 21.0}"#);

        assert_eq!(payload_i64(&payload, "device_temperature"), Some(21));
    }

    #[test]
    fn extract_metric_follows_the_declared_type() {
        let payload = payload(r#"{"distance": 1.4, "movement": "approach", "count": 3}"#);

        let float = ZigbeeField {
            key: "distance".to_owned(),
            field_type: ZigbeeFieldType::Float,
        };
        let text = ZigbeeField {
            key: "movement".to_owned(),
            field_type: ZigbeeFieldType::String,
        };
        let int = ZigbeeField {
            key: "count".to_owned(),
            field_type: ZigbeeFieldType::Int,
        };

        assert_eq!(
            extract_metric(&payload, &float),
            Some(MetricValue::Numeric(1.4))
        );
        assert_eq!(
            extract_metric(&payload, &text),
            Some(MetricValue::Text("approach".to_owned()))
        );
        assert_eq!(
            extract_metric(&payload, &int),
            Some(MetricValue::Numeric(3.0))
        );
    }

    #[test]
    fn extract_metric_does_not_coerce_across_kinds() {
        let payload = payload(r#"{"power": "98"}"#);

        let numeric = ZigbeeField {
            key: "power".to_owned(),
            field_type: ZigbeeFieldType::Float,
        };

        assert_eq!(extract_metric(&payload, &numeric), None);
    }

    #[test]
    fn a_metric_written_as_a_bare_string_is_rejected() {
        let raw = serde_yaml::from_str::<RawZigbeeModelProfile>("metrics: { voltage: voltage }");

        assert!(raw.is_err());
    }
}
