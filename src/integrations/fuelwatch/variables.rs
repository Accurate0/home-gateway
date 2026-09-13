use schemars::JsonSchema;

use super::types::FuelSite;
use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables, JsonSchema)]
pub struct FuelwatchVariables {
    pub site_id: i32,
    pub name: String,
    pub brand: String,
    pub suburb: String,
    pub address: String,
    pub price: f64,
    pub price_tomorrow: Option<f64>,
}

impl From<FuelSite> for FuelwatchVariables {
    fn from(site: FuelSite) -> Self {
        FuelwatchVariables {
            site_id: site.site_id,
            name: site.name,
            brand: site.brand,
            suburb: site.suburb,
            address: site.address,
            price: site.price,
            price_tomorrow: site.price_tomorrow,
        }
    }
}
