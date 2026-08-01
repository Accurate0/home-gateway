use anyhow::{Context, Result};
use esp_idf_svc::http::Method;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::ota::EspOta;
use log::info;

use crate::http_client;

const CHUNK_SIZE: usize = 4096;
const NVS_NAMESPACE: &str = "ota";
const ATTEMPT_VERSION_KEY: &str = "attempt_ver";
const ATTEMPT_COUNT_KEY: &str = "attempt_count";
const MAX_ATTEMPTS: u8 = 3;
const VERSION_BUFFER_SIZE: usize = 64;

pub struct AttemptTracker {
    nvs: EspNvs<NvsDefault>,
}

impl AttemptTracker {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self> {
        let nvs = EspNvs::new(partition, NVS_NAMESPACE, true)?;

        Ok(Self { nvs })
    }

    fn attempted_version(&self, buffer: &mut [u8]) -> Option<String> {
        match self.nvs.get_str(ATTEMPT_VERSION_KEY, buffer) {
            Ok(version) => version.map(|v| v.to_string()),
            Err(e) => {
                log::warn!("failed to read ota attempt version: {e}");
                None
            }
        }
    }

    pub fn should_attempt(&self, version: &str) -> bool {
        let mut buffer = [0u8; VERSION_BUFFER_SIZE];

        if self.attempted_version(&mut buffer).as_deref() != Some(version) {
            return true;
        }

        let count = self
            .nvs
            .get_u8(ATTEMPT_COUNT_KEY)
            .unwrap_or(None)
            .unwrap_or(0);

        if count >= MAX_ATTEMPTS {
            log::error!("giving up on firmware {version} after {count} failed attempts");
            return false;
        }

        true
    }

    pub fn record_attempt(&mut self, version: &str) {
        let mut buffer = [0u8; VERSION_BUFFER_SIZE];

        let count = if self.attempted_version(&mut buffer).as_deref() == Some(version) {
            self.nvs
                .get_u8(ATTEMPT_COUNT_KEY)
                .unwrap_or(None)
                .unwrap_or(0)
                + 1
        } else {
            1
        };

        if let Err(e) = self.nvs.set_str(ATTEMPT_VERSION_KEY, version) {
            log::warn!("failed to record ota attempt version: {e}");
        }

        if let Err(e) = self.nvs.set_u8(ATTEMPT_COUNT_KEY, count) {
            log::warn!("failed to record ota attempt count: {e}");
        }

        info!("ota attempt {count}/{MAX_ATTEMPTS} for firmware {version}");
    }

    pub fn confirm(&mut self, running_version: &str) {
        let mut buffer = [0u8; VERSION_BUFFER_SIZE];

        if self.attempted_version(&mut buffer).as_deref() != Some(running_version) {
            return;
        }

        info!("firmware {running_version} confirmed good, clearing ota attempts");

        if let Err(e) = self.nvs.remove(ATTEMPT_VERSION_KEY) {
            log::warn!("failed to clear ota attempt version: {e}");
        }

        if let Err(e) = self.nvs.remove(ATTEMPT_COUNT_KEY) {
            log::warn!("failed to clear ota attempt count: {e}");
        }
    }
}

pub fn mark_valid() -> Result<()> {
    let mut ota = EspOta::new()?;
    ota.mark_running_slot_valid()?;

    Ok(())
}

pub fn apply(url: &str) -> Result<()> {
    info!("downloading firmware from {}...", url);

    let mut client = http_client::client()?;
    let headers = [("X-Api-Key", http_client::api_key())];
    let request = client.request(Method::Get, url, &headers)?;
    let mut response = request.submit()?;

    let status = response.status();
    info!("response status: {}", status);

    if status != 200 {
        anyhow::bail!("unexpected status code: {}", status);
    }

    let mut ota = EspOta::new()?;
    let mut update = ota.initiate_update()?;

    let mut buffer = [0u8; CHUNK_SIZE];
    let mut total_bytes = 0usize;

    loop {
        let n = match response.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                update.abort()?;
                return Err(e).context("failed to read firmware data");
            }
        };

        if let Err(e) = update.write(&buffer[..n]) {
            update.abort()?;
            return Err(e).context("failed to write firmware data");
        }

        total_bytes += n;
    }

    if total_bytes == 0 {
        update.abort()?;
        anyhow::bail!("firmware download was empty");
    }

    info!("downloaded {} bytes of firmware", total_bytes);

    let finished = update.finish()?;
    finished.activate()?;

    info!("firmware activated, rebooting");

    Ok(())
}
