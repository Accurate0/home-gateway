use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct MediaPlayerVariables {
    pub device: String,
    pub name: String,
    pub room: Option<String>,
    pub state: String,
    pub entity_state: String,
    pub app: Option<String>,
    pub source: Option<String>,
    pub item: Option<String>,
    pub series: Option<String>,
    pub item_type: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
    pub volume: Option<f64>,
    pub muted: Option<bool>,
}
