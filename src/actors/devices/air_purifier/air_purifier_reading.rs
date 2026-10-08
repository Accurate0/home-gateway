use crate::db::AirPurifierMode;

pub struct AirPurifierReading {
    pub address: String,
    pub on: bool,
    pub mode: Option<AirPurifierMode>,
    pub speed: Option<i32>,
    pub pm25: Option<f64>,
    pub filter_life: Option<f64>,
    pub display: Option<bool>,
}
