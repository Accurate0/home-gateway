use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct DeviceConnectionVariables {
    pub device_id: String,
    pub transport: String,
    pub room: Option<String>,
    pub connected: bool,
}
