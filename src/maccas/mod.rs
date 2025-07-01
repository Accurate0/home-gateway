use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOfferIngest {
    pub details: MaccasOfferDetails,
    pub offer: MaccasOffer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOfferDetails {
    #[serde(rename = "proposition_id")]
    pub proposition_id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    #[serde(rename = "short_name")]
    pub short_name: String,
    #[serde(rename = "image_base_name")]
    pub image_base_name: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub categories: Value,
    pub migrated: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOffer {
    pub id: String,
    #[serde(rename = "offer_id")]
    pub offer_id: i64,
    #[serde(rename = "valid_from")]
    pub valid_from: String,
    #[serde(rename = "valid_to")]
    pub valid_to: String,
    #[serde(rename = "creation_date")]
    pub creation_date: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "offer_proposition_id")]
    pub offer_proposition_id: i64,
    #[serde(rename = "account_id")]
    pub account_id: String,
}
