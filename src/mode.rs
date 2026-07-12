use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Home,
    Away,
    Vacation,
    Night,
    Guest,
    Party,
}

impl Mode {
    pub const ALL: &'static [Mode] = &[
        Mode::Home,
        Mode::Away,
        Mode::Vacation,
        Mode::Night,
        Mode::Guest,
        Mode::Party,
    ];

    const OCCUPANCY: &'static [Mode] = &[Mode::Home, Mode::Away, Mode::Vacation];

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Home => "home",
            Mode::Away => "away",
            Mode::Vacation => "vacation",
            Mode::Night => "night",
            Mode::Guest => "guest",
            Mode::Party => "party",
        }
    }

    pub fn state_key(&self) -> String {
        format!("mode:{}", self.as_str())
    }

    pub fn exclusive_peers(&self) -> impl Iterator<Item = Mode> + '_ {
        let group: &'static [Mode] = if Self::OCCUPANCY.contains(self) {
            Self::OCCUPANCY
        } else {
            &[]
        };
        group.iter().copied().filter(move |m| m != self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occupancy_modes_are_mutually_exclusive() {
        let peers: Vec<Mode> = Mode::Home.exclusive_peers().collect();
        assert_eq!(peers, vec![Mode::Away, Mode::Vacation]);
    }

    #[test]
    fn toggle_modes_have_no_peers() {
        assert_eq!(Mode::Guest.exclusive_peers().count(), 0);
        assert_eq!(Mode::Night.exclusive_peers().count(), 0);
        assert_eq!(Mode::Party.exclusive_peers().count(), 0);
    }

    #[test]
    fn state_key_namespaced() {
        assert_eq!(Mode::Vacation.state_key(), "mode:vacation");
    }
}
