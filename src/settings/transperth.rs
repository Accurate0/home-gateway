use crate::timedelta_format::time_delta_from_str;
use chrono::{NaiveTime, TimeDelta};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TransperthRoute {
    pub id: String,
    pub from: String,
    pub to: String,
    pub limit: usize,

    #[serde(default)]
    pub route_group: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawPeakWindow {
    start: String,
    end: String,
}

impl RawPeakWindow {
    fn resolve(self) -> Result<PeakWindow, String> {
        let parse = |value: &str| {
            NaiveTime::parse_from_str(value, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
                .map_err(|_| format!("transperth: invalid peak time `{value}`, expected HH:MM"))
        };

        Ok(PeakWindow {
            start: parse(&self.start)?,
            end: parse(&self.end)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PeakWindow {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl PeakWindow {
    pub fn contains(&self, now: NaiveTime) -> bool {
        if self.start <= self.end {
            now >= self.start && now < self.end
        } else {
            now >= self.start || now < self.end
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawTransperthSettings {
    #[serde(default)]
    pub realtime_api_key: Option<String>,
    #[serde(default)]
    pub reference_data_api_key: Option<String>,

    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh_peak: TimeDelta,

    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh_off_peak: TimeDelta,

    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub horizon: TimeDelta,

    #[serde(default)]
    pub peak: Vec<RawPeakWindow>,

    pub routes: Vec<TransperthRoute>,
}

impl RawTransperthSettings {
    pub fn resolve(self) -> Result<TransperthSettings, String> {
        let RawTransperthSettings {
            realtime_api_key,
            reference_data_api_key,
            refresh_peak,
            refresh_off_peak,
            horizon,
            peak,
            routes,
        } = self;

        let peak = peak
            .into_iter()
            .map(RawPeakWindow::resolve)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TransperthSettings {
            realtime_api_key,
            reference_data_api_key,
            refresh_peak,
            refresh_off_peak,
            horizon,
            peak,
            routes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TransperthSettings {
    pub realtime_api_key: Option<String>,
    pub reference_data_api_key: Option<String>,
    pub refresh_peak: TimeDelta,
    pub refresh_off_peak: TimeDelta,
    pub horizon: TimeDelta,
    pub peak: Vec<PeakWindow>,
    pub routes: Vec<TransperthRoute>,
}

impl TransperthSettings {
    pub fn refresh_for(&self, now: NaiveTime) -> TimeDelta {
        if self.peak.iter().any(|window| window.contains(now)) {
            self.refresh_peak
        } else {
            self.refresh_off_peak
        }
    }
}
