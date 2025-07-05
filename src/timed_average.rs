use std::time::Duration;

use chrono::{DateTime, Utc};

struct Value {
    v: f64,
    time: DateTime<Utc>,
}

pub struct TimedAverage {
    last_minutes: Duration,
    last_n_values: Vec<Value>,
}

impl TimedAverage {
    pub fn new(last_minutes: Duration) -> Self {
        Self {
            last_minutes,
            last_n_values: Default::default(),
        }
    }

    pub fn push(&mut self, v: f64) {
        let now = chrono::Utc::now();
        let last_allowed = now - self.last_minutes;
        self.last_n_values.push(Value { v, time: now });
        self.last_n_values.retain(|v| v.time >= last_allowed);
    }

    pub fn value(&mut self) -> f64 {
        let mut avg = 0.0;
        let count = self.last_n_values.len();

        for val in &self.last_n_values {
            avg += val.v
        }

        avg / count as f64
    }
}
