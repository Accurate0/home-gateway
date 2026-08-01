use anyhow::Result;
use esp_idf_svc::hal::gpio::{AnyIOPin, Output, PinDriver};
use esp_idf_sys::{gpio_hold_dis, gpio_hold_en, gpio_num_t_GPIO_NUM_43};

pub struct PanelPower<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
}

impl<'a> PanelPower<'a> {
    pub fn enable(pin: AnyIOPin) -> Result<Self> {
        unsafe {
            gpio_hold_dis(gpio_num_t_GPIO_NUM_43);
        }

        let mut pin = PinDriver::output(pin)?;
        pin.set_high()?;

        Ok(Self { pin })
    }
}

impl Drop for PanelPower<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.pin.set_low() {
            log::error!("failed to power down panel rail: {e}");
            return;
        }

        unsafe {
            gpio_hold_en(gpio_num_t_GPIO_NUM_43);
        }
    }
}
