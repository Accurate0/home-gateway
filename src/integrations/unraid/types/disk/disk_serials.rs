use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiskSerials {
    names: HashMap<String, String>,
}

impl DiskSerials {
    pub fn new(names: HashMap<String, String>) -> Self {
        Self { names }
    }

    pub fn name(&self, serial: &str) -> Option<&str> {
        self.names.get(serial).map(String::as_str)
    }

    pub fn serial(&self, name: &str) -> Option<&str> {
        self.names
            .iter()
            .find(|(_, disk)| *disk == name)
            .map(|(serial, _)| serial.as_str())
    }
}
