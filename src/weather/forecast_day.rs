use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForecastDay {
    Today,
    Tomorrow,
}

impl ForecastDay {
    pub const ALL: &'static [ForecastDay] = &[ForecastDay::Today, ForecastDay::Tomorrow];

    pub fn index(&self) -> usize {
        match self {
            ForecastDay::Today => 0,
            ForecastDay::Tomorrow => 1,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ForecastDay::Today => "today",
            ForecastDay::Tomorrow => "tomorrow",
        }
    }
}
