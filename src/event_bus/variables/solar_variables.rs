use crate::integrations::solar::types::SolarCurrentStatisticsAverages;
use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct SolarVariables {
    pub current: f64,
    pub avg_15m: Option<f64>,
    pub avg_1h: Option<f64>,
    pub avg_3h: Option<f64>,
}

impl SolarVariables {
    pub fn new(current: f64, averages: Option<&SolarCurrentStatisticsAverages>) -> Self {
        SolarVariables {
            current,
            avg_15m: averages.and_then(|averages| averages.last_15_mins),
            avg_1h: averages.and_then(|averages| averages.last_1_hour),
            avg_3h: averages.and_then(|averages| averages.last_3_hours),
        }
    }
}
