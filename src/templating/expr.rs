use schemars::JsonSchema;
use serde::Deserialize;

use crate::variables::{Path, Scope, Shape, Value, VarType, Vars};

use super::parser::split_outside_quotes;
use super::{Evaluated, Filter};

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct Expr {
    pub path: Path,
    pub filters: Vec<Filter>,
}

impl TryFrom<String> for Expr {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Expr::parse(&raw)
    }
}

impl Expr {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut parts = split_outside_quotes(input, '|').into_iter();

        let path = parts.next().unwrap_or_default();
        let path = Path::parse(path.trim())?;

        let filters = parts.map(Filter::parse).collect::<Result<Vec<_>, _>>()?;

        Ok(Expr { path, filters })
    }

    pub fn check(&self, scope: &Scope) -> Result<VarType, String> {
        let (mut ty, mut optional) = match scope.lookup(&self.path)? {
            Shape::Scalar { ty, optional } => (*ty, *optional),
            Shape::Object(_) => {
                return Err(format!(
                    "`{}` is an object; pick one of its fields",
                    self.path
                ));
            }
        };

        for filter in &self.filters {
            (ty, optional) = filter
                .check(ty, optional)
                .map_err(|error| format!("`{self}`: {error}"))?;
        }

        if optional {
            return Err(format!("`{self}` may be missing; add `| default(...)`"));
        }

        Ok(ty)
    }

    pub fn evaluate(&self, vars: &Vars) -> Evaluated {
        let initial = Evaluated {
            value: vars.value(&self.path).cloned(),
            decimals: None,
        };

        self.filters
            .iter()
            .fold(initial, |evaluated, filter| filter.apply(evaluated))
    }

    pub fn value(&self, vars: &Vars) -> Result<Value, String> {
        self.evaluate(vars)
            .value
            .ok_or_else(|| format!("`{}` has no value", self.path))
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)?;

        for filter in &self.filters {
            write!(f, " | {filter}")?;
        }

        Ok(())
    }
}
