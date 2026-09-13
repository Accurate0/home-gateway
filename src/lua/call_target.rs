use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct CallTarget {
    script: String,
    function: String,
}

impl CallTarget {
    pub fn script(&self) -> &str {
        &self.script
    }

    pub fn function(&self) -> &str {
        &self.function
    }
}

impl TryFrom<String> for CallTarget {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let Some((script, function)) = raw
            .split_once('.')
            .filter(|(script, function)| is_identifier(script) && is_identifier(function))
        else {
            return Err(format!(
                "`{raw}` must be written as `script.function` using lua identifiers"
            ));
        };

        Ok(CallTarget {
            script: script.to_owned(),
            function: function.to_owned(),
        })
    }
}

impl std::fmt::Display for CallTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.script, self.function)
    }
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();

    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::CallTarget;

    #[test]
    fn a_call_names_a_script_and_a_function() {
        let target = CallTarget::try_from("brief.morning".to_owned()).expect("expected a target");

        assert_eq!(target.script(), "brief");
        assert_eq!(target.function(), "morning");
        assert_eq!(target.to_string(), "brief.morning");
    }

    #[test]
    fn a_call_must_name_a_script_and_a_function() {
        for bad in [
            "lights",
            "lights.",
            ".all",
            "lights.all.on",
            "lights.all()",
            "1ights.all",
        ] {
            assert!(
                CallTarget::try_from(bad.to_owned()).is_err(),
                "`{bad}` should be rejected"
            );
        }
    }
}
