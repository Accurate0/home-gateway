pub struct RoborockReading {
    pub device_id: String,
    pub status: Option<String>,
    pub room: Option<String>,
    pub battery: Option<i64>,
}
