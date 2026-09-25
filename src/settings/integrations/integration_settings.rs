use super::esphome::EsphomeSettings;
use super::fuelwatch::FuelWatchSettings;
use super::holidays::HolidaySettings;
use super::jellyfin::JellyfinSettings;
use super::s3::S3Settings;
use super::solar::SolarSettings;
use super::transperth::TransperthSettings;
use super::trmnl::TrmnlSettings;
use super::willyweather::WillyWeatherSettings;
use super::woolworths::WoolworthsSettings;

#[derive(Debug, Clone)]
pub struct IntegrationSettings {
    pub s3: S3Settings,
    pub esphome: EsphomeSettings,
    pub holidays: HolidaySettings,
    pub jellyfin: JellyfinSettings,
    pub woolworths: WoolworthsSettings,
    pub trmnl: TrmnlSettings,
    pub willyweather: WillyWeatherSettings,
    pub fuelwatch: FuelWatchSettings,
    pub solar: SolarSettings,
    pub transperth: TransperthSettings,
}
