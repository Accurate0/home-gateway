use anyhow::Result;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Gpio1, Gpio6, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;

const DIVIDER_RATIO: f32 = 2.0;
const SAMPLES: u32 = 10;

pub fn read_voltage(
    adc1: impl Peripheral<P = ADC1> + 'static,
    adc_pin: Gpio1,
    enable_pin: Gpio6,
) -> Result<f32> {
    let mut enable: PinDriver<'_, Gpio6, Output> = PinDriver::output(enable_pin)?;
    enable.set_high()?;
    FreeRtos::delay_ms(10);

    let adc = AdcDriver::new(adc1)?;
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut channel = AdcChannelDriver::new(&adc, adc_pin, &config)?;

    let mut sum: u32 = 0;
    for _ in 0..SAMPLES {
        sum += adc.read(&mut channel)? as u32;
        FreeRtos::delay_ms(5);
    }

    enable.set_low()?;

    let avg_mv = sum as f32 / SAMPLES as f32;
    Ok(avg_mv / 1000.0 * DIVIDER_RATIO)
}
