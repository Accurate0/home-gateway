use schemars::JsonSchema;
use serde::Deserialize;

use crate::variables::{Scope, Value, VarType, Vars};

use super::parser::parse_segments;
use super::{Expr, Segment};

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct Template {
    raw: String,
    segments: Vec<Segment>,
}

impl Template {
    pub fn parse(raw: &str) -> Result<Self, String> {
        Ok(Template {
            raw: raw.to_owned(),
            segments: parse_segments(raw)?,
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn exprs(&self) -> impl Iterator<Item = &Expr> {
        self.segments.iter().filter_map(|segment| match segment {
            Segment::Expr(expr) => Some(expr),
            Segment::Text(_) => None,
        })
    }

    pub fn references_namespace(&self, namespace: &str) -> bool {
        self.exprs().any(|expr| expr.path.namespace() == namespace)
    }

    pub fn check(&self, scope: &Scope) -> Result<VarType, String> {
        let mut types = self
            .exprs()
            .map(|expr| expr.check(scope))
            .collect::<Result<Vec<_>, _>>()?;

        let ty = match self.segments.as_slice() {
            [Segment::Expr(_)] => types.pop().unwrap_or(VarType::String),
            _ => VarType::String,
        };

        Ok(ty)
    }

    pub fn render(&self, vars: &Vars) -> Result<String, String> {
        let mut out = String::with_capacity(self.raw.len());

        for segment in &self.segments {
            match segment {
                Segment::Text(text) => out.push_str(text),
                Segment::Expr(expr) => {
                    let rendered = expr
                        .evaluate(vars)
                        .render()
                        .ok_or_else(|| format!("`{}` has no value", expr.path))?;

                    out.push_str(&rendered);
                }
            }
        }

        Ok(out)
    }

    pub fn evaluate(&self, vars: &Vars) -> Result<Value, String> {
        match self.segments.as_slice() {
            [Segment::Expr(expr)] => expr.value(vars),
            _ => self.render(vars).map(Value::String),
        }
    }
}

impl TryFrom<String> for Template {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Template::parse(&raw)
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}
