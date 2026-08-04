use anyhow::Result;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::info;

const NVS_NAMESPACE: &str = "epd";
const IMAGE_HASH_KEY: &str = "img_hash";
const HASH_BUFFER_SIZE: usize = 80;

pub struct ImageHashStore {
    nvs: EspNvs<NvsDefault>,
}

impl ImageHashStore {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self> {
        let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)?;

        Ok(Self { nvs })
    }

    pub fn stored(&self) -> Option<String> {
        let mut buffer = [0u8; HASH_BUFFER_SIZE];

        match self.nvs.get_str(IMAGE_HASH_KEY, &mut buffer) {
            Ok(hash) => hash.map(|h| h.to_string()),
            Err(e) => {
                log::warn!("failed to read stored image hash: {e}");
                None
            }
        }
    }

    pub fn store(&mut self, hash: &str) {
        if let Err(e) = self.nvs.set_str(IMAGE_HASH_KEY, hash) {
            log::warn!("failed to store image hash: {e}");
            return;
        }

        info!("stored image hash {hash}");
    }

    pub fn clear(&mut self) {
        if let Err(e) = self.nvs.remove(IMAGE_HASH_KEY) {
            log::warn!("failed to clear image hash: {e}");
        }
    }
}
