use serde::Deserialize;

#[derive(Deserialize)]
pub struct AndroidAppAlarmPayload {
    pub local_time: String,
}
