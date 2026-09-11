use std::collections::BTreeMap;

use super::{Path, Shape};

#[derive(Debug, Clone, Default)]
pub struct Scope(BTreeMap<String, Shape>);

impl Scope {
    pub fn with(mut self, namespace: impl Into<String>, shape: Shape) -> Self {
        self.0.insert(namespace.into(), shape);
        self
    }

    pub fn lookup(&self, path: &Path) -> Result<&Shape, String> {
        let (first, rest) = path
            .segments()
            .split_first()
            .ok_or_else(|| "empty variable path".to_owned())?;

        self.0
            .get(first)
            .and_then(|shape| shape.lookup(rest))
            .ok_or_else(|| {
                format!(
                    "unknown variable `{path}`; available: [{}]",
                    self.paths().join(", ")
                )
            })
    }

    pub fn paths(&self) -> Vec<String> {
        self.0
            .iter()
            .flat_map(|(namespace, shape)| shape.paths(namespace))
            .collect()
    }
}
