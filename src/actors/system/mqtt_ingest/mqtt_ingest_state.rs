use crate::lua::LuaDecoder;

pub struct MqttIngestState {
    pub decoder: LuaDecoder,
}
