use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct GarageDoorVariables {
    pub device: String,
    pub name: String,
    pub state: String,
    pub open: bool,
}
