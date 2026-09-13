#[derive(thiserror::Error, Debug)]
pub enum LuaError {
    #[error("lua syntax error: {0}")]
    Syntax(String),
    #[error("lua runtime error: {0}")]
    Runtime(String),
    #[error("lua script timed out after {0:?}")]
    Timeout(std::time::Duration),
    #[error("lua script exceeded {0} instructions")]
    InstructionLimit(u32),
    #[error("lua script returned {got}, expected {expected}")]
    ReturnType {
        got: &'static str,
        expected: &'static str,
    },
    #[error("lua script returned `{key}` as {got}, expected {expected}")]
    ReturnValue {
        key: String,
        got: String,
        expected: crate::variables::VarType,
    },
    #[error("lua script returned unknown key `{key}`; declared: [{declared}]")]
    UndeclaredReturn { key: String, declared: String },
    #[error("lua script did not return declared key `{key}`")]
    MissingReturn { key: String },
}

impl LuaError {
    pub fn from_mlua(error: mlua::Error) -> Self {
        if let Some(limit) = instruction_limit(&error) {
            return LuaError::InstructionLimit(limit);
        }

        match error {
            mlua::Error::SyntaxError { message, .. } => LuaError::Syntax(message),
            other => LuaError::Runtime(other.to_string()),
        }
    }
}

fn instruction_limit(error: &mlua::Error) -> Option<u32> {
    match error {
        mlua::Error::ExternalError(external) => external
            .downcast_ref::<InstructionLimit>()
            .map(|limit| limit.0),
        mlua::Error::CallbackError { cause, .. } => instruction_limit(cause),
        mlua::Error::WithContext { cause, .. } => instruction_limit(cause),
        _ => None,
    }
}

#[derive(Debug)]
pub struct InstructionLimit(pub u32);

impl std::fmt::Display for InstructionLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exceeded {} instructions", self.0)
    }
}

impl std::error::Error for InstructionLimit {}
