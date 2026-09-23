use serde_json::Value;

use crate::media_control::MediaCommand;

pub enum Command {
    Light(Value),
    Media(MediaCommand),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Light(_) => f.write_str("light"),
            Command::Media(command) => write!(f, "media {command}"),
        }
    }
}
