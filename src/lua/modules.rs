use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(super) struct Modules<'a> {
    pub kind: &'static str,
    pub sources: &'a BTreeMap<String, Vec<u8>>,
}
