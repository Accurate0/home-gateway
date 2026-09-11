use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct WoolworthsVariables {
    pub product_id: i64,
    pub name: String,
    pub old_price: f64,
    pub new_price: f64,
    pub drop: f64,
}
