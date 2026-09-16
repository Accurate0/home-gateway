use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::auth::scope::Scope;
use crate::lua::LuaSource;
use crate::variables::VarType;

use super::workflow::HttpMethod;

pub const RESERVED_PREFIXES: &[&str] = &[
    "/admin",
    "/control",
    "/epd",
    "/graphql",
    "/health",
    "/ingest",
    "/lua",
    "/metrics",
    "/push",
    "/schema",
    "/solar",
    "/weather",
    "/workflow",
];

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct EndpointSettings {
    #[serde(default)]
    pub routes: Vec<Endpoint>,
}

impl EndpointSettings {
    pub fn validate(&self) -> Result<(), String> {
        let mut seen: Vec<(&str, HttpMethod)> = Vec::with_capacity(self.routes.len());

        for endpoint in &self.routes {
            endpoint.validate()?;

            let key = (endpoint.path.as_str(), endpoint.method);

            if seen.contains(&key) {
                return Err(format!(
                    "endpoint `{} {}` is declared twice",
                    endpoint.method, endpoint.path
                ));
            }

            seen.push(key);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Endpoint {
    pub path: String,
    pub method: HttpMethod,
    #[serde(flatten)]
    pub source: LuaSource,
    #[serde(default)]
    pub scopes: Vec<RequiredScope>,
    #[serde(default)]
    pub params: BTreeMap<String, ParamType>,
    #[serde(default)]
    pub body: BTreeMap<String, ParamType>,
}

impl Endpoint {
    fn validate(&self) -> Result<(), String> {
        if !self.path.starts_with('/') {
            return Err(format!("endpoint path `{}` must start with `/`", self.path));
        }

        if self.path == "/" {
            return Err("endpoint path must not be `/`".to_owned());
        }

        if let Some(prefix) = reserved_prefix(&self.path) {
            return Err(format!(
                "endpoint path `{}` collides with the built-in `{prefix}` routes",
                self.path
            ));
        }

        Ok(())
    }

    pub fn required_scopes(&self) -> impl Iterator<Item = Scope> + '_ {
        self.scopes.iter().map(|scope| scope.0)
    }
}

fn reserved_prefix(path: &str) -> Option<&'static str> {
    RESERVED_PREFIXES.iter().copied().find(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct RequiredScope(pub Scope);

impl TryFrom<String> for RequiredScope {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Scope::parse(&raw)
            .map(RequiredScope)
            .map_err(|error| format!("`{raw}` is not a valid scope: {error}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct ParamType {
    pub ty: VarType,
    pub required: bool,
}

impl TryFrom<String> for ParamType {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let (name, required) = match raw.strip_suffix('?') {
            Some(name) => (name, false),
            None => (raw.as_str(), true),
        };

        let ty = match name {
            "string" => VarType::String,
            "int" => VarType::Int,
            "float" => VarType::Float,
            "bool" => VarType::Bool,
            _ => {
                return Err(format!(
                    "`{raw}` must be one of string, int, float, bool, optionally suffixed with `?`"
                ));
            }
        };

        Ok(ParamType { ty, required })
    }
}

#[cfg(test)]
mod tests {
    use super::{EndpointSettings, ParamType, RequiredScope};
    use crate::variables::VarType;

    fn endpoints(yaml: &str) -> EndpointSettings {
        serde_yaml::from_str(yaml).expect("expected the endpoints to parse")
    }

    #[test]
    fn a_param_type_may_be_optional() {
        let required = ParamType::try_from("int".to_owned()).expect("expected a type");
        let optional = ParamType::try_from("int?".to_owned()).expect("expected a type");

        assert_eq!(
            required,
            ParamType {
                ty: VarType::Int,
                required: true
            }
        );
        assert_eq!(
            optional,
            ParamType {
                ty: VarType::Int,
                required: false
            }
        );
    }

    #[test]
    fn an_unknown_param_type_is_rejected() {
        let error = ParamType::try_from("date".to_owned()).expect_err("expected a rejection");

        assert!(error.contains("must be one of"));
    }

    #[test]
    fn a_scope_is_parsed_at_load() {
        RequiredScope::try_from("light:read".to_owned()).expect("expected a scope");

        let error =
            RequiredScope::try_from("light:teleport".to_owned()).expect_err("expected a rejection");

        assert!(error.contains("is not a valid scope"));
    }

    #[test]
    fn a_path_must_not_collide_with_a_built_in_prefix() {
        let settings = endpoints(
            r#"
            routes:
              - path: /ingest/mine
                method: GET
                call: endpoints.mine
            "#,
        );

        let error = settings.validate().expect_err("expected a collision");

        assert!(error.contains("collides with the built-in `/ingest`"));
    }

    #[test]
    fn a_path_that_merely_shares_a_prefix_is_allowed() {
        let settings = endpoints(
            r#"
            routes:
              - path: /ingestion
                method: GET
                call: endpoints.mine
            "#,
        );

        settings.validate().expect("expected the path to be free");
    }

    #[test]
    fn a_path_must_start_with_a_slash() {
        let settings = endpoints(
            r#"
            routes:
              - path: display
                method: GET
                call: endpoints.mine
            "#,
        );

        let error = settings.validate().expect_err("expected a rejection");

        assert!(error.contains("must start with `/`"));
    }

    #[test]
    fn the_same_path_and_method_may_not_be_declared_twice() {
        let settings = endpoints(
            r#"
            routes:
              - path: /display
                method: GET
                call: endpoints.mine
              - path: /display
                method: GET
                call: endpoints.other
            "#,
        );

        let error = settings.validate().expect_err("expected a duplicate");

        assert!(error.contains("is declared twice"));
    }

    #[test]
    fn the_same_path_may_serve_two_methods() {
        let settings = endpoints(
            r#"
            routes:
              - path: /display
                method: GET
                call: endpoints.read
              - path: /display
                method: POST
                call: endpoints.write
            "#,
        );

        settings.validate().expect("expected both methods to load");
    }
}
