use schemars::JsonSchema;
use serde::Deserialize;

use super::{CallTarget, Script};

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LuaSource {
    Script {
        script: Script,
    },
    Call {
        call: CallTarget,
        #[serde(default)]
        args: Vec<serde_json::Value>,
    },
}

impl LuaSource {
    pub fn summary(&self) -> String {
        match self {
            LuaSource::Script { script } => script.summary(),
            LuaSource::Call { call, .. } => format!("call {call}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LuaSource;

    #[test]
    fn a_step_source_accepts_either_a_script_or_a_call() {
        let script: LuaSource = serde_json::from_value(serde_json::json!({ "script": "return 1" }))
            .expect("expected a script source");
        let called: LuaSource =
            serde_json::from_value(serde_json::json!({ "call": "buttons.dispatch", "args": [1] }))
                .expect("expected a call source");

        assert_eq!(script.summary(), "return 1");
        assert_eq!(called.summary(), "call buttons.dispatch");
    }

    #[test]
    fn a_malformed_call_is_rejected() {
        let result: Result<LuaSource, _> =
            serde_json::from_value(serde_json::json!({ "call": "buttons" }));

        assert!(result.is_err());
    }
}
