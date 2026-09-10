use async_graphql::InputObject;
use chrono::TimeDelta;

use crate::settings::NotifyAcknowledge;

#[derive(InputObject)]
#[graphql(name = "PushNotificationAcknowledgeInput")]
pub struct PushNotificationAcknowledgeInput {
    pub remind_after_seconds: i64,
    pub reminders: i32,
}

impl TryFrom<PushNotificationAcknowledgeInput> for NotifyAcknowledge {
    type Error = String;

    fn try_from(input: PushNotificationAcknowledgeInput) -> Result<Self, Self::Error> {
        if input.remind_after_seconds < 0 {
            return Err("remindAfterSeconds must not be negative".to_owned());
        }

        let reminders =
            u32::try_from(input.reminders).map_err(|_| "reminders must not be negative")?;

        Ok(NotifyAcknowledge {
            remind_after: TimeDelta::seconds(input.remind_after_seconds),
            reminders,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_seconds_into_a_reminder_policy() {
        let acknowledge = NotifyAcknowledge::try_from(PushNotificationAcknowledgeInput {
            remind_after_seconds: 7200,
            reminders: 1,
        })
        .unwrap();

        assert_eq!(acknowledge.remind_after, TimeDelta::hours(2));
        assert_eq!(acknowledge.reminders, 1);
    }

    #[test]
    fn rejects_negative_values() {
        for (remind_after_seconds, reminders) in [(-1, 1), (60, -1)] {
            let result = NotifyAcknowledge::try_from(PushNotificationAcknowledgeInput {
                remind_after_seconds,
                reminders,
            });

            assert!(result.is_err());
        }
    }
}
