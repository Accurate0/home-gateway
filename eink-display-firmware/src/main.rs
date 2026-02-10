use anyhow::Result;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{IOPin, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_sys::{esp_deep_sleep_start, esp_sleep_enable_timer_wakeup};

mod driver;
mod http_client;
mod wifi;
use driver::{Gdep133c02, EPD_IMAGE_DATA_BUFFER_SIZE};

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;
    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    // EPD config
    let mut load_sw = PinDriver::output(pins.gpio45.downgrade())?;
    load_sw.set_high()?;
    let epd_buffer = vec![0u8; EPD_IMAGE_DATA_BUFFER_SIZE];

    let display = Gdep133c02::new(
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
    if wifi.is_connected()? {
        log::info!("Wifi connected, fetching config...");
        match http_client::fetch_config() {
            Ok(config) => log::info!("Config: {:?}", config),
            Err(e) => log::error!("HTTP Error: {:?}", e),
        }
    }

    run_epd_test(epd_buffer, display)?;

    log::info!("sleeping for 15 mins");
    unsafe {
        esp_sleep_enable_timer_wakeup(15 * 60 * 1_000_000);
        esp_deep_sleep_start();
    }
}

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
