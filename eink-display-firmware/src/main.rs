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
mod ota;
mod panel_power;
mod refresh;
mod wifi;
use driver::Gdep133c02;
use panel_power::PanelPower;
use refresh::Refresh;

use crate::driver::EPD_IMAGE_FULL_BUFFER_SIZE;

const DEFAULT_REFRESH_MINS: u64 = 15;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    match run_task() {
        Ok(time_to_sleep) => deep_sleep(time_to_sleep),
        Err(e) => {
            log::error!("error in task: {e}");
            deep_sleep(DEFAULT_REFRESH_MINS);
        }
    }

    Ok(())
}

fn deep_sleep(mins: u64) {
    log::info!("sleeping for {mins} mins");
    unsafe {
        gpio_deep_sleep_hold_en();
        esp_sleep_enable_timer_wakeup(mins * 60 * 1_000_000);
        esp_deep_sleep_start();
    }
}

fn run_task() -> Result<u64, anyhow::Error> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    let _panel_power = PanelPower::enable(pins.gpio43.downgrade())?;

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

    let mut epd_buffer = vec![0u8; EPD_IMAGE_FULL_BUFFER_SIZE];

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

    let mut wifi = wifi::try_connect(peripherals.modem, sys_loop, Some(nvs))?;

    let (refresh, refresh_time_in_mins) = if wifi.is_connected()? {
        log::info!("wifi connected, fetching config...");
        let config = http_client::fetch_config(battery_voltage, is_charging)?;
        log::info!("config: {:?}", config);

        if let Err(e) = ota::mark_valid() {
            log::warn!("failed to mark running slot valid: {e}");
        }

        if let Some(url) = &config.firmware_url {
            log::info!(
                "firmware update available: {} -> {}",
                http_client::FIRMWARE_VERSION,
                config.firmware_version.as_deref().unwrap_or("unknown")
            );

            match ota::apply(url) {
                Ok(_) => unsafe { esp_restart() },
                Err(e) => log::error!("firmware update failed: {:?}", e),
            }
        }

        let refresh = if config.clear_screen == Some(true) {
            Refresh::Clear
        } else if let Some(url) = config.image_url {
            match http_client::fetch_image(&url, &mut epd_buffer) {
                Ok(_) => {
                    log::info!("image fetched successfully");
                    Refresh::Image
                }
                Err(e) => {
                    log::error!("failed to fetch image: {:?}", e);
                    Refresh::None
                }
            }
        } else {
            Refresh::None
        };

        (
            refresh,
            config.refresh_interval_mins.unwrap_or(DEFAULT_REFRESH_MINS),
        )
    } else {
        (Refresh::None, DEFAULT_REFRESH_MINS)
    };

    wifi.stop()?;
    drop(wifi);
    log::info!("wifi stopped");

    match refresh {
        Refresh::None => {
            log::info!("nothing to refresh, skipping display");
        }

        Refresh::Clear => {
            log::info!("clearing display to white");

            display.init_epd()?;

            display.hardware_reset()?;
            display.set_cs_all(true)?;

            display.init_epd()?;
            display.display_color(driver::EPD_WHITE, &mut epd_buffer)?;
        }

        Refresh::Image => {
            log::info!("rendering image to display");

            display.init_epd()?;

            display.hardware_reset()?;
            display.set_cs_all(true)?;

            display.init_epd()?;
            display.display_buffer(&epd_buffer)?;
        }
    }

    Ok(refresh_time_in_mins)
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
