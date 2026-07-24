use anyhow::{Context, Result};
use esp_idf_svc::http::{
    client::{Configuration, EspHttpConnection},
    Method,
};
use embedded_svc::io::Write;
use log::info;
use serde::{Deserialize, Serialize};

const API_KEY: &str = env!("HOME_GATEWAY_API_KEY");
const DEVICE_NAME: &str = env!("DEVICE_NAME");

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u64>,
    pub image_url: Option<String>,
    pub clear_screen: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ConfigRequest {
    device_id: String,
    device_name: &'static str,
    battery_voltage: Option<f32>,
}

fn device_id() -> String {
    let mut mac = [0u8; 6];
    unsafe {
        esp_idf_sys::esp_read_mac(mac.as_mut_ptr(), esp_idf_sys::esp_mac_type_t_ESP_MAC_WIFI_STA);
    }
    let id = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    info!("device id (address): {}", id);
    id
}

pub fn fetch_config(battery_voltage: Option<f32>) -> Result<EpdConfig> {
    #[cfg(not(debug_assertions))]
    let url = "https://home.anurag.sh/v1/epd/config";
    #[cfg(debug_assertions)]
    let url = "http://192.168.0.104:8000/v1/epd/config";
    info!("Fetching config from {}...", url);

    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&config)?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);

    let payload = serde_json::to_vec(&ConfigRequest {
        device_id: device_id(),
        device_name: DEVICE_NAME,
        battery_voltage,
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
    info!("Response status: {}", status);

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
    info!("Fetched config: {:?}", config);

    Ok(config)
}

pub fn fetch_image(url: &str, buffer: &mut [u8]) -> Result<()> {
    info!("Fetching image from {}...", url);

    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&config)?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);

    let headers = vec![("X-Api-Key", API_KEY)];
    let request = client.request(Method::Get, url, &headers)?;
    let response = request.submit()?;

    let status = response.status();
    info!("Response status: {}", status);

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

    info!("Fetched {} bytes of image data", total_bytes);

    if total_bytes < buffer.len() {
        log::warn!(
            "Image data size ({}) is smaller than buffer size ({})",
            total_bytes,
            buffer.len()
        );
    }

    Ok(())
}
