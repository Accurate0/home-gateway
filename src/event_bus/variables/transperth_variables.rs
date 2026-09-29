use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct TransperthVariables {
    pub route: String,
    pub origin: String,
    pub destination: String,
    pub departures: i32,
    pub next_line: Option<String>,
    pub next_headsign: Option<String>,
    pub next_minutes_away: Option<i32>,
    pub next_delay_minutes: Option<i32>,
}
