use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LocationSettings {
    pub latitude: f64,
    pub longitude: f64,
}
