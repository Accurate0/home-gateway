use std::collections::HashMap;

use crate::device_registry::Transport;
use crate::lua::LuaDecoder;

pub struct Decoders {
    by_transport: HashMap<Transport, LuaDecoder>,
}

impl Decoders {
    pub fn new(by_transport: HashMap<Transport, LuaDecoder>) -> Self {
        Self { by_transport }
    }

    pub fn get(&self, transport: Transport) -> Option<&LuaDecoder> {
        self.by_transport.get(&transport)
    }
}
