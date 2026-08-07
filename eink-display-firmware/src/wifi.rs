use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripheral;
use esp_idf_svc::ipv4;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::info;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::net_cache::{self, NetCache};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");
const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);

static NETIF_SEQUENCE: AtomicU32 = AtomicU32::new(0);

pub struct Session {
    wifi: BlockingWifi<EspWifi<'static>>,
    cached: bool,
}

impl Session {
    pub fn cached(&self) -> bool {
        self.cached
    }

    pub fn stop(&mut self) -> Result<()> {
        self.wifi.stop()?;

        Ok(())
    }
}

pub fn connect(
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
) -> Result<Session> {
    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    let cache = net_cache::take();

    if let Some(cache) = cache {
        match associate(&mut wifi, Some(&cache)) {
            Ok(_) => {
                info!("associated using the cached lease");

                return Ok(Session { wifi, cached: true });
            }
            Err(e) => {
                log::warn!("cached association failed, falling back to dhcp: {e}");
                net_cache::invalidate();
            }
        }
    }

    associate(&mut wifi, None)?;
    cache_association(&wifi);

    Ok(Session {
        wifi,
        cached: false,
    })
}

pub fn reassociate_with_dhcp(session: &mut Session) -> Result<()> {
    net_cache::invalidate();

    if let Err(e) = session.wifi.stop() {
        log::warn!("failed to stop wifi before reassociating: {e}");
    }

    associate(&mut session.wifi, None)?;
    cache_association(&session.wifi);
    session.cached = false;

    Ok(())
}

fn associate(wifi: &mut BlockingWifi<EspWifi<'static>>, cache: Option<&NetCache>) -> Result<()> {
    let ip_configuration = match cache {
        Some(cache) => ipv4::Configuration::Client(ipv4::ClientConfiguration::Fixed(
            ipv4::ClientSettings {
                ip: cache.ip(),
                subnet: cache.subnet(),
                dns: cache.dns(),
                secondary_dns: None,
            },
        )),
        None => ipv4::Configuration::Client(ipv4::ClientConfiguration::DHCP(Default::default())),
    };

    let key = format!("WIFI_STA_{}", NETIF_SEQUENCE.fetch_add(1, Ordering::Relaxed));

    let netif_configuration = NetifConfiguration {
        key: key.as_str().try_into().unwrap(),
        ip_configuration: Some(ip_configuration),
        ..NetifConfiguration::wifi_default_client()
    };

    wifi.wifi_mut()
        .swap_netif_sta(EspNetif::new_with_conf(&netif_configuration)?)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        bssid: cache.map(|cache| cache.bssid()),
        channel: cache.map(|cache| cache.channel()),
        ..Default::default()
    }))?;

    info!("starting wifi...");
    wifi.start()?;

    info!("connecting wifi...");
    wifi.wifi_mut().connect()?;
    wifi.wifi_wait_while(
        || wifi.wifi().is_connected().map(|connected| !connected),
        Some(CONNECT_TIMEOUT),
    )?;

    wifi.ip_wait_while(|| wifi.is_up().map(|up| !up), Some(CONNECT_TIMEOUT))?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("wifi ip info: {ip_info:?}");

    if !wifi.is_connected()? {
        anyhow::bail!("wifi did not connect");
    }

    Ok(())
}

fn cache_association(wifi: &BlockingWifi<EspWifi<'static>>) {
    let ip_info = match wifi.wifi().sta_netif().get_ip_info() {
        Ok(ip_info) => ip_info,
        Err(e) => {
            log::warn!("failed to read ip info: {e}");
            return;
        }
    };

    let Some((bssid, channel)) = associated_ap() else {
        return;
    };

    net_cache::store(bssid, channel, &ip_info);
}

fn associated_ap() -> Option<([u8; 6], u8)> {
    let mut record = esp_idf_sys::wifi_ap_record_t::default();

    let status = unsafe { esp_idf_sys::esp_wifi_sta_get_ap_info(&mut record) };

    if status != esp_idf_sys::ESP_OK {
        log::warn!("failed to read ap info: {status}");
        return None;
    }

    Some((record.bssid, record.primary))
}
