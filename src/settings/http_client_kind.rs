#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpClientKind {
    Bom,
    FuelWatch,
    Holidays,
    HomeAssistant,
    Jellyfin,
    OAuth,
    Push,
    Transperth,
    Trmnl,
    WillyWeather,
    Woolworths,
}
