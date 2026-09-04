use std::fmt;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Read,
    Write,
}

impl Action {
    fn from_segment(s: &str) -> Option<Self> {
        Some(match s {
            "read" => Self::Read,
            "write" => Self::Write,
            _ => return None,
        })
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

macro_rules! scopes {
    ($($variant:ident => $path:literal [$($action:ident),+ $(,)?]),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Resource {
            $($variant),+
        }

        impl Resource {
            pub const ALL: &'static [Resource] = &[$(Resource::$variant),+];

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $path),+
                }
            }

            fn from_path(s: &str) -> Option<Self> {
                match s {
                    $($path => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl Scope {
            pub const ALL: &'static [Scope] = &[
                $($(Scope::new(Resource::$variant, Action::$action),)+)+
            ];
        }
    };
}

scopes! {
    AdhocTask => "adhoc_task" [Read, Write],
    AdminKeys => "admin.keys" [Read, Write],
    Control => "control" [Write],
    Door => "door" [Read],
    Energy => "energy" [Read],
    Environment => "environment" [Read],
    Epd => "epd" [Read, Write],
    FuelWatch => "fuelwatch" [Read],
    HomeAssistant => "home_assistant" [Read],
    IngestHome => "ingest.home" [Write],
    IngestSynergy => "ingest.synergy" [Write],
    IngestUnifi => "ingest.unifi" [Write],
    Jellyfin => "jellyfin" [Read],
    Light => "light" [Read, Write],
    MediaPlayer => "media.player" [Read, Write],
    Presence => "presence" [Read],
    Push => "push" [Write],
    RobotVacuum => "robot_vacuum" [Read, Write],
    Schema => "schema" [Read],
    Solar => "solar" [Read],
    Transperth => "transperth" [Read],
    Weather => "weather" [Read],
    Woolworths => "woolworths" [Read],
    Workflow => "workflow" [Read, Write],

    EventsBattery => "events.battery" [Read],
    EventsCron => "events.cron" [Read],
    EventsDoor => "events.door" [Read],
    EventsEnvironment => "events.environment" [Read],
    EventsHomeAssistant => "events.home_assistant" [Read],
    EventsJellyfin => "events.jellyfin" [Read],
    EventsLight => "events.light" [Read],
    EventsMediaPlayer => "events.media_player" [Read],
    EventsMode => "events.mode" [Read],
    EventsPresence => "events.presence" [Read],
    EventsSolar => "events.solar" [Read],
    EventsSun => "events.sun" [Read],
    EventsSwitch => "events.switch" [Read],
    EventsUnifi => "events.unifi" [Read],
    EventsWoolworths => "events.woolworths" [Read],
}

impl Resource {
    pub fn for_event_kind(kind: &str) -> Option<Self> {
        Some(match kind {
            "presence" => Self::EventsPresence,
            "door" => Self::EventsDoor,
            "switch" => Self::EventsSwitch,
            "environment" => Self::EventsEnvironment,
            "cron" => Self::EventsCron,
            "light" => Self::EventsLight,
            "unifi" => Self::EventsUnifi,
            "sun" => Self::EventsSun,
            "mode" => Self::EventsMode,
            "home_assistant" => Self::EventsHomeAssistant,
            "woolworths" => Self::EventsWoolworths,
            "device_battery" => Self::EventsBattery,
            "jellyfin" => Self::EventsJellyfin,
            "media_player" => Self::EventsMediaPlayer,
            "solar" => Self::EventsSolar,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scope {
    pub resource: Resource,
    pub action: Action,
}

impl Scope {
    pub const fn new(resource: Resource, action: Action) -> Self {
        Self { resource, action }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.resource.as_str(), self.action.as_str())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScopeParseError {
    #[error("expected `resource:action`, got `{0}`")]
    Shape(String),
    #[error("`@` is reserved for a future target selector")]
    ReservedTarget,
    #[error("empty path segment in `{0}`")]
    EmptySegment(String),
    #[error("`**` must be the last path segment")]
    RestNotLast,
    #[error("unknown resource `{0}`")]
    UnknownResource(String),
    #[error("unknown action `{0}`")]
    UnknownAction(String),
    #[error("resource `{resource}` has no `{action}` action")]
    UnsupportedAction { resource: String, action: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    Exact(String),
    One,
    Rest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionSegment {
    Any,
    Exact(Action),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePattern {
    path: Vec<PathSegment>,
    action: ActionSegment,
}

impl ScopePattern {
    pub fn parse(raw: &str) -> Result<Self, ScopeParseError> {
        let raw = raw.trim();
        if raw.contains('@') {
            return Err(ScopeParseError::ReservedTarget);
        }

        let (resource, action) = raw
            .split_once(':')
            .ok_or_else(|| ScopeParseError::Shape(raw.to_owned()))?;
        if resource.is_empty() || action.contains(':') {
            return Err(ScopeParseError::Shape(raw.to_owned()));
        }

        let mut path = Vec::new();
        for segment in resource.split('.') {
            if segment.is_empty() {
                return Err(ScopeParseError::EmptySegment(resource.to_owned()));
            }
            path.push(match segment {
                "*" => PathSegment::One,
                "**" => PathSegment::Rest,
                other => PathSegment::Exact(other.to_owned()),
            });
        }

        if path
            .iter()
            .position(|s| *s == PathSegment::Rest)
            .is_some_and(|i| i != path.len() - 1)
        {
            return Err(ScopeParseError::RestNotLast);
        }

        let exact_resource = if path.iter().all(|s| matches!(s, PathSegment::Exact(_))) {
            Some(
                Resource::from_path(resource)
                    .ok_or_else(|| ScopeParseError::UnknownResource(resource.to_owned()))?,
            )
        } else {
            None
        };

        let action = if action == "*" {
            ActionSegment::Any
        } else {
            ActionSegment::Exact(
                Action::from_segment(action)
                    .ok_or_else(|| ScopeParseError::UnknownAction(action.to_owned()))?,
            )
        };

        if let (Some(resource), ActionSegment::Exact(action)) = (exact_resource, action)
            && !Scope::ALL.contains(&Scope::new(resource, action))
        {
            return Err(ScopeParseError::UnsupportedAction {
                resource: resource.as_str().to_owned(),
                action: action.as_str().to_owned(),
            });
        }

        Ok(Self { path, action })
    }

    pub fn matches(&self, required: &Scope) -> bool {
        match self.action {
            ActionSegment::Any => {}
            ActionSegment::Exact(action) if action == required.action => {}
            ActionSegment::Exact(_) => return false,
        }

        let mut actual = required.resource.as_str().split('.');
        for segment in &self.path {
            match segment {
                PathSegment::Rest => return actual.next().is_some(),
                PathSegment::One => {
                    if actual.next().is_none() {
                        return false;
                    }
                }
                PathSegment::Exact(expected) => {
                    if actual.next() != Some(expected.as_str()) {
                        return false;
                    }
                }
            }
        }

        actual.next().is_none()
    }
}

impl fmt::Display for ScopePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self
            .path
            .iter()
            .map(|segment| match segment {
                PathSegment::Exact(value) => value.as_str(),
                PathSegment::One => "*",
                PathSegment::Rest => "**",
            })
            .collect::<Vec<_>>()
            .join(".");

        let action = match self.action {
            ActionSegment::Any => "*",
            ActionSegment::Exact(action) => action.as_str(),
        };

        write!(f, "{path}:{action}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLAR_READ: Scope = Scope::new(Resource::Solar, Action::Read);
    const LIGHT_READ: Scope = Scope::new(Resource::Light, Action::Read);
    const LIGHT_WRITE: Scope = Scope::new(Resource::Light, Action::Write);
    const MEDIA_PLAYER_READ: Scope = Scope::new(Resource::MediaPlayer, Action::Read);
    const MEDIA_PLAYER_WRITE: Scope = Scope::new(Resource::MediaPlayer, Action::Write);
    const ADMIN_KEYS_READ: Scope = Scope::new(Resource::AdminKeys, Action::Read);
    const ADMIN_KEYS_WRITE: Scope = Scope::new(Resource::AdminKeys, Action::Write);
    const EVENTS_PRESENCE_READ: Scope = Scope::new(Resource::EventsPresence, Action::Read);

    fn matches(granted: &str, required: &Scope) -> bool {
        ScopePattern::parse(granted).unwrap().matches(required)
    }

    #[test]
    fn exact_match() {
        assert!(matches("solar:read", &SOLAR_READ));
        assert!(matches("media.player:write", &MEDIA_PLAYER_WRITE));
    }

    #[test]
    fn action_wildcard() {
        assert!(matches("light:*", &LIGHT_READ));
        assert!(matches("light:*", &LIGHT_WRITE));
        assert!(!matches("light:*", &SOLAR_READ));
    }

    #[test]
    fn write_does_not_grant_read() {
        assert!(!matches("light:write", &LIGHT_READ));
    }

    #[test]
    fn single_segment_wildcard_does_not_cross_a_dot() {
        assert!(matches("*:read", &SOLAR_READ));
        assert!(!matches("*:read", &MEDIA_PLAYER_READ));
        assert!(matches("media.*:read", &MEDIA_PLAYER_READ));
        assert!(!matches("media.*:read", &SOLAR_READ));
    }

    #[test]
    fn rest_wildcard_spans_subtrees() {
        assert!(matches("**:read", &SOLAR_READ));
        assert!(matches("**:read", &MEDIA_PLAYER_READ));
        assert!(!matches("**:read", &LIGHT_WRITE));
        assert!(matches("events.**:read", &EVENTS_PRESENCE_READ));
        assert!(!matches("events.**:read", &SOLAR_READ));
    }

    #[test]
    fn global_wildcard() {
        assert!(matches("**:*", &SOLAR_READ));
        assert!(matches("**:*", &ADMIN_KEYS_WRITE));
        assert!(matches("**:*", &MEDIA_PLAYER_WRITE));
    }

    #[test]
    fn readers_do_not_get_admin() {
        assert!(matches("**:read", &ADMIN_KEYS_READ));
        assert!(!matches("events.**:read", &ADMIN_KEYS_READ));
    }

    #[test]
    fn every_event_kind_maps_to_a_resource() {
        for kind in crate::event_bus::EventBusMessage::KINDS {
            assert!(
                Resource::for_event_kind(kind).is_some(),
                "event kind `{kind}` has no Resource mapping"
            );
        }
    }

    #[test]
    fn every_resource_round_trips() {
        for resource in Resource::ALL {
            assert_eq!(Resource::from_path(resource.as_str()), Some(*resource));
        }
    }

    #[test]
    fn patterns_round_trip_through_display() {
        for raw in ["solar:read", "media.*:write", "events.**:read", "**:*"] {
            assert_eq!(ScopePattern::parse(raw).unwrap().to_string(), raw);
        }
    }

    #[test]
    fn invalid_scopes_do_not_parse() {
        assert_eq!(
            ScopePattern::parse("solar"),
            Err(ScopeParseError::Shape("solar".to_owned()))
        );
        assert_eq!(
            ScopePattern::parse("solar:read:extra"),
            Err(ScopeParseError::Shape("solar:read:extra".to_owned()))
        );
        assert_eq!(
            ScopePattern::parse("bogus:read"),
            Err(ScopeParseError::UnknownResource("bogus".to_owned()))
        );
        assert_eq!(
            ScopePattern::parse("solar:bogus"),
            Err(ScopeParseError::UnknownAction("bogus".to_owned()))
        );
        assert_eq!(
            ScopePattern::parse("**.player:read"),
            Err(ScopeParseError::RestNotLast)
        );
        assert_eq!(
            ScopePattern::parse("media..player:read"),
            Err(ScopeParseError::EmptySegment("media..player".to_owned()))
        );
        assert_eq!(
            ScopePattern::parse("light:write@kitchen_lamp"),
            Err(ScopeParseError::ReservedTarget)
        );
        assert_eq!(
            ScopePattern::parse("control:read"),
            Err(ScopeParseError::UnsupportedAction {
                resource: "control".to_owned(),
                action: "read".to_owned()
            })
        );
    }
}
