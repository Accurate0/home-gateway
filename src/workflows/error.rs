use super::MAX_DEPTH;
use crate::actors::devices::{air_purifier, garage_door};
use crate::auth::scope::Scope;

#[derive(thiserror::Error, Debug)]
pub enum WorkflowError {
    #[error("actor `{0}` not found")]
    ActorNotFound(&'static str),
    #[error("workflow recursion depth exceeded (>{MAX_DEPTH})")]
    DepthExceeded,
    #[error("messaging error: {0}")]
    Messaging(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("switch `{0}` has no control path: only a switch declared `as: light` can be driven")]
    NotAControllableSwitch(String),
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error("template error: {0}")]
    Template(String),
    #[error(transparent)]
    Lua(#[from] crate::lua::LuaError),
    #[error("step `{step}` needs scope `{scope}`")]
    MissingScope { step: &'static str, scope: Scope },
    #[error("`{0}` is not a robot vacuum")]
    NotARobotVacuum(String),
    #[error("`{0}` is not a garage door")]
    NotAGarageDoor(String),
    #[error(transparent)]
    GarageDoor(#[from] garage_door::GarageDoorCommandError),
    #[error("`{0}` is not an air purifier")]
    NotAnAirPurifier(String),
    #[error(transparent)]
    AirPurifier(#[from] air_purifier::AirPurifierCommandError),
    #[error("http request to {url} returned {status}")]
    Http {
        url: String,
        status: reqwest::StatusCode,
    },
    #[error("http request refused: {0}")]
    BlockedUrl(String),
    #[error(transparent)]
    HttpRequest(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Mqtt(#[from] crate::integrations::mqtt::MqttError),
    #[error(transparent)]
    HomeAssistant(#[from] crate::integrations::home_assistant::HomeAssistantError),
    #[error(transparent)]
    VacuumCommand(#[from] crate::device_command::DeviceCommandError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
