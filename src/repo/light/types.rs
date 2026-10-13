use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySource {
    Edge,
    Sample,
}

impl HistorySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            HistorySource::Edge => "edge",
            HistorySource::Sample => "sample",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightSample {
    pub address: String,
    pub device_id: Option<String>,
    pub source: HistorySource,
    pub event_id: Option<Uuid>,
    pub state: LightState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileBucket {
    pub address: String,
    pub isodow: i16,
    pub slot: i16,
    pub on_fraction: f64,
    pub observations: i64,
    pub turned_on: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightState {
    pub on: bool,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LightAttributes {
    pub state: Option<String>,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<String>,
}

impl LightAttributes {
    pub fn state(state: impl Into<String>) -> Self {
        Self {
            state: Some(state.into()),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_none()
            && self.brightness.is_none()
            && self.colour_temp.is_none()
            && self.colour.is_none()
    }

    pub fn satisfied_by(&self, state: &LightState) -> bool {
        let on_matches = match self.state.as_deref() {
            Some(want) => state.on == (want == "ON"),
            None => true,
        };

        let brightness_matches = self
            .brightness
            .is_none_or(|want| state.brightness == Some(want));

        let colour_temp_matches = self
            .colour_temp
            .is_none_or(|want| state.colour_temp == Some(want));

        let colour_matches = self
            .colour
            .as_deref()
            .is_none_or(|want| state.colour.as_deref() == Some(want));

        on_matches && brightness_matches && colour_temp_matches && colour_matches
    }

    pub fn explains(&self, field: &str, state: &LightState) -> bool {
        match field {
            LightState::STATE => match self.state.as_deref() {
                Some(want) => state.on == (want == "ON"),
                None => state.on && !self.is_empty(),
            },
            LightState::BRIGHTNESS => {
                self.brightness.is_some() && self.brightness == state.brightness
            }
            LightState::COLOUR_TEMP => {
                self.colour_temp.is_some() && self.colour_temp == state.colour_temp
            }
            LightState::COLOUR => self.colour.is_some() && self.colour == state.colour,
            _ => false,
        }
    }
}

impl LightState {
    pub const STATE: &str = "state";
    pub const BRIGHTNESS: &str = "brightness";
    pub const COLOUR_TEMP: &str = "colour_temp";
    pub const COLOUR: &str = "colour";

    pub fn changed_from(&self, previous: &LightState) -> Vec<&'static str> {
        let mut changed = Vec::new();

        if self.on != previous.on {
            changed.push(Self::STATE);
        }

        if self.brightness != previous.brightness {
            changed.push(Self::BRIGHTNESS);
        }

        if self.colour_temp != previous.colour_temp {
            changed.push(Self::COLOUR_TEMP);
        }

        if self.colour != previous.colour {
            changed.push(Self::COLOUR);
        }

        changed
    }
}
