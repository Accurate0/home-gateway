use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct FuelWatchVariables {
    pub change: String,
    pub site_id: i32,
    pub name: String,
    pub brand: String,
    pub suburb: String,
    pub address: String,
    pub old_price: f64,
    pub new_price: f64,
    pub drop: f64,
}
