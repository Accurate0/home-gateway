#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityDomain {
    Sensor,
    BinarySensor,
    TextSensor,
    Light,
    MediaPlayer,
}

impl std::fmt::Display for EntityDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            EntityDomain::Sensor => "sensor",
            EntityDomain::BinarySensor => "binary_sensor",
            EntityDomain::TextSensor => "text_sensor",
            EntityDomain::Light => "light",
            EntityDomain::MediaPlayer => "media_player",
        };

        f.write_str(name)
    }
}
