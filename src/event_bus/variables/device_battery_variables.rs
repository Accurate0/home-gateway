use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct DeviceBatteryVariables {
    pub device_id: String,
    pub kind: String,
    pub name: String,
    pub battery_voltage: Option<f64>,
    pub battery_percent: Option<f64>,
}
