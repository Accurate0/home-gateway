use super::{LuaType, LuaTyped};

pub trait LuaReturns {
    const RETURNS: Option<LuaType>;
}

impl LuaReturns for () {
    const RETURNS: Option<LuaType> = None;
}

impl<T: LuaTyped> LuaReturns for T {
    const RETURNS: Option<LuaType> = Some(T::TYPE);
}
