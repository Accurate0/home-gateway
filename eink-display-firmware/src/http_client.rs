use anyhow::{Context, Result};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EpdConfig {
    pub refresh_interval_mins: Option<u32>,
    pub image_url: Option<String>,
}

pub fn fetch_config() -> Result<EpdConfig> {
    let url = "https://home.anurag.sh/v1/epd/config";
    info!("Fetching config from {}...", url);

    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&config)?;
    let mut client = embedded_svc::http::client::Client::wrap(connection);

    let request = client.get(url)?;
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
