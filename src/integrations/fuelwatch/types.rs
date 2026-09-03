use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct FuelSite {
    pub name: String,
    pub brand: String,
    pub suburb: String,
    pub postcode: i32,
    pub address: String,
    pub price: f64,
    pub price_tomorrow: Option<f64>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site_name: String,
    pub brand_name: String,
    pub address: Address,
    pub product: Product,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub line1: String,
    pub location: String,
    pub post_code: i32,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub price_today: Option<f64>,
    pub price_tomorrow: Option<f64>,
}
