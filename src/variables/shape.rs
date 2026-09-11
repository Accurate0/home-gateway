use std::collections::BTreeMap;

use super::VarType;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Scalar { ty: VarType, optional: bool },
    Object(BTreeMap<String, Shape>),
}

impl Shape {
    pub fn required(ty: VarType) -> Self {
        Shape::Scalar {
            ty,
            optional: false,
        }
    }

    pub fn optional(ty: VarType) -> Self {
        Shape::Scalar { ty, optional: true }
    }

    pub fn empty() -> Self {
        Shape::Object(BTreeMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, shape: Shape) {
        if let Shape::Object(fields) = self {
            fields.insert(key.into(), shape);
        }
    }

    pub fn lookup(&self, segments: &[String]) -> Option<&Shape> {
        let Some((first, rest)) = segments.split_first() else {
            return Some(self);
        };

        match self {
            Shape::Object(fields) => fields.get(first)?.lookup(rest),
            Shape::Scalar { .. } => None,
        }
    }

    pub fn require(&mut self, segments: &[&str]) -> bool {
        let Some((first, rest)) = segments.split_first() else {
            return match self {
                Shape::Scalar { optional, .. } => {
                    *optional = false;
                    true
                }
                Shape::Object(_) => false,
            };
        };

        match self {
            Shape::Object(fields) => fields
                .get_mut(*first)
                .is_some_and(|field| field.require(rest)),
            Shape::Scalar { .. } => false,
        }
    }

    pub fn paths(&self, prefix: &str) -> Vec<String> {
        match self {
            Shape::Scalar { .. } => vec![prefix.to_owned()],
            Shape::Object(fields) => fields
                .iter()
                .flat_map(|(key, field)| {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };

                    field.paths(&path)
                })
                .collect(),
        }
    }
}
