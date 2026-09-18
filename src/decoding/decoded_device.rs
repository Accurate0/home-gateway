use std::sync::Arc;

use super::model_profile::ModelProfile;

#[derive(Debug, Clone)]
pub struct DecodedDevice {
    pub id: String,
    pub address: String,
    pub profile: Arc<ModelProfile>,
}
