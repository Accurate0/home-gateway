use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripheral;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::info;
use std::time::Duration;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");
const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);

pub fn try_connect(
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), nvs)?;

    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    info!("Starting wifi...");
    wifi.start()?;

    info!("Connecting wifi...");
    wifi.wifi_mut().connect()?;
    wifi.wifi_wait_while(
        || wifi.wifi().is_connected().map(|connected| !connected),
        Some(CONNECT_TIMEOUT),
    )?;

    info!("Waiting for DHCP lease...");
    wifi.ip_wait_while(
        || wifi.is_up().map(|up| !up),
        Some(CONNECT_TIMEOUT),
    )?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(wifi)
}
