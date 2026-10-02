use anyhow::Result;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;
use std::time::Duration;

use crate::grpc_web;
use crate::proto::{WakeRequest, WakeResponse};

const API_KEY: &str = env!("HOME_GATEWAY_API_KEY");
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const WAKE_PATH: &str = "/v1/grpc/home_gateway.eink.v1.EinkDisplay/Wake";
pub const FIRMWARE_VERSION: &str = env!("FIRMWARE_VERSION");

#[cfg(not(debug_assertions))]
const HOST: &str = "https://home.anurag.sh";
#[cfg(debug_assertions)]
const HOST: &str = "http://192.168.0.149:8000";

pub type HttpClient = embedded_svc::http::client::Client<EspHttpConnection>;

pub fn client() -> Result<HttpClient> {
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

pub fn wake(
    client: &mut HttpClient,
    battery_voltage: Option<f32>,
    is_charging: bool,
    previous_refresh_failed: bool,
) -> Result<WakeResponse> {
    let url = format!("{HOST}{WAKE_PATH}");
    info!("waking against {}...", url);

    let request = WakeRequest {
        device_id: device_id(),
        battery_voltage,
        is_charging,
        battery_chemistry: crate::battery::CHEMISTRY.to_owned(),
        battery_kind: crate::battery::KIND.to_owned(),
        firmware_version: FIRMWARE_VERSION.to_owned(),
        previous_refresh_failed,
    };

    let response: WakeResponse = grpc_web::unary(client, &url, API_KEY, &request)?;

    info!(
        "wake answered: sleep {} secs, firmware {:?}, refresh {}",
        response.sleep_secs,
        response.firmware,
        crate::refresh::describe(response.refresh.as_ref())
    );

    Ok(response)
}
