use anyhow::Result;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, IOPin};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_sys::{
    esp_deep_sleep_start, esp_restart, esp_sleep_enable_timer_wakeup, gpio_deep_sleep_hold_en,
};

mod battery;
mod driver;
mod http_client;
mod image_hash;
mod net_cache;
mod ota;
mod panel_power;
mod refresh;
mod watchdog;
mod wifi;
use driver::Gdep133c02;
use image_hash::ImageHashStore;
use panel_power::PanelPower;
use refresh::Refresh;

use crate::driver::EPD_IMAGE_FULL_BUFFER_SIZE;

const DEFAULT_REFRESH_MINS: u64 = 15;
const MIN_REFRESH_SECS: u64 = 60;
const MAX_REFRESH_SECS: u64 = 1440 * 60;
const LOW_BATTERY_SLEEP_MINS: u64 = 360;
const CRITICAL_BATTERY_SLEEP_MINS: u64 = 1440;
const MAX_BACKOFF_SHIFT: u32 = 3;

#[link_section = ".rtc.data"]
static mut CONSECUTIVE_FAILURES: u32 = 0;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    watchdog::start();

    let time_to_sleep = match run_task() {
        Ok(time_to_sleep) => {
            unsafe { CONSECUTIVE_FAILURES = 0 };
            time_to_sleep
        }
        Err(e) => {
            log::error!("error in task: {e}");
            backoff_secs()
        }
    };

    watchdog::stop();
    deep_sleep(time_to_sleep);

    Ok(())
}

fn backoff_secs() -> u64 {
    let failures = unsafe {
        CONSECUTIVE_FAILURES = CONSECUTIVE_FAILURES.saturating_add(1);
        CONSECUTIVE_FAILURES
    };

    let shift = (failures - 1).min(MAX_BACKOFF_SHIFT);
    let mins = DEFAULT_REFRESH_MINS << shift;

    log::warn!("{failures} consecutive failures, backing off to {mins} mins");

    mins * 60
}

fn deep_sleep(secs: u64) {
    log::info!("sleeping for {secs} secs");
    unsafe {
        gpio_deep_sleep_hold_en();
        esp_sleep_enable_timer_wakeup(secs * 1_000_000);
        esp_deep_sleep_start();
    }
}

fn run_task() -> Result<u64, anyhow::Error> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    let battery_voltage = match battery::read_voltage(peripherals.adc1, pins.gpio1, pins.gpio6) {
        Ok(v) => {
            log::info!("battery voltage: {v:.2}V");
            Some(v)
        }
        Err(e) => {
            log::warn!("failed to read battery voltage: {e}");
            None
        }
    };

    let is_charging = battery::is_charging();
    log::info!("charging: {is_charging}");

    if let (Some(voltage), false) = (battery_voltage, is_charging) {
        if voltage < battery::CRITICAL_VOLTAGE_CUTOFF {
            log::warn!(
                "battery critically low ({voltage:.2}V), skipping cycle for {CRITICAL_BATTERY_SLEEP_MINS} mins"
            );
            return Ok(CRITICAL_BATTERY_SLEEP_MINS * 60);
        }

        if voltage < battery::LOW_VOLTAGE_CUTOFF {
            log::warn!(
                "battery low ({voltage:.2}V), skipping cycle for {LOW_BATTERY_SLEEP_MINS} mins"
            );
            return Ok(LOW_BATTERY_SLEEP_MINS * 60);
        }
    }

    let mut epd_buffer = vec![0u8; EPD_IMAGE_FULL_BUFFER_SIZE];

    let mut ota_attempts = match ota::AttemptTracker::new(nvs.clone()) {
        Ok(tracker) => Some(tracker),
        Err(e) => {
            log::warn!("failed to open ota attempt store: {e}");
            None
        }
    };

    let mut image_hashes = match ImageHashStore::new(nvs.clone()) {
        Ok(store) => Some(store),
        Err(e) => {
            log::warn!("failed to open image hash store: {e}");
            None
        }
    };

    let mut wifi = wifi::connect(peripherals.modem, sys_loop, Some(nvs))?;

    watchdog::feed();

    let stored_hash = image_hashes.as_ref().and_then(|store| store.stored());

    log::info!("wifi connected, fetching config...");
    let mut client = http_client::client()?;

    let config = match http_client::fetch_config(
        &mut client,
        battery_voltage,
        is_charging,
        stored_hash.clone(),
    ) {
        Ok(config) => config,
        Err(e) if wifi.cached() => {
            log::warn!("config fetch failed on the cached lease, retrying over dhcp: {e:?}");

            wifi::reassociate_with_dhcp(&mut wifi)?;
            client = http_client::client()?;

            http_client::fetch_config(
                &mut client,
                battery_voltage,
                is_charging,
                stored_hash.clone(),
            )?
        }
        Err(e) => return Err(e),
    };

    watchdog::feed();

    if let Err(e) = ota::mark_valid() {
        log::warn!("failed to mark running slot valid: {e}");
    } else if let Some(tracker) = ota_attempts.as_mut() {
        tracker.confirm(http_client::FIRMWARE_VERSION);
    }

    if let Some(url) = &config.firmware_url {
        let target = config.firmware_version.as_deref().unwrap_or("unknown");

        log::info!(
            "firmware update available: {} -> {}",
            http_client::FIRMWARE_VERSION,
            target
        );

        let allowed = ota_attempts
            .as_ref()
            .map(|tracker| tracker.should_attempt(target))
            .unwrap_or(true);

        if allowed {
            if let Some(tracker) = ota_attempts.as_mut() {
                tracker.record_attempt(target);
            }

            match ota::apply(&mut client, url) {
                Ok(_) => unsafe { esp_restart() },
                Err(e) => log::error!("firmware update failed: {:?}", e),
            }
        }
    }

    let unchanged = match (&config.image_hash, &stored_hash) {
        (Some(hash), Some(stored)) => hash == stored,
        _ => false,
    };

    let refresh = if config.clear_screen == Some(true) {
        Some(Refresh::Clear)
    } else if unchanged {
        log::info!("image unchanged, skipping download and refresh");
        None
    } else if let (Some(window), Some(url)) = (config.partial, config.image_url.as_ref()) {
        log::info!(
            "partial refresh requested: x={} y={} w={} h={}",
            window.x,
            window.y,
            window.width,
            window.height
        );

        let size = window.buffer_size();

        if size > epd_buffer.len() {
            log::error!("partial window {size} bytes exceeds the frame buffer");
            None
        } else {
            match http_client::fetch_image(&mut client, url, &mut epd_buffer[..size]) {
                Ok(_) => Some(Refresh::Partial {
                    hash: config.image_hash,
                    window,
                }),
                Err(e) => {
                    log::error!("failed to fetch partial image: {:?}", e);
                    None
                }
            }
        }
    } else if let Some(url) = config.image_url {
        match http_client::fetch_image(&mut client, &url, &mut epd_buffer) {
            Ok(_) => {
                log::info!("image fetched successfully");
                Some(Refresh::Image(config.image_hash))
            }
            Err(e) => {
                log::error!("failed to fetch image: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    watchdog::feed();

    let refresh_time_in_secs = config
        .refresh_interval_secs
        .or_else(|| config.refresh_interval_mins.map(|mins| mins * 60))
        .unwrap_or(DEFAULT_REFRESH_MINS * 60)
        .clamp(MIN_REFRESH_SECS, MAX_REFRESH_SECS);

    wifi.stop()?;
    drop(wifi);
    log::info!("wifi stopped");

    watchdog::feed();

    let Some(refresh) = refresh else {
        log::info!("nothing to refresh, leaving the panel unpowered");

        return Ok(refresh_time_in_secs);
    };

    let _panel_power = PanelPower::enable(pins.gpio43.downgrade())?;

    let mut display = Gdep133c02::new(
        peripherals.spi3,
        pins.gpio7,
        pins.gpio9,
        Option::<AnyIOPin>::None,
        pins.gpio44.downgrade(),
        pins.gpio41.downgrade(),
        pins.gpio10.downgrade(),
        pins.gpio38.downgrade(),
        pins.gpio4.downgrade(),
    )?;

    display.init_epd()?;

    display.hardware_reset()?;
    display.set_cs_all(true)?;

    display.init_epd()?;

    match refresh {
        Refresh::Clear => {
            log::info!("clearing display to white");

            display.display_color(driver::EPD_WHITE, &mut epd_buffer)?;

            if let Some(store) = image_hashes.as_mut() {
                store.clear();
            }
        }

        Refresh::Image(hash) => {
            log::info!("rendering image to display");

            display.display_buffer(&epd_buffer)?;

            if let (Some(store), Some(hash)) = (image_hashes.as_mut(), hash) {
                store.store(&hash);
            }
        }

        Refresh::Partial { hash, window } => {
            log::info!("rendering partial window to display");

            let region = &epd_buffer[..window.buffer_size()];

            match display.display_partial(region, window.x, window.y, window.width, window.height) {
                Ok(_) => {
                    if let (Some(store), Some(hash)) = (image_hashes.as_mut(), hash) {
                        store.store(&hash);
                    }
                }
                Err(e) => log::error!("partial refresh failed: {:?}", e),
            }
        }
    }

    Ok(refresh_time_in_secs)
}

#[allow(unused)]
fn run_epd_test(mut epd_buffer: Vec<u8>, mut display: Gdep133c02<'_>) -> Result<(), anyhow::Error> {
    log::info!("initializing display...");

    display.init_epd()?;

    display.hardware_reset()?;
    display.set_cs_all(true)?;

    log::info!("display color bar");
    display.init_epd()?;
    display.display_color_bar(&mut epd_buffer)?;
    FreeRtos::delay_ms(2000);

    log::info!("display checkerboard");
    display.init_epd()?;
    match display.draw_checkerboard() {
        Ok(_) => log::info!("checkerboard displayed"),
        Err(e) => log::error!("checkerboard failed: {}", e),
    }
    FreeRtos::delay_ms(2000);

    log::info!("display white");
    display.init_epd()?;
    display.display_color(driver::EPD_WHITE, &mut epd_buffer)?;
    FreeRtos::delay_ms(2000);

    Ok(())
}
