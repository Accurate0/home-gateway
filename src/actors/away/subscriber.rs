use crate::event_bus::{BusEvent, EventBusMessage, EventSubscriber};
use crate::settings::SettingsContainer;

use super::AwayMessage;

pub struct AwaySubscriber {
    pub settings: SettingsContainer,
}

impl EventSubscriber for AwaySubscriber {
    type Msg = AwayMessage;

    const KINDS: &'static [&'static str] = &["mode"];

    fn to_actor_message(&self, event: &BusEvent) -> Option<AwayMessage> {
        match &event.message {
            EventBusMessage::Mode { mode, active, .. } if self.settings.away.arms_on(mode) => {
                Some(AwayMessage::Arm(*active))
            }
            _ => None,
        }
    }
}
