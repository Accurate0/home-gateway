use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct UnifiVariables {
    pub client: String,
    pub mac_address: String,
    pub connected: bool,
}
