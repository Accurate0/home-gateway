use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct PresenceVariables {
    pub sensor: String,
    pub present: bool,
}
