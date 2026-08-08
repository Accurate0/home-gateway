use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrString {
    Num(f64),
    Str(String),
}

impl NumberOrString {
    fn as_f64(self) -> Option<f64> {
        match self {
            NumberOrString::Num(n) => Some(n),
            NumberOrString::Str(s) => s.parse::<f64>().ok(),
        }
    }
}

pub fn opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<NumberOrString>::deserialize(deserializer)?.and_then(NumberOrString::as_f64))
}

pub fn opt_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(opt_f64(deserializer)?.map(|n| n as i32))
}
