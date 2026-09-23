#[derive(Debug, Clone, PartialEq)]
pub enum MediaCommand {
    Play,
    Pause,
    PlayPause,
    Stop,
    Next,
    Previous,
    Volume(f64),
    Mute(bool),
    TurnOff,
    PlayMedia { url: String, announcement: bool },
}

impl std::fmt::Display for MediaCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MediaCommand::Play => "play",
            MediaCommand::Pause => "pause",
            MediaCommand::PlayPause => "play_pause",
            MediaCommand::Stop => "stop",
            MediaCommand::Next => "next",
            MediaCommand::Previous => "previous",
            MediaCommand::Volume(_) => "volume",
            MediaCommand::Mute(_) => "mute",
            MediaCommand::TurnOff => "turn_off",
            MediaCommand::PlayMedia { .. } => "play_media",
        };

        f.write_str(name)
    }
}
