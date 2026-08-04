use anyhow::{Context, Result};
use embedded_svc::io::Write;
use esp_idf_svc::http::{
    client::{Configuration, EspHttpConnection},
    Method,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_KEY: &str = env!("HOME_GATEWAY_API_KEY");
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
pub const FIRMWARE_VERSION: &str = env!("FIRMWARE_VERSION");

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PartialWindow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PartialWindow {
    pub fn buffer_size(&self) -> usize {
        (self.width / 2) as usize * self.height as usize
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u64>,
    pub image_url: Option<String>,
    pub image_hash: Option<String>,
    pub clear_screen: Option<bool>,
    pub firmware_url: Option<String>,
    pub firmware_version: Option<String>,
    pub partial: Option<PartialWindow>,
}

#[derive(Debug, Serialize)]
struct ConfigRequest {
    device_id: String,
    battery_voltage: Option<f32>,
    is_charging: bool,
    battery_chemistry: &'static str,
    battery_kind: &'static str,
    firmware_version: &'static str,
    current_image_hash: Option<String>,
}

pub fn client() -> Result<embedded_svc::http::client::Client<EspHttpConnection>> {
    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        timeout: Some(HTTP_TIMEOUT),
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&config)?;

    Ok(embedded_svc::http::client::Client::wrap(connection))
}

pub fn api_key() -> &'static str {
    API_KEY
}

fn device_id() -> String {
    let mut mac = [0u8; 6];
    unsafe {
        esp_idf_sys::esp_read_mac(
            mac.as_mut_ptr(),
            esp_idf_sys::esp_mac_type_t_ESP_MAC_WIFI_STA,
        );
    }
    let id = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    info!("device id (address): {}", id);
    id
}

pub fn fetch_config(
    battery_voltage: Option<f32>,
    is_charging: bool,
    current_image_hash: Option<String>,
) -> Result<EpdConfig> {
    #[cfg(not(debug_assertions))]
    let url = "https://home.anurag.sh/v1/epd/config";
    #[cfg(debug_assertions)]
    let url = "http://192.168.0.149:8000/v1/epd/config";
    info!("fetching config from {}...", url);

    let mut client = client()?;

    let payload = serde_json::to_vec(&ConfigRequest {
        device_id: device_id(),
        battery_voltage,
        is_charging,
        battery_chemistry: crate::battery::CHEMISTRY,
        battery_kind: crate::battery::KIND,
        firmware_version: FIRMWARE_VERSION,
        current_image_hash,
    })?;
    let content_length = payload.len().to_string();

    let headers = [
        ("X-Api-Key", API_KEY),
        ("Content-Type", "application/json"),
        ("Content-Length", content_length.as_str()),
    ];
    let mut request = client.request(Method::Post, url, &headers)?;
    request.write_all(&payload)?;
    request.flush()?;
    let response = request.submit()?;

    let status = response.status();
    info!("response status: {}", status);

    let mut body = Vec::new();
    let mut buffer = [0u8; 1024];
    let mut reader = response;

    loop {
        let n = reader
            .read(&mut buffer)
            .context("Failed to read response")?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..n]);
    }

    if status != 200 {
        log::error!(
            "config request failed: status {} body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
        anyhow::bail!("Unexpected status code: {}", status);
    }

    let config: EpdConfig = serde_json::from_slice(&body)?;
    info!("fetched config: {:?}", config);

    Ok(config)
}

pub fn fetch_image(url: &str, buffer: &mut [u8]) -> Result<()> {
    info!("fetching image from {}...", url);

    let mut client = client()?;

    let headers = vec![("X-Api-Key", API_KEY)];
    let request = client.request(Method::Get, url, &headers)?;
    let response = request.submit()?;

    let status = response.status();
    info!("response status: {}", status);

    if status != 200 {
        let mut body = Vec::new();
        let mut err_buf = [0u8; 512];
        let mut reader = response;
        while let Ok(n) = reader.read(&mut err_buf) {
            if n == 0 {
                break;
            }
            body.extend_from_slice(&err_buf[..n]);
        }
        log::error!(
            "image request failed: status {} body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
        anyhow::bail!("Unexpected status code: {}", status);
    }

    let mut total_bytes = 0;
    let mut reader = response;

    loop {
        if total_bytes >= buffer.len() {
            break;
        }
        let n = reader
            .read(&mut buffer[total_bytes..])
            .context("Failed to read image data")?;
        if n == 0 {
            break;
        }
        total_bytes += n;
    }

    info!("fetched {} bytes of image data", total_bytes);

    if total_bytes != buffer.len() {
        anyhow::bail!(
            "incomplete image: got {} bytes, expected {}",
            total_bytes,
            buffer.len()
        );
    }

    Ok(())
}
