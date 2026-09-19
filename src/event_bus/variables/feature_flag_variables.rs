use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct FeatureFlagVariables {
    pub state: String,
    pub version: Option<String>,
}
