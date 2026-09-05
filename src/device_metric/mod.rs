use uuid::Uuid;

pub mod value;

pub use value::MetricValue;

pub struct DeviceMetric {
    pub event_id: Uuid,
    pub address: String,
    pub device_id: Option<String>,
    pub metric: String,
    pub value: MetricValue,
}
