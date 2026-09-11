use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct ModeVariables {
    pub mode: String,
    pub previous: String,
}
