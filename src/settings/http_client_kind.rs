#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpClientKind {
    Bom,
    FuelWatch,
    GoodWe,
    Holidays,
    HomeAssistant,
    OAuth,
    Push,
    Reddit,
    Transperth,
    Trmnl,
    WillyWeather,
    Woolworths,
}
