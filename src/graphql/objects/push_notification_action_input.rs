use async_graphql::InputObject;

use crate::actors::system::push::types::{PushAction, PushActionKind};
use crate::graphql::objects::push_notification_action_kind::PushNotificationActionKind;
use crate::settings::SettingsContainer;

#[derive(InputObject)]
#[graphql(name = "PushNotificationActionInput")]
pub struct PushNotificationActionInput {
    pub label: String,
    pub kind: PushNotificationActionKind,
    pub workflow_slug: Option<String>,
    pub snooze_seconds: Option<i64>,
}

impl PushNotificationActionInput {
    pub fn into_push_action(self, settings: &SettingsContainer) -> Result<PushAction, String> {
        let kind = match self.kind {
            PushNotificationActionKind::RunWorkflow => {
                let slug = self.workflow_slug.ok_or_else(|| {
                    format!(
                        "action '{}' is run_workflow but has no workflowSlug",
                        self.label
                    )
                })?;

                if !settings.workflows.values().any(|w| w.body().slug == slug) {
                    return Err(format!("unknown workflow slug: {slug}"));
                }

                PushActionKind::RunWorkflow { slug }
            }
            PushNotificationActionKind::Snooze => {
                let seconds = self.snooze_seconds.ok_or_else(|| {
                    format!("action '{}' is snooze but has no snoozeSeconds", self.label)
                })?;

                let seconds = u64::try_from(seconds)
                    .map_err(|_| format!("action '{}' has negative snoozeSeconds", self.label))?;

                PushActionKind::Snooze { seconds }
            }
            PushNotificationActionKind::Dismiss => PushActionKind::Dismiss,
            PushNotificationActionKind::Acknowledge => PushActionKind::Acknowledge,
        };

        Ok(PushAction {
            label: self.label,
            kind,
        })
    }
}
