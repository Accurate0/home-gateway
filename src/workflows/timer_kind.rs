#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerKind {
    Hold,
    Delay,
}

impl TimerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimerKind::Hold => "hold",
            TimerKind::Delay => "delay",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hold" => Some(TimerKind::Hold),
            "delay" => Some(TimerKind::Delay),
            _ => None,
        }
    }
}
