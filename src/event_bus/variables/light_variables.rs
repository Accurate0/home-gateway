use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct LightVariables {
    pub device: String,
    pub on: bool,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<String>,
}
