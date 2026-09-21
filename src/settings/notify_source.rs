use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum NotificationSource {
    #[strum(to_string = "watchdog.stale")]
    WatchdogStale,
    #[strum(to_string = "watchdog.recovered")]
    WatchdogRecovered,
    #[strum(to_string = "door.left_open")]
    DoorLeftOpen,
    #[strum(to_string = "workflow.notify.{slug}")]
    Workflow { slug: String },
    #[strum(to_string = "lua.notify.{origin}")]
    Lua { origin: String },
    #[strum(to_string = "api.push")]
    Api,
    #[strum(to_string = "graphql.push")]
    Graphql,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_dotted() {
        assert_eq!(
            NotificationSource::WatchdogStale.to_string(),
            "watchdog.stale"
        );
        assert_eq!(
            NotificationSource::WatchdogRecovered.to_string(),
            "watchdog.recovered"
        );
        assert_eq!(
            NotificationSource::DoorLeftOpen.to_string(),
            "door.left_open"
        );
        assert_eq!(
            NotificationSource::Workflow {
                slug: "bedtime".to_owned()
            }
            .to_string(),
            "workflow.notify.bedtime"
        );
        assert_eq!(
            NotificationSource::Lua {
                origin: "garage".to_owned()
            }
            .to_string(),
            "lua.notify.garage"
        );
        assert_eq!(NotificationSource::Api.to_string(), "api.push");
        assert_eq!(NotificationSource::Graphql.to_string(), "graphql.push");
    }
}
