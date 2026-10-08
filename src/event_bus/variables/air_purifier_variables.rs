use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct AirPurifierVariables {
    pub device: String,
    pub name: String,
    pub on: bool,
    pub mode: Option<String>,
    pub speed: Option<i32>,
    pub display: Option<bool>,
}
