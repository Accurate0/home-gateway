use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct Script {
    raw: String,
}

impl Script {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let lua = mlua::Lua::new();

        lua.load(raw)
            .set_name("script")
            .into_function()
            .map_err(|error| error.to_string())?;

        Ok(Script {
            raw: raw.to_owned(),
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn summary(&self) -> String {
        let first = self
            .raw
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with("--"))
            .unwrap_or_default();

        if self
            .raw
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
            > 1
        {
            format!("{first} …")
        } else {
            first.to_owned()
        }
    }
}

impl TryFrom<String> for Script {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Script::parse(&raw)
    }
}

impl std::fmt::Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::Script;

    #[test]
    fn a_syntax_error_is_rejected_at_parse_time() {
        let error = Script::parse("return 1 +").expect_err("expected a syntax error");

        assert!(error.contains("syntax"), "unexpected error: {error}");
    }

    #[test]
    fn a_valid_chunk_is_accepted() {
        let script = Script::parse("return 1 + 1").expect("expected the chunk to compile");

        assert_eq!(script.raw(), "return 1 + 1");
    }

    #[test]
    fn a_summary_marks_a_multi_line_script() {
        let script = Script::parse("local a = 1\nreturn a").expect("expected the chunk to compile");

        assert_eq!(script.summary(), "local a = 1 …");
    }
}
