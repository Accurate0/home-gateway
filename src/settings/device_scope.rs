use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

use crate::settings::{DeviceAliases, DeviceIds};

pub(crate) struct DeviceScope<'a> {
    ids: &'a DeviceIds,
    aliases: &'a DeviceAliases,
    disabled: &'a HashSet<String>,
    seen_disabled: RefCell<BTreeSet<String>>,
}

impl<'a> DeviceScope<'a> {
    pub(crate) fn new(
        ids: &'a DeviceIds,
        aliases: &'a DeviceAliases,
        disabled: &'a HashSet<String>,
    ) -> Self {
        Self {
            ids,
            aliases,
            disabled,
            seen_disabled: RefCell::new(BTreeSet::new()),
        }
    }

    pub(crate) fn validate(&self, reference: &str) -> Result<(), String> {
        let id = self
            .aliases
            .get(reference)
            .map_or(reference, String::as_str);

        if self.ids.contains_key(id) {
            return Ok(());
        }

        if self.disabled.contains(id) {
            self.seen_disabled.borrow_mut().insert(reference.to_owned());

            return Ok(());
        }

        Err(format!("unknown device registry id: {reference}"))
    }

    pub(crate) fn take_disabled(&self) -> BTreeSet<String> {
        std::mem::take(&mut self.seen_disabled.borrow_mut())
    }
}
