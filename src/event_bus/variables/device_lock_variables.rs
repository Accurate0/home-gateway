use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct DeviceLockVariables {
    pub device_id: String,
    pub locked: bool,
    pub source: Option<String>,
    pub expires_at: Option<i64>,
}
