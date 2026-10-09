use crate::repo::garage_door::GarageDoorState;

pub struct GarageDoorReading {
    pub address: String,
    pub state: GarageDoorState,
    pub contact: Option<bool>,
}
