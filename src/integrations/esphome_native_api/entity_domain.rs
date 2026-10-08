#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityDomain {
    Sensor,
    BinarySensor,
    TextSensor,
    Light,
    MediaPlayer,
    Fan,
    Switch,
    Number,
    Select,
}

impl std::fmt::Display for EntityDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            EntityDomain::Sensor => "sensor",
            EntityDomain::BinarySensor => "binary_sensor",
            EntityDomain::TextSensor => "text_sensor",
            EntityDomain::Light => "light",
            EntityDomain::MediaPlayer => "media_player",
            EntityDomain::Fan => "fan",
            EntityDomain::Switch => "switch",
            EntityDomain::Number => "number",
            EntityDomain::Select => "select",
        };

        f.write_str(name)
    }
}
