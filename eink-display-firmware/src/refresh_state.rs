use anyhow::Result;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

const NVS_NAMESPACE: &str = "epd";
const PENDING_KEY: &str = "pending";

pub struct RefreshState {
    nvs: EspNvs<NvsDefault>,
}

impl RefreshState {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self> {
        let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)?;

        Ok(Self { nvs })
    }

    pub fn pending(&self) -> bool {
        match self.nvs.get_u8(PENDING_KEY) {
            Ok(pending) => pending.unwrap_or(0) != 0,
            Err(e) => {
                log::warn!("failed to read the pending refresh flag: {e}");
                false
            }
        }
    }

    pub fn set_pending(&mut self, pending: bool) {
        if self.pending() == pending {
            return;
        }

        if let Err(e) = self.nvs.set_u8(PENDING_KEY, u8::from(pending)) {
            log::warn!("failed to store the pending refresh flag: {e}");
        }
    }
}
