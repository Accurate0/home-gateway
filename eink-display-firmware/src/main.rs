use anyhow::Result;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, IOPin, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi::{config::DriverConfig, Dma, SpiDeviceDriver};
use esp_idf_svc::log::EspLogger;

mod driver;
use driver::{Gdep133c02, EPD_IMAGE_DATA_BUFFER_SIZE};

fn main() -> Result<()> {
    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();

    log::info!("Starting Rust E-Ink Driver...");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // Initialize SPI
    // MOSI: GPIO 41
    // MISO: GPIO 40
    // SCLK: GPIO 9
    // CS0: GPIO 18 (Manual)
    // CS1: GPIO 17 (Manual)

    let spi = peripherals.spi3;
    let sclk = pins.gpio9;
    let mosi = pins.gpio41;
    let miso = pins.gpio40;

    // CS Pins controlled manually
    // Use downgrade() to get AnyIOPin for flexible struct type
    let mut cs0 = PinDriver::output(pins.gpio18.downgrade())?;
    let mut cs1 = PinDriver::output(pins.gpio17.downgrade())?;

    // Set initial state
    cs0.set_high()?;
    cs1.set_high()?;

    let driver_config = DriverConfig::default().dma(Dma::Auto(4096));
    let spi_config = esp_idf_svc::hal::spi::config::Config::new().baudrate(10.MHz().into());

    // Configure SPI Device Driver (No CS managed by driver)
    let spi_driver = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Some(miso),
        Option::<AnyIOPin>::None,
        &driver_config,
        &spi_config,
    )?;

    // Other Pins
    let rst = PinDriver::output(pins.gpio6.downgrade())?;
    let mut busy = PinDriver::input(pins.gpio7.downgrade())?;
    busy.set_pull(Pull::Floating)?;

    let mut load_sw = PinDriver::output(pins.gpio45.downgrade())?;
    load_sw.set_high()?;

    // Buffer for EPD
    // Using Vec. Ensure PSRAM is enabled if internal RAM is insufficient for large buffers.
    // EPD_IMAGE_DATA_BUFFER_SIZE is 8192 bytes, which fits in internal RAM.
    let mut epd_buffer = vec![0u8; EPD_IMAGE_DATA_BUFFER_SIZE];

    let mut display = Gdep133c02::new(spi_driver, cs0, cs1, rst, busy);

    log::info!("Initializing Display...");
    display.init_epd()?;

    // Check Driver Status
    if let Ok(status) = display.check_driver_ic_status() {
        if status {
            log::info!("Driver IC check passed.");
        } else {
            log::error!("Driver IC check failed!");
        }
    } else {
        log::error!("Driver IC check error!");
    }

    // Clear / Reset
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

    FreeRtos::delay_ms(1000);

    loop {
        FreeRtos::delay_ms(10);
    }
}
