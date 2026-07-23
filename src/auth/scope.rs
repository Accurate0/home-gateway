#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Graphql,
    Rest,
    Ingest,
    Admin,
    Events,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Energy,
    Entity,
    Events,
    Solar,
    Weather,
    Woolworths,
    Control,
    Workflow,
    Push,
    Epd,
    Schema,
    Synergy,
    Home,
    Unifi,
    Keys,
    Presence,
    Door,
    Switch,
    Environment,
    Cron,
    Light,
    Sun,
    Mode,
    HomeAssistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Execute,
}

impl Domain {
    fn from_segment(s: &str) -> Option<Self> {
        Some(match s {
            "graphql" => Self::Graphql,
            "rest" => Self::Rest,
            "ingest" => Self::Ingest,
            "admin" => Self::Admin,
            "events" => Self::Events,
            _ => return None,
        })
    }
}

impl Resource {
    fn from_segment(s: &str) -> Option<Self> {
        Some(match s {
            "energy" => Self::Energy,
            "entity" => Self::Entity,
            "events" => Self::Events,
            "solar" => Self::Solar,
            "weather" => Self::Weather,
            "woolworths" => Self::Woolworths,
            "control" => Self::Control,
            "workflow" => Self::Workflow,
            "push" => Self::Push,
            "epd" => Self::Epd,
            "schema" => Self::Schema,
            "synergy" => Self::Synergy,
            "home" => Self::Home,
            "unifi" => Self::Unifi,
            "keys" => Self::Keys,
            "presence" => Self::Presence,
            "door" => Self::Door,
            "switch" => Self::Switch,
            "environment" => Self::Environment,
            "cron" => Self::Cron,
            "light" => Self::Light,
            "sun" => Self::Sun,
            "mode" => Self::Mode,
            "home_assistant" => Self::HomeAssistant,
            _ => return None,
        })
    }
}

impl Action {
    fn from_segment(s: &str) -> Option<Self> {
        Some(match s {
            "read" => Self::Read,
            "write" => Self::Write,
            "execute" => Self::Execute,
            _ => return None,
        })
    }
}

impl Domain {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Graphql => "graphql",
            Self::Rest => "rest",
            Self::Ingest => "ingest",
            Self::Admin => "admin",
            Self::Events => "events",
        }
    }
}

impl Resource {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Energy => "energy",
            Self::Entity => "entity",
            Self::Events => "events",
            Self::Solar => "solar",
            Self::Weather => "weather",
            Self::Woolworths => "woolworths",
            Self::Control => "control",
            Self::Workflow => "workflow",
            Self::Push => "push",
            Self::Epd => "epd",
            Self::Schema => "schema",
            Self::Synergy => "synergy",
            Self::Home => "home",
            Self::Unifi => "unifi",
            Self::Keys => "keys",
            Self::Presence => "presence",
            Self::Door => "door",
            Self::Switch => "switch",
            Self::Environment => "environment",
            Self::Cron => "cron",
            Self::Light => "light",
            Self::Sun => "sun",
            Self::Mode => "mode",
            Self::HomeAssistant => "home_assistant",
        }
    }

    pub fn for_event_kind(kind: &str) -> Option<Self> {
        Some(match kind {
            "presence" => Self::Presence,
            "door" => Self::Door,
            "switch" => Self::Switch,
            "environment" => Self::Environment,
            "cron" => Self::Cron,
            "light" => Self::Light,
            "unifi" => Self::Unifi,
            "sun" => Self::Sun,
            "mode" => Self::Mode,
            "home_assistant" => Self::HomeAssistant,
            "woolworths" => Self::Woolworths,
            _ => return None,
        })
    }
}

impl Action {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Execute => "execute",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment<T> {
    Any,
    Exact(T),
}

impl<T> Segment<T> {
    fn as_str(&self, render: impl Fn(&T) -> &'static str) -> &'static str {
        match self {
            Segment::Any => "*",
            Segment::Exact(value) => render(value),
        }
    }
}

impl<T: PartialEq> Segment<T> {
    fn matches(&self, other: &T) -> bool {
        match self {
            Segment::Any => true,
            Segment::Exact(value) => value == other,
        }
    }

    fn parse(s: &str, parse_value: impl Fn(&str) -> Option<T>) -> Option<Self> {
        if s == "*" {
            Some(Segment::Any)
        } else {
            parse_value(s).map(Segment::Exact)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    pub domain: Domain,
    pub resource: Resource,
    pub action: Action,
}

impl Scope {
    pub const fn new(domain: Domain, resource: Resource, action: Action) -> Self {
        Self {
            domain,
            resource,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopePattern {
    Global,
    Parts {
        domain: Segment<Domain>,
        resource: Segment<Resource>,
        action: Segment<Action>,
    },
}

impl ScopePattern {
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw == "*" {
            return Some(ScopePattern::Global);
        }

        let mut segments = raw.split(':');
        let domain = segments.next()?;
        let resource = segments.next()?;
        let action = segments.next()?;
        if segments.next().is_some() {
            return None;
        }

        Some(ScopePattern::Parts {
            domain: Segment::parse(domain, Domain::from_segment)?,
            resource: Segment::parse(resource, Resource::from_segment)?,
            action: Segment::parse(action, Action::from_segment)?,
        })
    }

    pub fn matches(&self, required: &Scope) -> bool {
        match self {
            ScopePattern::Global => true,
            ScopePattern::Parts {
                domain,
                resource,
                action,
            } => {
                domain.matches(&required.domain)
                    && resource.matches(&required.resource)
                    && action.matches(&required.action)
            }
        }
    }
}

impl std::fmt::Display for ScopePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopePattern::Global => f.write_str("*"),
            ScopePattern::Parts {
                domain,
                resource,
                action,
            } => write!(
                f,
                "{}:{}:{}",
                domain.as_str(Domain::as_str),
                resource.as_str(Resource::as_str),
                action.as_str(Action::as_str)
            ),
        }
    }
}

pub mod required {
    use super::{Action, Domain, Resource, Scope};

    pub const GRAPHQL_ENERGY_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Energy, Action::Read);
    pub const GRAPHQL_ENTITY_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Entity, Action::Read);
    pub const GRAPHQL_EVENTS_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Events, Action::Read);
    pub const GRAPHQL_LIGHT_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Light, Action::Read);
    pub const GRAPHQL_DOOR_READ: Scope = Scope::new(Domain::Graphql, Resource::Door, Action::Read);
    pub const GRAPHQL_PRESENCE_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Presence, Action::Read);
    pub const GRAPHQL_ENVIRONMENT_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Environment, Action::Read);
    pub const GRAPHQL_SOLAR_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Solar, Action::Read);
    pub const GRAPHQL_WEATHER_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Weather, Action::Read);
    pub const GRAPHQL_WOOLWORTHS_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Woolworths, Action::Read);
    pub const GRAPHQL_WORKFLOW_READ: Scope =
        Scope::new(Domain::Graphql, Resource::Workflow, Action::Read);

    pub const GRAPHQL_LIGHT_WRITE: Scope =
        Scope::new(Domain::Graphql, Resource::Light, Action::Write);
    pub const GRAPHQL_WORKFLOW_WRITE: Scope =
        Scope::new(Domain::Graphql, Resource::Workflow, Action::Write);

    pub const REST_CONTROL_WRITE: Scope =
        Scope::new(Domain::Rest, Resource::Control, Action::Write);
    pub const REST_WORKFLOW_EXECUTE: Scope =
        Scope::new(Domain::Rest, Resource::Workflow, Action::Execute);
    pub const REST_PUSH_WRITE: Scope = Scope::new(Domain::Rest, Resource::Push, Action::Write);
    pub const REST_EPD_READ: Scope = Scope::new(Domain::Rest, Resource::Epd, Action::Read);
    pub const REST_EPD_WRITE: Scope = Scope::new(Domain::Rest, Resource::Epd, Action::Write);
    pub const REST_SCHEMA_READ: Scope = Scope::new(Domain::Rest, Resource::Schema, Action::Read);

    pub const INGEST_SYNERGY_WRITE: Scope =
        Scope::new(Domain::Ingest, Resource::Synergy, Action::Write);
    pub const INGEST_SOLAR_WRITE: Scope =
        Scope::new(Domain::Ingest, Resource::Solar, Action::Write);
    pub const INGEST_HOME_WRITE: Scope = Scope::new(Domain::Ingest, Resource::Home, Action::Write);
    pub const INGEST_UNIFI_WRITE: Scope =
        Scope::new(Domain::Ingest, Resource::Unifi, Action::Write);

    pub const ADMIN_KEYS_READ: Scope = Scope::new(Domain::Admin, Resource::Keys, Action::Read);
    pub const ADMIN_KEYS_WRITE: Scope = Scope::new(Domain::Admin, Resource::Keys, Action::Write);
}

#[cfg(test)]
mod tests {
    use super::required::*;
    use super::*;

    fn matches(granted: &str, required: &Scope) -> bool {
        ScopePattern::parse(granted).unwrap().matches(required)
    }

    #[test]
    fn exact_match() {
        assert!(matches("graphql:solar:read", &GRAPHQL_SOLAR_READ));
    }

    #[test]
    fn resource_wildcard() {
        assert!(matches("graphql:*:read", &GRAPHQL_SOLAR_READ));
        assert!(matches("graphql:*:read", &GRAPHQL_ENERGY_READ));
        assert!(!matches("graphql:*:read", &REST_CONTROL_WRITE));
    }

    #[test]
    fn action_wildcard() {
        assert!(matches("graphql:solar:*", &GRAPHQL_SOLAR_READ));
        assert!(matches("ingest:*:write", &INGEST_HOME_WRITE));
        assert!(matches("ingest:*:write", &INGEST_UNIFI_WRITE));
    }

    #[test]
    fn global_wildcard() {
        assert!(matches("*", &GRAPHQL_SOLAR_READ));
        assert!(matches("*", &REST_CONTROL_WRITE));
        assert!(matches("*", &ADMIN_KEYS_WRITE));
    }

    #[test]
    fn no_match_different_domain() {
        assert!(!matches("graphql:solar:read", &REST_CONTROL_WRITE));
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
    fn invalid_scopes_do_not_parse() {
        assert!(ScopePattern::parse("graphql:solar").is_none());
        assert!(ScopePattern::parse("graphql:solar:read:extra").is_none());
        assert!(ScopePattern::parse("bogus:solar:read").is_none());
        assert!(ScopePattern::parse("graphql:bogus:read").is_none());
        assert!(ScopePattern::parse("graphql:solar:bogus").is_none());
    }
}
