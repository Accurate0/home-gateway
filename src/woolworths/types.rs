use serde::Deserialize;
use serde::Serialize;

#[derive(Eq)]
pub struct WoolworthsTrackedProduct {
    pub id: uuid::Uuid,
    pub product_id: i64,
    pub notify_channel: i64,
    pub mentions: Vec<i64>,
}

impl PartialEq for WoolworthsTrackedProduct {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl std::hash::Hash for WoolworthsTrackedProduct {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u128(self.id.as_u128());
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WoolworthsProductResponse {
    #[serde(rename = "Product")]
    pub product: Product,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "TileID")]
    pub tile_id: i64,
    #[serde(rename = "Stockcode")]
    pub stockcode: i64,
    #[serde(rename = "Barcode")]
    pub barcode: String,
    #[serde(rename = "GtinFormat")]
    pub gtin_format: i64,
    #[serde(rename = "CupPrice")]
    pub cup_price: f64,
    #[serde(rename = "InstoreCupPrice")]
    pub instore_cup_price: f64,
    #[serde(rename = "CupMeasure")]
    pub cup_measure: String,
    #[serde(rename = "CupString")]
    pub cup_string: String,
    #[serde(rename = "InstoreCupString")]
    pub instore_cup_string: String,
    #[serde(rename = "HasCupPrice")]
    pub has_cup_price: bool,
    #[serde(rename = "InstoreHasCupPrice")]
    pub instore_has_cup_price: bool,
    #[serde(rename = "Price")]
    pub price: f64,
}
