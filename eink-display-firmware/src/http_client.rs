use anyhow::{Context, Result};
use esp_idf_svc::http::{
    client::{Configuration, EspHttpConnection},
    Method,
};
use log::info;
use serde::{Deserialize, Serialize};

const API_KEY: &str = env!("HOME_GATEWAY_API_KEY");

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u64>,
    pub image_url: Option<String>,
    pub clear_screen: Option<bool>,
}

pub fn fetch_config() -> Result<EpdConfig> {
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

    let headers = vec![("X-Api-Key", API_KEY)];
    let request = client.request(Method::Get, url, &headers)?;
    let response = request.submit()?;

    let status = response.status();
    info!("Response status: {}", status);

    if status != 200 {
        anyhow::bail!("Unexpected status code: {}", status);
    }

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
