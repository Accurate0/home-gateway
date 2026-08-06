use schemars::JsonSchema;
use serde::Deserialize;

/// S3 / object-storage config. Credentials are taken from the standard AWS
/// environment, never from this file. `endpoint` is only set for
/// S3-compatible stores (MinIO/R2/…); omit it for plain AWS S3.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct S3Settings {
    pub bucket: String,
    pub region: String,
    #[serde(default)]
    pub endpoint: Option<String>,
}
