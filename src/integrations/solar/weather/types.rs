use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UVXMLDocument {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub location: Vec<Location>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub name: String,
    pub index: f64,
    pub time: String,
    pub date: String,
    pub fulldate: String,
    pub utcdatetime: String,
    pub status: String,
}
