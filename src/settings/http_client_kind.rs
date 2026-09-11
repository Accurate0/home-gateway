#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpClientKind {
    Bom,
    FuelWatch,
    HomeAssistant,
    Jellyfin,
    OAuth,
    Push,
    Transperth,
    Trmnl,
    WillyWeather,
    Woolworths,
}
