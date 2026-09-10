use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    async_graphql::Enum,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Home,
    Away,
    Vacation,
    Guest,
}

impl Mode {
    pub const ALL: &'static [Mode] = &[Mode::Home, Mode::Away, Mode::Vacation, Mode::Guest];

    pub const STATE_KEY: &'static str = "mode";

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Home => "home",
            Mode::Away => "away",
            Mode::Vacation => "vacation",
            Mode::Guest => "guest",
        }
    }

    pub fn parse(value: &str) -> Option<Mode> {
        Self::ALL
            .iter()
            .copied()
            .find(|mode| mode.as_str() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_round_trips_through_its_name() {
        for mode in Mode::ALL {
            assert_eq!(Mode::parse(mode.as_str()), Some(*mode));
        }
    }

    #[test]
    fn an_unknown_name_does_not_parse() {
        assert_eq!(Mode::parse("party"), None);
    }

    #[test]
    fn the_house_defaults_to_home() {
        assert_eq!(Mode::default(), Mode::Home);
    }
}
