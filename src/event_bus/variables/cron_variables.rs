use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct CronVariables {
    pub name: String,
}
