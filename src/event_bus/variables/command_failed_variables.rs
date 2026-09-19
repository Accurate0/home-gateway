use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct CommandFailedVariables {
    pub kind: String,
    pub address: String,
    pub device_id: Option<String>,
    pub attempts: i32,
}
