use std::collections::HashSet;

use schemars::JsonSchema;
use serde::Deserialize;

use super::esphome::EsphomeSettings;
use super::fuelwatch::FuelWatchSettings;
use super::holidays::HolidaySettings;
use super::integration_settings::IntegrationSettings;
use super::jellyfin::JellyfinSettings;
use super::s3::S3Settings;
use super::solar::SolarSettings;
use super::transperth::{RawTransperthSettings, TransperthSettings};
use super::trmnl::TrmnlSettings;
use super::willyweather::WillyWeatherSettings;
use super::woolworths::WoolworthsSettings;

fn missing(value: Option<&str>) -> bool {
    value.map(str::trim).is_none_or(str::is_empty)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawIntegrationSettings {
    s3: S3Settings,
    pub(crate) esphome: EsphomeSettings,
    holidays: HolidaySettings,
    jellyfin: JellyfinSettings,
    woolworths: WoolworthsSettings,
    trmnl: TrmnlSettings,
    willyweather: WillyWeatherSettings,
    fuelwatch: FuelWatchSettings,
    solar: SolarSettings,
    transperth: RawTransperthSettings,
}

impl RawIntegrationSettings {
    pub fn resolve(self) -> Result<IntegrationSettings, String> {
        let RawIntegrationSettings {
            s3,
            esphome,
            holidays,
            jellyfin,
            woolworths,
            trmnl,
            willyweather,
            fuelwatch,
            solar,
            transperth,
        } = self;

        validate_esphome(&esphome)?;
        validate_holidays(&holidays)?;
        validate_jellyfin(&jellyfin)?;
        validate_willyweather(&willyweather)?;

        let transperth = transperth.resolve()?;

        if transperth.state.is_enabled() {
            validate_transperth(&transperth)?;
        }

        Ok(IntegrationSettings {
            s3,
            esphome,
            holidays,
            jellyfin,
            woolworths,
            trmnl,
            willyweather,
            fuelwatch,
            solar,
            transperth,
        })
    }
}

fn validate_esphome(esphome: &EsphomeSettings) -> Result<(), String> {
    if esphome.state.is_enabled() && missing(esphome.encryption_key.as_deref()) {
        return Err(
            "integrations.esphome.encryption_key is required (set INTEGRATIONS__ESPHOME__ENCRYPTION_KEY)"
                .to_owned(),
        );
    }

    Ok(())
}

fn validate_holidays(holidays: &HolidaySettings) -> Result<(), String> {
    if holidays.url.trim().is_empty() {
        return Err("integrations.holidays.url must not be empty".to_owned());
    }

    if holidays.regions.is_empty() {
        return Err("integrations.holidays.regions must declare at least one region".to_owned());
    }

    Ok(())
}

fn validate_jellyfin(jellyfin: &JellyfinSettings) -> Result<(), String> {
    if !jellyfin.state.is_enabled() {
        return Ok(());
    }

    if jellyfin.url.trim().is_empty() {
        return Err("integrations.jellyfin.url must not be empty".to_owned());
    }

    if missing(jellyfin.api_key.as_deref()) {
        return Err(
            "integrations.jellyfin.api_key is required (set INTEGRATIONS__JELLYFIN__API_KEY)"
                .to_owned(),
        );
    }

    if jellyfin.poll_interval <= chrono::TimeDelta::zero() {
        return Err("integrations.jellyfin.poll_interval must be positive".to_owned());
    }

    Ok(())
}

fn validate_willyweather(willyweather: &WillyWeatherSettings) -> Result<(), String> {
    if willyweather.state.is_enabled() && missing(willyweather.api_key.as_deref()) {
        return Err(
            "integrations.willyweather.api_key is required (set INTEGRATIONS__WILLYWEATHER__API_KEY)"
                .to_owned(),
        );
    }

    if willyweather.locations.is_empty() {
        return Err(
            "integrations.willyweather.locations must declare at least one location".to_owned(),
        );
    }

    if !willyweather
        .locations
        .contains_key(&willyweather.default_location)
    {
        return Err(format!(
            "integrations.willyweather.default_location `{}` is not one of its locations",
            willyweather.default_location
        ));
    }

    if willyweather.refresh <= chrono::TimeDelta::zero() {
        return Err("integrations.willyweather.refresh must be positive".to_owned());
    }

    if willyweather.days < 1 {
        return Err("integrations.willyweather.days must be at least 1".to_owned());
    }

    Ok(())
}

fn validate_transperth(transperth: &TransperthSettings) -> Result<(), String> {
    if missing(transperth.reference_data_api_key.as_deref()) {
        return Err(
            "integrations.transperth.reference_data_api_key is required (set INTEGRATIONS__TRANSPERTH__REFERENCE_DATA_API_KEY)"
                .to_owned(),
        );
    }

    if transperth.routes.is_empty() {
        return Err("integrations.transperth.routes must not be empty".to_owned());
    }

    let mut seen_route_ids = HashSet::new();

    for route in &transperth.routes {
        if !seen_route_ids.insert(route.id.clone()) {
            return Err(format!("duplicate transperth route id: {}", route.id));
        }

        if route.from.trim().is_empty() || route.to.trim().is_empty() {
            return Err(format!(
                "transperth route {} has an empty from/to",
                route.id
            ));
        }

        if route.limit == 0 {
            return Err(format!("transperth route {} must have limit > 0", route.id));
        }
    }

    Ok(())
}
