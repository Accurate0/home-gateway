use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
#[graphql(name = "HomeDeviceConnection")]
pub struct DeviceConnectionObject {
    pub device_id: String,
    pub connected: bool,
    pub changed_at: DateTime<Utc>,
}
