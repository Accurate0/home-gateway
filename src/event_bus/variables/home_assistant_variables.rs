use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct HomeAssistantVariables {
    pub entity_id: String,
    pub state: String,
}
