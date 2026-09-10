use std::collections::BTreeMap;

use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WillyWeatherSettings {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    pub days: i64,
    pub default_location: String,
    pub locations: BTreeMap<String, String>,
}

impl WillyWeatherSettings {
    pub fn resolve_location(&self, location: &str) -> Option<&str> {
        self.locations
            .get_key_value(location)
            .or_else(|| {
                self.locations
                    .iter()
                    .find(|(_, id)| id.as_str() == location)
            })
            .map(|(alias, _)| alias.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_resolves_by_alias_or_by_willyweather_id() {
        let settings = WillyWeatherSettings {
            api_key: None,
            refresh: TimeDelta::hours(1),
            days: 7,
            default_location: "perth".to_owned(),
            locations: BTreeMap::from([("perth".to_owned(), "14576".to_owned())]),
        };

        assert_eq!(settings.resolve_location("perth"), Some("perth"));
        assert_eq!(settings.resolve_location("14576"), Some("perth"));
        assert_eq!(settings.resolve_location("sydney"), None);
    }
}
