use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct DoorVariables {
    pub device: String,
    pub open: bool,
}
