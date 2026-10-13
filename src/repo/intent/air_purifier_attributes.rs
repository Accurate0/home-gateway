use serde::{Deserialize, Serialize};

use crate::repo::air_purifier::{AirPurifierField, AirPurifierMode, AirPurifierState};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirPurifierAttributes {
    pub on: Option<bool>,
    pub mode: Option<AirPurifierMode>,
    pub speed: Option<i32>,
    pub display: Option<bool>,
}

impl AirPurifierAttributes {
    pub fn is_empty(&self) -> bool {
        self.on.is_none() && self.mode.is_none() && self.speed.is_none() && self.display.is_none()
    }

    pub fn merged_over(self, pending: &AirPurifierAttributes) -> Self {
        let turning_off = self.on == Some(false);

        let mode = if turning_off {
            None
        } else {
            self.mode.or(pending.mode)
        };

        let speed = if turning_off || self.mode.is_some() {
            self.speed
        } else {
            self.speed.or(pending.speed)
        };

        Self {
            on: self.on.or(pending.on),
            mode,
            speed,
            display: self.display.or(pending.display),
        }
    }

    pub fn satisfied_by(&self, state: &AirPurifierState) -> bool {
        let on_matches = self.on.is_none_or(|want| state.on == want);
        let mode_matches = self.mode.is_none_or(|want| state.mode == Some(want));
        let speed_matches = self.speed.is_none_or(|want| state.speed == Some(want));
        let display_matches = self.display.is_none_or(|want| state.display == Some(want));

        on_matches && mode_matches && speed_matches && display_matches
    }

    pub fn explains(&self, field: &str, state: &AirPurifierState) -> bool {
        let on: &str = AirPurifierField::On.into();
        let mode: &str = AirPurifierField::Mode.into();
        let speed: &str = AirPurifierField::Speed.into();
        let display: &str = AirPurifierField::Display.into();

        if field == on {
            self.on == Some(state.on)
        } else if field == mode {
            self.mode.is_some() && self.mode == state.mode
        } else if field == speed {
            match self.speed {
                Some(want) => state.speed == Some(want),
                None => self.on.is_some() || self.mode.is_some(),
            }
        } else if field == display {
            self.display.is_some() && self.display == state.display
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(on: bool, mode: AirPurifierMode, speed: i32, display: bool) -> AirPurifierState {
        AirPurifierState {
            on,
            mode: Some(mode),
            speed: Some(speed),
            display: Some(display),
        }
    }

    #[test]
    fn a_later_command_is_merged_over_what_is_still_pending() {
        let pending = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            ..Default::default()
        };
        let display = AirPurifierAttributes {
            display: Some(false),
            ..Default::default()
        };

        assert_eq!(
            display.merged_over(&pending),
            AirPurifierAttributes {
                on: Some(true),
                mode: Some(AirPurifierMode::Sleep),
                speed: None,
                display: Some(false),
            }
        );
    }

    #[test]
    fn turning_off_drops_the_pending_mode_and_speed() {
        let pending = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Manual),
            speed: Some(2),
            display: Some(false),
        };
        let off = AirPurifierAttributes {
            on: Some(false),
            ..Default::default()
        };

        assert_eq!(
            off.merged_over(&pending),
            AirPurifierAttributes {
                on: Some(false),
                mode: None,
                speed: None,
                display: Some(false),
            }
        );
    }

    #[test]
    fn a_new_mode_drops_the_pending_speed() {
        let pending = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Manual),
            speed: Some(3),
            display: None,
        };
        let auto = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Auto),
            ..Default::default()
        };

        assert_eq!(auto.merged_over(&pending).speed, None);
    }

    #[test]
    fn an_intent_is_satisfied_only_by_every_wanted_field() {
        let wanted = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            speed: None,
            display: Some(false),
        };

        assert!(!wanted.satisfied_by(&state(true, AirPurifierMode::Sleep, 1, true)));
        assert!(wanted.satisfied_by(&state(true, AirPurifierMode::Sleep, 1, false)));
    }

    #[test]
    fn only_wanted_values_are_explained() {
        let wanted = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            ..Default::default()
        };
        let reported = state(true, AirPurifierMode::Sleep, 1, true);

        assert!(wanted.explains("on", &reported));
        assert!(wanted.explains("mode", &reported));
        assert!(wanted.explains("speed", &reported));
        assert!(!wanted.explains("display", &reported));
        assert!(!wanted.explains("mode", &state(true, AirPurifierMode::Auto, 1, true)));
    }

    #[test]
    fn a_display_intent_does_not_explain_a_speed_change() {
        let wanted = AirPurifierAttributes {
            display: Some(false),
            ..Default::default()
        };

        assert!(!wanted.explains("speed", &state(true, AirPurifierMode::Manual, 3, false)));
    }
}
