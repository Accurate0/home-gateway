use esp_idf_sys::{
    esp_task_wdt_add, esp_task_wdt_config_t, esp_task_wdt_delete, esp_task_wdt_init,
    esp_task_wdt_reconfigure, esp_task_wdt_reset, ESP_ERR_INVALID_STATE, ESP_OK,
};

const TIMEOUT_MS: u32 = 300_000;

pub fn start() {
    let config = esp_task_wdt_config_t {
        timeout_ms: TIMEOUT_MS,
        idle_core_mask: 0,
        trigger_panic: true,
    };

    let init = unsafe { esp_task_wdt_init(&config) };

    let init = if init == ESP_ERR_INVALID_STATE {
        unsafe { esp_task_wdt_reconfigure(&config) }
    } else {
        init
    };

    if init != ESP_OK {
        log::warn!("failed to init task watchdog: {init}");
        return;
    }

    let subscribe = unsafe { esp_task_wdt_add(std::ptr::null_mut()) };

    if subscribe != ESP_OK {
        log::warn!("failed to subscribe to task watchdog: {subscribe}");
        return;
    }

    log::info!("task watchdog armed for {TIMEOUT_MS}ms");
}

pub fn feed() {
    unsafe { esp_task_wdt_reset() };
}

pub fn stop() {
    unsafe { esp_task_wdt_delete(std::ptr::null_mut()) };
}
