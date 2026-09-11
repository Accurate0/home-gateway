use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SolarMetric {
    Current,
    #[serde(rename = "avg_15m")]
    Avg15m,
    #[serde(rename = "avg_1h")]
    Avg1h,
    #[serde(rename = "avg_3h")]
    Avg3h,
}

impl std::fmt::Display for SolarMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SolarMetric::Current => "current",
            SolarMetric::Avg15m => "avg_15m",
            SolarMetric::Avg1h => "avg_1h",
            SolarMetric::Avg3h => "avg_3h",
        };

        f.write_str(name)
    }
}
