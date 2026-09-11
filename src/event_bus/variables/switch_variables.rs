use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct SwitchVariables {
    pub device: String,
    pub action: String,
}
