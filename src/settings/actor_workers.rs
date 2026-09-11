use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ActorWorkerSettings {
    pub mqtt_ingest: usize,
    pub control_switch: usize,
    pub door_sensor: usize,
    pub environment_sensor: usize,
    pub light: usize,
    pub media_player: usize,
    pub plant_sensor: usize,
    pub presence_sensor: usize,
    pub robot_vacuum: usize,
    pub smart_switch: usize,
}
