mod attribution;

pub use attribution::Attribution;

use crate::repo::IntentRepo;
use crate::repo::intent::{DeviceIntent, DeviceReport, IntentStatus};

pub async fn observe(
    repo: &IntentRepo,
    address: &str,
    previous: Option<&DeviceReport<'_>>,
    current: &DeviceReport<'_>,
) -> Attribution {
    let pending = match repo.pending_for(current.kind(), address).await {
        Ok(pending) => pending,
        Err(e) => {
            tracing::warn!("failed to read pending intents for {address}: {e}");
            return Attribution::Unchanged;
        }
    };

    let confirmed = confirmed(&pending, current);

    if !confirmed.is_empty() {
        tracing::info!(
            "confirmed {} intent(s) for {address} from device report",
            confirmed.len()
        );

        if let Err(e) = repo.settle(&confirmed, IntentStatus::Confirmed).await {
            tracing::warn!("failed to settle intents for {address}: {e}");
        }
    }

    attribute(&pending, previous, current)
}

fn confirmed(pending: &[DeviceIntent], current: &DeviceReport) -> Vec<i64> {
    pending
        .iter()
        .filter(|intent| intent.relative || intent.attributes.satisfied_by(current))
        .map(|intent| intent.id)
        .collect()
}

fn attribute(
    pending: &[DeviceIntent],
    previous: Option<&DeviceReport>,
    current: &DeviceReport,
) -> Attribution {
    let Some(previous) = previous else {
        return Attribution::Unchanged;
    };

    let changed = current.changed_from(previous);

    if changed.is_empty() {
        return Attribution::Unchanged;
    }

    let fields = changed
        .into_iter()
        .filter(|field| {
            !pending
                .iter()
                .any(|intent| intent.relative || intent.attributes.explains(field, current))
        })
        .collect::<Vec<_>>();

    if fields.is_empty() {
        Attribution::Gateway
    } else {
        Attribution::External { fields }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::repo::air_purifier::{AirPurifierMode, AirPurifierState};
    use crate::repo::intent::{AirPurifierAttributes, IntentAttributes};

    fn state(on: bool, mode: AirPurifierMode, speed: i32, display: bool) -> AirPurifierState {
        AirPurifierState {
            on,
            mode: Some(mode),
            speed: Some(speed),
            display: Some(display),
        }
    }

    fn intent(attributes: AirPurifierAttributes) -> DeviceIntent {
        DeviceIntent {
            id: 1,
            address: "purifier".to_owned(),
            attributes: IntentAttributes::AirPurifier(attributes),
            relative: false,
            attempts: 0,
            event_id: Uuid::new_v4(),
            requested_at: Utc::now(),
        }
    }

    #[test]
    fn a_first_report_is_not_attributed() {
        let current = state(true, AirPurifierMode::Auto, 1, true);

        assert_eq!(
            attribute(&[], None, &DeviceReport::AirPurifier(&current)),
            Attribution::Unchanged
        );
    }

    #[test]
    fn a_change_nobody_asked_for_is_external() {
        let previous = state(true, AirPurifierMode::Auto, 1, true);
        let current = state(true, AirPurifierMode::Sleep, 1, true);

        assert_eq!(
            attribute(
                &[],
                Some(&DeviceReport::AirPurifier(&previous)),
                &DeviceReport::AirPurifier(&current)
            ),
            Attribution::External {
                fields: vec!["mode"]
            }
        );
    }

    #[test]
    fn a_change_a_pending_intent_wants_is_the_gateway() {
        let previous = state(true, AirPurifierMode::Auto, 1, true);
        let current = state(true, AirPurifierMode::Sleep, 1, true);
        let pending = [intent(AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            speed: None,
            display: Some(false),
        })];

        assert_eq!(
            attribute(
                &pending,
                Some(&DeviceReport::AirPurifier(&previous)),
                &DeviceReport::AirPurifier(&current)
            ),
            Attribution::Gateway
        );
        assert!(confirmed(&pending, &DeviceReport::AirPurifier(&current)).is_empty());
    }

    #[test]
    fn a_change_against_a_pending_intent_is_external() {
        let previous = state(true, AirPurifierMode::Auto, 1, true);
        let current = state(true, AirPurifierMode::Manual, 3, true);
        let pending = [intent(AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            ..Default::default()
        })];

        assert_eq!(
            attribute(
                &pending,
                Some(&DeviceReport::AirPurifier(&previous)),
                &DeviceReport::AirPurifier(&current)
            ),
            Attribution::External {
                fields: vec!["mode"]
            }
        );
    }

    #[test]
    fn the_fan_moving_itself_in_auto_is_not_a_change() {
        let previous = state(true, AirPurifierMode::Auto, 1, true);
        let current = state(true, AirPurifierMode::Auto, 3, true);

        assert_eq!(
            attribute(
                &[],
                Some(&DeviceReport::AirPurifier(&previous)),
                &DeviceReport::AirPurifier(&current)
            ),
            Attribution::Unchanged
        );
    }
}
