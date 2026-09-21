use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
pub struct EsphomeDiscovery {
    pub name: String,
    pub friendly_name: String,
    #[allow(unused)]
    pub mac: Option<String>,
    #[allow(unused)]
    pub ip: Option<String>,
    #[allow(unused)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EsphomeDomain {
    Sensor,
    BinarySensor,
    Light,
}

impl std::fmt::Display for EsphomeDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            EsphomeDomain::Sensor => "sensor",
            EsphomeDomain::BinarySensor => "binary_sensor",
            EsphomeDomain::Light => "light",
        };

        f.write_str(name)
    }
}

impl EsphomeDomain {
    pub fn parse(self, payload: &[u8]) -> Option<Value> {
        match self {
            EsphomeDomain::Sensor => parse_sensor_state(payload).map(Value::from),
            EsphomeDomain::BinarySensor => parse_binary_state(payload).map(Value::from),
            EsphomeDomain::Light => parse_light_state(payload),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EsphomeTarget {
    pub node: String,
    pub domain: EsphomeDomain,
    pub object_id: String,
}

impl EsphomeTarget {
    pub fn state_topic(&self) -> String {
        format!("{}/{}/{}/state", self.node, self.domain, self.object_id)
    }
}

pub fn light_command_topic(node: &str, object_id: &str) -> String {
    format!("{node}/light/{object_id}/command")
}

fn parse_light_state(payload: &[u8]) -> Option<Value> {
    #[derive(Deserialize)]
    struct Colour {
        r: u8,
        g: u8,
        b: u8,
    }

    #[derive(Deserialize)]
    struct LightState {
        state: String,
        brightness: Option<u32>,
        color: Option<Colour>,
    }

    if let Some(on) = parse_binary_state(payload) {
        return Some(json!({ "state": on_off(on) }));
    }

    let parsed: LightState = serde_json::from_slice(payload).ok()?;
    let on = parse_binary_state(parsed.state.as_bytes())?;

    let brightness = parsed.brightness.map(|value| (value * 254).div_ceil(255));

    let colour = parsed.color.map(
        |colour| json!({ "hex": format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b) }),
    );

    Some(json!({
        "state": on_off(on),
        "brightness": brightness,
        "color": colour,
    }))
}

fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

fn parse_binary_state(payload: &[u8]) -> Option<bool> {
    match payload {
        b"ON" => Some(true),
        b"OFF" => Some(false),
        _ => None,
    }
}

fn parse_sensor_state(payload: &[u8]) -> Option<f64> {
    std::str::from_utf8(payload).ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_target_builds_its_state_topic() {
        let target = EsphomeTarget {
            node: "apollo-mtr-1-livingroom".to_owned(),
            domain: EsphomeDomain::BinarySensor,
            object_id: "ld2450_presence".to_owned(),
        };

        assert_eq!(
            target.state_topic(),
            "apollo-mtr-1-livingroom/binary_sensor/ld2450_presence/state"
        );
    }

    #[test]
    fn each_domain_normalises_its_payload() {
        assert_eq!(EsphomeDomain::Sensor.parse(b" 21.5 "), Some(json!(21.5)));
        assert_eq!(EsphomeDomain::BinarySensor.parse(b"ON"), Some(json!(true)));
        assert_eq!(EsphomeDomain::BinarySensor.parse(b"maybe"), None);
        assert_eq!(
            EsphomeDomain::Light
                .parse(br#"{"state":"ON","brightness":255,"color":{"r":255,"g":0,"b":16}}"#),
            Some(json!({ "state": "ON", "brightness": 254, "color": { "hex": "#ff0010" } }))
        );
        assert_eq!(
            EsphomeDomain::Light.parse(b"OFF"),
            Some(json!({ "state": "OFF" }))
        );
    }
}
