use crate::db::GarageDoorState;

pub struct GarageDoorReading {
    pub address: String,
    pub state: GarageDoorState,
    pub contact: Option<bool>,
}
