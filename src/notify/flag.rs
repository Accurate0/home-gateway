use open_feature::{EvaluationContext, StructValue, Value};

use crate::integrations::feature_flag::FeatureFlagClient;
use crate::settings::{NotificationSource, NotifyFilter};

pub const NOTIFICATION_CONFIG_FLAG: &str = "home-gateway-notification-config";

fn filter_from(value: StructValue) -> NotifyFilter {
    let patterns: Vec<String> = match value.fields.get("disabled") {
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(|entry| match entry {
                Value::String(pattern) => Some(pattern.clone()),
                _ => {
                    tracing::warn!("{NOTIFICATION_CONFIG_FLAG} disabled entry {entry:?} is not a string, ignoring");

                    None
                }
            })
            .collect(),
        Some(_) => {
            tracing::warn!("{NOTIFICATION_CONFIG_FLAG} disabled is not an array, ignoring");

            Vec::new()
        }
        None => Vec::new(),
    };

    NotifyFilter::new(&patterns)
}

pub async fn filter(client: &FeatureFlagClient, source: &NotificationSource) -> NotifyFilter {
    let evaluation_context =
        EvaluationContext::default().with_custom_field("source", source.to_string());

    match client
        .get_struct(NOTIFICATION_CONFIG_FLAG, evaluation_context)
        .await
    {
        Ok(value) => filter_from(value),
        Err(e) => {
            tracing::error!(
                "error evaluating {NOTIFICATION_CONFIG_FLAG}: {e:?}, using config only"
            );

            NotifyFilter::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_value(fields: Vec<(&str, Value)>) -> StructValue {
        let mut value = StructValue::default();

        for (key, field) in fields {
            value.fields.insert(key.to_owned(), field);
        }

        value
    }

    #[test]
    fn disabled_patterns_are_read_from_the_flag() {
        let filter = filter_from(struct_value(vec![(
            "disabled",
            Value::Array(vec![
                Value::String("watchdog.*".to_owned()),
                Value::String("workflow.notify.bedtime".to_owned()),
            ]),
        )]));

        assert!(filter.is_disabled("watchdog.stale"));
        assert!(filter.is_disabled("workflow.notify.bedtime"));
        assert!(!filter.is_disabled("door.left_open"));
    }

    #[test]
    fn a_wrongly_typed_entry_is_ignored_without_dropping_the_rest() {
        let filter = filter_from(struct_value(vec![(
            "disabled",
            Value::Array(vec![Value::Int(3), Value::String("api.push".to_owned())]),
        )]));

        assert!(filter.is_disabled("api.push"));
    }

    #[test]
    fn an_empty_flag_disables_nothing() {
        let filter = filter_from(struct_value(vec![]));

        assert!(!filter.is_disabled("watchdog.stale"));
    }

    #[test]
    fn a_non_array_disabled_is_ignored() {
        let filter = filter_from(struct_value(vec![(
            "disabled",
            Value::String("*".to_owned()),
        )]));

        assert!(!filter.is_disabled("watchdog.stale"));
    }
}
