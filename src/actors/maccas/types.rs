use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOfferIngest {
    pub details: MaccasOfferDetails,
    pub offer: MaccasOffer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOfferDetails {
    pub name: String,
    pub description: String,
    pub price: Option<f64>,
    #[serde(rename = "short_name")]
    pub short_name: String,
    #[serde(rename = "image_base_name")]
    pub image_base_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOffer {
    #[serde(rename = "valid_from")]
    pub valid_from: String,
    #[serde(rename = "valid_to")]
    pub valid_to: String,
    #[serde(rename = "creation_date")]
    pub creation_date: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
}
