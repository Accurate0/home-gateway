use super::message::EventBusMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventFilter {
    All,
    Parts {
        domain: FilterSegment,
        entity: FilterSegment,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterSegment {
    Any,
    Exact(String),
}

impl FilterSegment {
    fn parse(s: &str) -> Self {
        if s == "*" {
            FilterSegment::Any
        } else {
            FilterSegment::Exact(s.to_owned())
        }
    }

    fn matches(&self, other: &str) -> bool {
        match self {
            FilterSegment::Any => true,
            FilterSegment::Exact(value) => value == other,
        }
    }
}

impl EventFilter {
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw == "*" {
            return Some(EventFilter::All);
        }

        let mut segments = raw.split(':');
        let domain = segments.next()?;
        let entity = segments.next()?;
        if segments.next().is_some() {
            return None;
        }

        let domain = FilterSegment::parse(domain);
        if let FilterSegment::Exact(domain) = &domain
            && !EventBusMessage::KINDS.contains(&domain.as_str())
        {
            return None;
        }

        Some(EventFilter::Parts {
            domain,
            entity: FilterSegment::parse(entity),
        })
    }

    pub fn matches(&self, msg: &EventBusMessage) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Parts { domain, entity } => {
                domain.matches(msg.kind()) && entity.matches(&msg.entity())
            }
        }
    }

    pub fn domains(&self) -> Vec<&str> {
        match self {
            EventFilter::All
            | EventFilter::Parts {
                domain: FilterSegment::Any,
                ..
            } => EventBusMessage::KINDS.to_vec(),
            EventFilter::Parts {
                domain: FilterSegment::Exact(domain),
                ..
            } => vec![domain.as_str()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn presence(sensor: &str) -> EventBusMessage {
        EventBusMessage::Presence {
            event_id: Uuid::nil(),
            sensor: sensor.to_owned(),
            present: true,
        }
    }

    fn cron(name: &str) -> EventBusMessage {
        EventBusMessage::Cron {
            event_id: Uuid::nil(),
            name: name.to_owned(),
        }
    }

    fn matches(pattern: &str, msg: &EventBusMessage) -> bool {
        EventFilter::parse(pattern).unwrap().matches(msg)
    }

    #[test]
    fn exact_match() {
        assert!(matches("presence:livingroom", &presence("livingroom")));
        assert!(!matches("presence:livingroom", &presence("kitchen")));
        assert!(!matches("presence:livingroom", &cron("nightly")));
    }

    #[test]
    fn entity_wildcard() {
        assert!(matches("presence:*", &presence("livingroom")));
        assert!(matches("presence:*", &presence("kitchen")));
        assert!(!matches("presence:*", &cron("nightly")));
    }

    #[test]
    fn domain_wildcard() {
        assert!(matches("*:livingroom", &presence("livingroom")));
        assert!(!matches("*:livingroom", &presence("kitchen")));
    }

    #[test]
    fn global_wildcard() {
        assert!(matches("*", &presence("livingroom")));
        assert!(matches("*", &cron("nightly")));
    }

    #[test]
    fn invalid_filters_do_not_parse() {
        assert!(EventFilter::parse("presence").is_none());
        assert!(EventFilter::parse("presence:living:extra").is_none());
        assert!(EventFilter::parse("bogus:living").is_none());
    }
}
