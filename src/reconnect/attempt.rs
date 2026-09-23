use std::time::Duration;

pub struct Attempt {
    pub delay: Duration,
    pub failures: u32,
    pub log: bool,
    pub last_log: bool,
}
