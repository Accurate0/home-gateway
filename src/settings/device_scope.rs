use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

use crate::settings::DeviceAliases;

pub(crate) struct DeviceScope<'a> {
    aliases: &'a DeviceAliases,
    disabled: &'a HashSet<String>,
    seen_disabled: RefCell<BTreeSet<String>>,
}

impl<'a> DeviceScope<'a> {
    pub(crate) fn new(aliases: &'a DeviceAliases, disabled: &'a HashSet<String>) -> Self {
        Self {
            aliases,
            disabled,
            seen_disabled: RefCell::new(BTreeSet::new()),
        }
    }

    pub(crate) fn validate(&self, reference: &str) -> Result<(), String> {
        if self.aliases.contains_key(reference) {
            return Ok(());
        }

        if self.disabled.contains(reference) {
            self.seen_disabled.borrow_mut().insert(reference.to_owned());

            return Ok(());
        }

        Err(format!("unknown device registry id: {reference}"))
    }

    pub(crate) fn take_disabled(&self) -> BTreeSet<String> {
        std::mem::take(&mut self.seen_disabled.borrow_mut())
    }
}
