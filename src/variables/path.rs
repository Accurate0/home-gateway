use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String")]
pub struct Path(Vec<String>);

impl Path {
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        &self.0[0]
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let segments = input
            .split('.')
            .map(|segment| {
                let valid = !segment.is_empty()
                    && segment
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

                if valid {
                    Ok(segment.to_owned())
                } else {
                    Err(format!("invalid variable path `{input}`"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Path(segments))
    }
}

impl TryFrom<String> for Path {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Path::parse(value.trim())
    }
}

impl From<Path> for String {
    fn from(path: Path) -> Self {
        path.to_string()
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join("."))
    }
}
