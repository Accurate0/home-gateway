#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpClientKind {
    Bom,
    FuelWatch,
    GoodWe,
    Holidays,
    HomeAssistant,
    Immich,
    Jellyfin,
    OAuth,
    Push,
    Reddit,
    Transperth,
    Trmnl,
    Unraid,
    WillyWeather,
    Woolworths,
}
