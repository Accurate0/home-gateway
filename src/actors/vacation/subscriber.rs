use crate::event_bus::{BusEvent, EventBusMessage, EventSubscriber};
use crate::settings::SettingsContainer;

use super::VacationMessage;

pub struct VacationSubscriber {
    pub settings: SettingsContainer,
}

impl EventSubscriber for VacationSubscriber {
    type Msg = VacationMessage;

    const KINDS: &'static [&'static str] = &["mode"];

    fn to_actor_message(&self, event: &BusEvent) -> Option<VacationMessage> {
        match &event.message {
            EventBusMessage::Mode { mode, .. } => {
                Some(VacationMessage::Arm(self.settings.vacation.arms_on(mode)))
            }
            _ => None,
        }
    }
}
