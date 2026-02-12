use anyhow::Result;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{IOPin, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_sys::{esp_deep_sleep_start, esp_sleep_enable_timer_wakeup};

mod driver;
mod http_client;
mod wifi;
use driver::Gdep133c02;

use crate::driver::EPD_IMAGE_FULL_BUFFER_SIZE;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    match run_task() {
        Ok(time_to_sleep) => deep_sleep(time_to_sleep),
        Err(e) => {
            log::error!("error in task: {e}");
            deep_sleep(15);
        }
    }

    Ok(())
}

fn deep_sleep(mins: u64) {
    log::info!("sleeping for {mins} mins");
    unsafe {
        esp_sleep_enable_timer_wakeup(mins * 60 * 1_000_000);
        esp_deep_sleep_start();
    }
}

fn run_task() -> Result<u64, anyhow::Error> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    let mut load_sw = PinDriver::output(pins.gpio45.downgrade())?;
    load_sw.set_high()?;

    let mut epd_buffer = vec![0u8; EPD_IMAGE_FULL_BUFFER_SIZE];

    let mut display = Gdep133c02::new(
        peripherals.spi3,
        pins.gpio9,
        pins.gpio41,
        Some(pins.gpio40),
        pins.gpio18.downgrade(),
        pins.gpio17.downgrade(),
        pins.gpio6.downgrade(),
        pins.gpio7.downgrade(),
    )?;

    let wifi = wifi::try_connect(peripherals.modem, sys_loop, Some(nvs))?;

    let mut refresh_time_in_mins = 15;

    if wifi.is_connected()? {
        log::info!("Wifi connected, fetching config...");
        let config = http_client::fetch_config()?;
        log::info!("Config: {:?}", config);

        if let Some(refresh_time) = config.refresh_interval_mins {
            refresh_time_in_mins = refresh_time;
        }

        if let Some(clear_screen) = config.clear_screen {
            if clear_screen {
                log::info!("Initializing Display...");

                display.init_epd()?;

                if let Ok(status) = display.check_driver_ic_status() {
                    if status {
                        log::info!("Driver IC check passed.");
                    } else {
                        log::error!("Driver IC check failed!");
                    }
                } else {
                    log::error!("Driver IC check error!");
                }

                display.hardware_reset()?;
                display.set_cs_all(true)?;

                log::info!("Display White");
                display.init_epd()?;
                display.display_color(driver::EPD_WHITE, &mut epd_buffer)?;
                return Ok(refresh_time_in_mins);
            }
        }

        if let Some(url) = config.image_url {
            match http_client::fetch_image(&url, &mut epd_buffer) {
                Ok(_) => {
                    log::info!("Image fetched successfully");

                    display.init_epd()?;

                    if let Ok(status) = display.check_driver_ic_status() {
                        if status {
                            log::info!("Driver IC check passed.");
                        } else {
                            log::error!("Driver IC check failed!");
                        }
                    } else {
                        log::error!("Driver IC check error!");
                    }

                    display.hardware_reset()?;
                    display.set_cs_all(true)?;

                    display.init_epd()?;
                    display.display_buffer(&epd_buffer)?;
                }
                Err(e) => log::error!("Failed to fetch image: {:?}", e),
            }
        }
    }

    // run_epd_test(epd_buffer, display)?;

    Ok(refresh_time_in_mins)
}

#[allow(unused)]
fn run_epd_test(mut epd_buffer: Vec<u8>, mut display: Gdep133c02<'_>) -> Result<(), anyhow::Error> {
    log::info!("Initializing Display...");

    display.init_epd()?;

    if let Ok(status) = display.check_driver_ic_status() {
        if status {
            log::info!("Driver IC check passed.");
        } else {
            log::error!("Driver IC check failed!");
        }
    } else {
        log::error!("Driver IC check error!");
    }

    display.hardware_reset()?;
    display.set_cs_all(true)?;

    log::info!("Display Color Bar");
    display.init_epd()?;
    display.display_color_bar(&mut epd_buffer)?;
    FreeRtos::delay_ms(2000);

    log::info!("Display Checkerboard");
    display.init_epd()?;
    match display.draw_checkerboard() {
        Ok(_) => log::info!("Checkerboard displayed."),
        Err(e) => log::error!("Checkerboard failed: {}", e),
    }
    FreeRtos::delay_ms(2000);

    log::info!("Display White");
    display.init_epd()?;
    display.display_color(driver::EPD_WHITE, &mut epd_buffer)?;
    FreeRtos::delay_ms(2000);

    Ok(())
}
