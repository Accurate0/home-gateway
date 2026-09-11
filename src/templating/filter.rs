use crate::variables::{Value, VarType};

use super::{Evaluated, Literal};

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Default(Literal),
    Round(u32),
    Int,
}

impl Filter {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();

        let (name, argument) = match input.split_once('(') {
            Some((name, rest)) => {
                let argument = rest
                    .trim_end()
                    .strip_suffix(')')
                    .ok_or_else(|| format!("filter `{input}` is missing a closing `)`"))?;

                (name.trim(), Some(argument.trim()))
            }
            None => (input, None),
        };

        match (name, argument) {
            ("default", Some(argument)) => Ok(Filter::Default(Literal::parse(argument)?)),
            ("round", Some(argument)) => argument.parse::<u32>().map(Filter::Round).map_err(|_| {
                format!("`round` expects a whole number of decimals, got `{argument}`")
            }),
            ("int", None) => Ok(Filter::Int),
            ("default", None) => Err("`default` expects a value, e.g. default(\"n/a\")".to_owned()),
            ("round", None) => Err("`round` expects decimals, e.g. round(1)".to_owned()),
            ("int", Some(_)) => Err("`int` takes no arguments".to_owned()),
            (name, _) => Err(format!(
                "unknown filter `{name}`; available: default, round, int"
            )),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Filter::Default(_) => "default",
            Filter::Round(_) => "round",
            Filter::Int => "int",
        }
    }

    pub fn check(&self, ty: VarType, optional: bool) -> Result<(VarType, bool), String> {
        match self {
            Filter::Round(_) | Filter::Int if !matches!(ty, VarType::Int | VarType::Float) => {
                Err(format!("`{}` needs a number, got {ty}", self.name()))
            }
            Filter::Round(_) => Ok((VarType::Float, optional)),
            Filter::Int => Ok((VarType::Int, optional)),
            Filter::Default(_) if !optional => {
                Err("value is never missing; remove `default`".to_owned())
            }
            Filter::Default(literal) => {
                let literal_type = literal.var_type();

                if ty.accepts(literal_type) && (ty != VarType::String || literal_type == ty) {
                    Ok((ty, false))
                } else if literal_type == VarType::String {
                    Ok((VarType::String, false))
                } else {
                    Err(format!(
                        "`default({literal})` is a {literal_type} but the value is a {ty}"
                    ))
                }
            }
        }
    }

    pub fn apply(&self, evaluated: Evaluated) -> Evaluated {
        match self {
            Filter::Round(decimals) => Evaluated {
                value: evaluated.value.map(|value| {
                    let factor = 10f64.powi(*decimals as i32);
                    let number = value.as_f64().unwrap_or_default();

                    Value::Float((number * factor).round() / factor)
                }),
                decimals: Some(*decimals),
            },
            Filter::Int => Evaluated {
                value: evaluated
                    .value
                    .map(|value| Value::Int(value.as_f64().unwrap_or_default().round() as i64)),
                decimals: None,
            },
            Filter::Default(literal) => match evaluated.value {
                Some(_) => evaluated,
                None => Evaluated {
                    value: Some(literal.to_value()),
                    decimals: None,
                },
            },
        }
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Default(literal) => write!(f, "default({literal})"),
            Filter::Round(decimals) => write!(f, "round({decimals})"),
            Filter::Int => f.write_str("int"),
        }
    }
}
