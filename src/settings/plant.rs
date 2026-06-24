/// Default esphome sensor entity for a plant: the Apollo PLT-1 publishes soil
/// moisture as a percentage on `<node>/sensor/soil_moisture/state`.
pub(crate) fn default_plant_entities() -> Vec<String> {
    vec!["soil_moisture".to_string()]
}

#[derive(Debug, Clone)]
pub struct PlantSensorSettings {
    #[allow(unused)]
    pub id: String,
    #[allow(unused)]
    pub entities: Vec<String>,
}
