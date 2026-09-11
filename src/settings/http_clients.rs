use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

use super::{HttpClientKind, HttpClientOverride, HttpClientSettings};

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct HttpClientsSettings {
    pub default: HttpClientSettings,
    #[serde(default)]
    pub bom: HttpClientOverride,
    #[serde(default)]
    pub fuelwatch: HttpClientOverride,
    #[serde(default)]
    pub home_assistant: HttpClientOverride,
    #[serde(default)]
    pub jellyfin: HttpClientOverride,
    #[serde(default)]
    pub oauth: HttpClientOverride,
    #[serde(default)]
    pub push: HttpClientOverride,
    #[serde(default)]
    pub transperth: HttpClientOverride,
    #[serde(default)]
    pub trmnl: HttpClientOverride,
    #[serde(default)]
    pub willyweather: HttpClientOverride,
    #[serde(default)]
    pub woolworths: HttpClientOverride,
}

impl HttpClientsSettings {
    pub fn timeout(&self) -> Duration {
        self.default.timeout()
    }

    pub fn timeout_for(&self, kind: HttpClientKind) -> Duration {
        let client = match kind {
            HttpClientKind::Bom => self.bom,
            HttpClientKind::FuelWatch => self.fuelwatch,
            HttpClientKind::HomeAssistant => self.home_assistant,
            HttpClientKind::Jellyfin => self.jellyfin,
            HttpClientKind::OAuth => self.oauth,
            HttpClientKind::Push => self.push,
            HttpClientKind::Transperth => self.transperth,
            HttpClientKind::Trmnl => self.trmnl,
            HttpClientKind::WillyWeather => self.willyweather,
            HttpClientKind::Woolworths => self.woolworths,
        };

        client
            .timeout
            .unwrap_or(self.default.timeout)
            .to_std()
            .unwrap_or_default()
    }
}
