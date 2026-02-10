use anyhow::Result;
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::{AnyIOPin, Input, Level, Output, PinDriver, Pull};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{config::Config as SpiConfig, config::DriverConfig, Dma, SpiDeviceDriver, SpiDriver};

pub const EPD_BLACK: u8 = 0x00;
pub const EPD_WHITE: u8 = 0x11;
pub const EPD_YELLOW: u8 = 0x22;
pub const EPD_RED: u8 = 0x33;
pub const EPD_BLUE: u8 = 0x55;
pub const EPD_GREEN: u8 = 0x66;

const EPD_PSR: u8 = 0x00;
const EPD_PWR: u8 = 0x01;
const EPD_POF: u8 = 0x02;
const EPD_PON: u8 = 0x04;
const EPD_BTST_N: u8 = 0x05;
const EPD_BTST_P: u8 = 0x06;
const EPD_DTM: u8 = 0x10;
const EPD_DRF: u8 = 0x12;
const EPD_CDI: u8 = 0x50;
const EPD_TCON: u8 = 0x60;
const EPD_TRES: u8 = 0x61;
#[allow(dead_code)]
const EPD_PTLW: u8 = 0x83;
const EPD_AN_TM: u8 = 0x74;
const EPD_AGID: u8 = 0x86;
const EPD_BUCK_BOOST_VDDN: u8 = 0xB0;
const EPD_TFT_VCOM_POWER: u8 = 0xB1;
const EPD_EN_BUF: u8 = 0xB6;
const EPD_BOOST_VDDP_EN: u8 = 0xB7;
const EPD_CCSET: u8 = 0xE0;
const EPD_PWS: u8 = 0xE3;
const EPD_CMD66: u8 = 0xF0;

pub const EPD_IMAGE_DATA_BUFFER_SIZE: usize = 8192;
pub const EPD_WIDTH: u32 = 1200;
pub const EPD_HEIGHT: u32 = 1600;

const PSR_V: [u8; 2] = [0xDF, 0x69];
const PWR_V: [u8; 6] = [0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38];
const POF_V: [u8; 1] = [0x00];
const DRF_V: [u8; 1] = [0x01];
const CDI_V: [u8; 1] = [0xF7];
const TCON_V: [u8; 2] = [0x03, 0x03];
const TRES_V: [u8; 4] = [0x04, 0xB0, 0x03, 0x20];
const CMD66_V: [u8; 6] = [0x49, 0x55, 0x13, 0x5D, 0x05, 0x10];
const EN_BUF_V: [u8; 1] = [0x07];
const CCSET_V: [u8; 1] = [0x01];
const PWS_V: [u8; 1] = [0x22];
const AN_TM_V: [u8; 9] = [0xC0, 0x1C, 0x1C, 0xCC, 0xCC, 0xCC, 0x15, 0x15, 0x55];
const AGID_V: [u8; 1] = [0x10];
const BTST_P_V: [u8; 2] = [0xE8, 0x28];
const BOOST_VDDP_EN_V: [u8; 1] = [0x01];
const BTST_N_V: [u8; 2] = [0xE8, 0x28];
const BUCK_BOOST_VDDN_V: [u8; 1] = [0x01];
const TFT_VCOM_POWER_V: [u8; 1] = [0x02];

pub struct Gdep133c02<'a> {
    spi: SpiDeviceDriver<'a, SpiDriver<'a>>,
    cs0: PinDriver<'a, AnyIOPin, Output>,
    cs1: PinDriver<'a, AnyIOPin, Output>,
    rst: PinDriver<'a, AnyIOPin, Output>,
    busy: PinDriver<'a, AnyIOPin, Input>,
    delay: Delay,
}

impl<'a> Gdep133c02<'a> {
    pub fn new(
        spi: impl Peripheral<P = impl esp_idf_hal::spi::SpiAnyPins> + 'a,
        sclk: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'a,
        mosi: impl Peripheral<P = impl esp_idf_hal::gpio::OutputPin> + 'a,
        miso: Option<impl Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin> + 'a>,
        cs0: AnyIOPin,
        cs1: AnyIOPin,
        rst: AnyIOPin,
        busy: AnyIOPin,
    ) -> Result<Self> {
        let driver_config = DriverConfig::default().dma(Dma::Auto(4096));
        let spi_config = SpiConfig::new().baudrate(10.MHz().into());

        let spi_driver = SpiDeviceDriver::new_single(
            spi,
            sclk,
            mosi,
            miso,
            Option::<AnyIOPin>::None,
            &driver_config,
            &spi_config,
        )?;

        let mut cs0 = PinDriver::output(cs0)?;
        let mut cs1 = PinDriver::output(cs1)?;
        cs0.set_high()?;
        cs1.set_high()?;

        let rst = PinDriver::output(rst)?;
        let mut busy = PinDriver::input(busy)?;
        busy.set_pull(Pull::Floating)?;

        Ok(Self {
            spi: spi_driver,
            cs0,
            cs1,
            rst,
            busy,
            delay: Delay::new_default(),
        })
    }

    fn delay_ms(&self, ms: u32) {
        self.delay.delay_ms(ms);
    }

    pub fn set_cs_all(&mut self, level: bool) -> Result<()> {
        let l = if level { Level::High } else { Level::Low };
        self.cs0.set_level(l)?;
        self.cs1.set_level(l)?;
        Ok(())
    }

    fn set_cs(&mut self, cs_idx: u8, level: bool) -> Result<()> {
        let l = if level { Level::High } else { Level::Low };
        match cs_idx {
            0 => self.cs0.set_level(l)?,
            1 => self.cs1.set_level(l)?,
            _ => {
                self.cs0.set_level(l)?;
                self.cs1.set_level(l)?;
            }
        }
        Ok(())
    }

    fn check_busy_high(&self) {
        while !self.busy.is_high() {
            self.delay_ms(10);
        }
    }

    fn write_command(&mut self, cmd: u8) -> Result<()> {
        self.spi
            .write(&[cmd])
            .map_err(|e| anyhow::anyhow!("SPI write error: {:?}", e))?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<()> {
        const CHUNK_SIZE: usize = 32768;
        for chunk in data.chunks(CHUNK_SIZE) {
            self.spi
                .write(chunk)
                .map_err(|e| anyhow::anyhow!("SPI write error: {:?}", e))?;
        }
        Ok(())
    }

    fn read_data(&mut self, cmd: u8, rx_buf: &mut [u8]) -> Result<()> {
        self.spi
            .write(&[cmd])
            .map_err(|e| anyhow::anyhow!("SPI write error: {:?}", e))?;
        self.spi
            .read(rx_buf)
            .map_err(|e| anyhow::anyhow!("SPI read error: {:?}", e))?;
        Ok(())
    }

    pub fn hardware_reset(&mut self) -> Result<()> {
        self.rst.set_low()?;
        self.delay_ms(20);
        self.rst.set_high()?;
        self.delay_ms(20);
        Ok(())
    }

    fn write_epd(&mut self, cmd: u8, data: &[u8]) -> Result<()> {
        self.write_command(cmd)?;
        if !data.is_empty() {
            self.write_data(data)?;
        }
        Ok(())
    }

    pub fn init_epd(&mut self) -> Result<()> {
        self.hardware_reset()?;
        self.check_busy_high();

        self.set_cs(0, false)?;
        self.write_epd(EPD_AN_TM, &AN_TM_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_CMD66, &CMD66_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_PSR, &PSR_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_CDI, &CDI_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_TCON, &TCON_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_AGID, &AGID_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_PWS, &PWS_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_CCSET, &CCSET_V)?;
        self.set_cs_all(true)?;

        self.set_cs_all(false)?;
        self.write_epd(EPD_TRES, &TRES_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_PWR, &PWR_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_EN_BUF, &EN_BUF_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_BTST_P, &BTST_P_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_BOOST_VDDP_EN, &BOOST_VDDP_EN_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_BTST_N, &BTST_N_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_BUCK_BOOST_VDDN, &BUCK_BOOST_VDDN_V)?;
        self.set_cs_all(true)?;

        self.set_cs(0, false)?;
        self.write_epd(EPD_TFT_VCOM_POWER, &TFT_VCOM_POWER_V)?;
        self.set_cs_all(true)?;

        log::info!("init_epd completed");
        Ok(())
    }

    pub fn check_driver_ic_status(&mut self) -> Result<bool> {
        let mut status = true;
        let mut data_buf = [0u8; 3];

        for i in 0..2 {
            self.set_cs(i, false)?;
            self.read_data(0xF2, &mut data_buf)?;
            self.set_cs(i, true)?;

            log::info!(
                "Driver IC [{}] = {:02X} {:02X} {:02X}",
                i,
                data_buf[0],
                data_buf[1],
                data_buf[2]
            );
            if (data_buf[0] & 0x01) == 0x01 {
                log::info!("Driver IC [{}] is ready.", i);
            } else {
                log::warn!("Driver IC [{}] did not reply.", i);
                status = false;
            }
        }
        Ok(status)
    }

    pub fn display(&mut self) -> Result<()> {
        log::info!("Write PON");
        self.set_cs_all(false)?;
        self.write_command(EPD_PON)?;
        self.check_busy_high();
        self.set_cs_all(true)?;

        log::info!("Write DRF");
        self.set_cs_all(false)?;
        self.delay_ms(30);
        self.write_epd(EPD_DRF, &DRF_V)?;
        self.check_busy_high();
        self.set_cs_all(true)?;

        log::info!("Write POF");
        self.set_cs_all(false)?;
        self.write_epd(EPD_POF, &POF_V)?;
        self.check_busy_high();
        self.set_cs_all(true)?;

        log::info!("Display Done!!");
        Ok(())
    }

    pub fn display_color(&mut self, color: u8, buffer: &mut [u8]) -> Result<()> {
        buffer.fill(color);

        self.set_cs_all(false)?;
        self.write_command(EPD_DTM)?;

        let total_bytes = 480000;
        let buf_len = buffer.len();

        let chunks = total_bytes / buf_len;
        let remainder = total_bytes % buf_len;

        for _ in 0..chunks {
            self.write_data(buffer)?;
        }
        if remainder > 0 {
            self.write_data(&buffer[0..remainder])?;
        }

        self.set_cs_all(true)?;
        self.display()?;
        log::info!("Display color complete.");
        Ok(())
    }

    pub fn display_color_bar(&mut self, buffer: &mut [u8]) -> Result<()> {
        let colors = [
            EPD_BLACK, EPD_WHITE, EPD_YELLOW, EPD_RED, EPD_BLUE, EPD_GREEN,
        ];
        let bytes_per_color = 80000;
        let buf_len = buffer.len();

        self.set_cs_all(false)?;
        self.write_command(EPD_DTM)?;

        for color in colors {
            buffer.fill(color);
            let chunks = bytes_per_color / buf_len;
            let remainder = bytes_per_color % buf_len;

            for _ in 0..chunks {
                self.write_data(buffer)?;
            }
            if remainder > 0 {
                self.write_data(&buffer[0..remainder])?;
            }
        }

        self.set_cs_all(true)?;
        self.display()?;
        log::info!("Display color bar complete.");
        Ok(())
    }

    pub fn pic_display_test(&mut self, image: &[u8]) -> Result<()> {
        if image.len() != 960000 {
            log::error!("Image size mismatch: expected 960000, got {}", image.len());
            return Ok(());
        }

        let width_px = EPD_WIDTH;
        let width_bytes = if width_px % 2 == 0 {
            width_px / 2
        } else {
            width_px / 2 + 1
        };
        let width1_bytes = if width_bytes % 2 == 0 {
            width_bytes / 2
        } else {
            width_bytes / 2 + 1
        } as usize;
        let height = EPD_HEIGHT as usize;

        self.set_cs_all(true)?;
        self.set_cs(0, false)?;
        self.write_command(EPD_DTM)?;

        for i in 0..height {
            let offset = i * (width_bytes as usize);
            let row_data = &image[offset..offset + width1_bytes];
            self.write_data(row_data)?;
        }
        self.set_cs_all(true)?;

        self.set_cs(1, false)?;
        self.write_command(EPD_DTM)?;

        for i in 0..height {
            let offset = i * (width_bytes as usize) + width1_bytes;
            let row_data = &image[offset..offset + width1_bytes];
            self.write_data(row_data)?;
        }
        self.set_cs_all(true)?;

        self.display()?;
        self.delay_ms(10);
        log::info!("Rendering completed");
        Ok(())
    }

    pub fn draw_checkerboard(&mut self) -> Result<()> {
        let width_bytes = if EPD_WIDTH % 2 == 0 {
            EPD_WIDTH / 2
        } else {
            EPD_WIDTH / 2 + 1
        };
        let height = EPD_HEIGHT;
        let buffer_size = 960000;
        let mut num = vec![0u8; buffer_size];

        let colors = [
            EPD_BLACK, EPD_WHITE, EPD_YELLOW, EPD_RED, EPD_BLUE, EPD_GREEN,
        ];
        let grid_cols = 6;
        let grid_rows = 8;
        let cell_width = EPD_WIDTH / grid_cols;
        let cell_height = EPD_HEIGHT / grid_rows;

        for y in 0..height {
            for x in (0..EPD_WIDTH).step_by(2) {
                let grid_x = x / cell_width;
                let grid_y = y / cell_height;
                let color_index = (grid_x + grid_y) % 6;
                let color1 = colors[color_index as usize];

                let grid_x2 = (x + 1) / cell_width;
                let color_index2 = (grid_x2 + grid_y) % 6;
                let color2 = colors[color_index2 as usize];

                let new_x = EPD_WIDTH - 2 - x;
                let new_index = (y * width_bytes) + (new_x / 2);

                num[new_index as usize] = (color1 << 4) | color2;
            }
        }

        self.pic_display_test(&num)?;
        Ok(())
    }
}
