use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct JellyfinVariables {
    pub state: String,
    pub session_id: String,
    pub user: String,
    pub device: String,
    pub client: String,
    pub item: String,
    pub item_type: String,
    pub series: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub position: Option<f64>,
    pub runtime: Option<f64>,
    pub play_method: Option<String>,
}
