use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};
use esp_idf_sys::{settimeofday, timeval};

pub fn now_unix_ms() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis() as i64,
        Err(_) => 0,
    }
}

pub fn set(unix_ms: i64) -> Result<()> {
    let time = timeval {
        tv_sec: unix_ms.div_euclid(1000) as _,
        tv_usec: (unix_ms.rem_euclid(1000) * 1000) as _,
    };

    let result = unsafe { settimeofday(&time, std::ptr::null()) };

    if result != 0 {
        bail!("settimeofday returned {result}");
    }

    Ok(())
}
