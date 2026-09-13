use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Node, Path, Value};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Vars(BTreeMap<String, Node>);

impl Vars {
    pub fn with(mut self, namespace: impl Into<String>, node: Node) -> Self {
        self.insert(namespace, node);
        self
    }

    pub fn insert(&mut self, namespace: impl Into<String>, node: Node) {
        self.0.insert(namespace.into(), node);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Node)> {
        self.0.iter()
    }

    pub fn contains(&self, namespace: &str) -> bool {
        self.0.contains_key(namespace)
    }

    pub fn namespace(&self, namespace: &str) -> Option<&Node> {
        self.0.get(namespace)
    }

    pub fn get(&self, path: &Path) -> Option<&Node> {
        let (first, rest) = path.segments().split_first()?;
        self.0.get(first)?.lookup(rest)
    }

    pub fn value(&self, path: &Path) -> Option<&Value> {
        match self.get(path)? {
            Node::Value(value) => value.as_ref(),
            Node::Object(_) => None,
        }
    }
}
