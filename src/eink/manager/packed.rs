pub enum Packed {
    Cached,
    Rendered(Vec<u8>),
}

impl Packed {
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Packed::Cached => None,
            Packed::Rendered(bytes) => Some(bytes),
        }
    }
}
