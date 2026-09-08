use crate::event_bus::{EventBusMessage, EventSubscriber};
use crate::settings::SettingsContainer;

use super::AwayMessage;

pub struct AwaySubscriber {
    pub settings: SettingsContainer,
}

impl EventSubscriber for AwaySubscriber {
    type Msg = AwayMessage;

    const KINDS: &'static [&'static str] = &["mode"];

    fn to_actor_message(&self, event: &EventBusMessage) -> Option<AwayMessage> {
        match event {
            EventBusMessage::Mode { mode, active, .. } if self.settings.away.arms_on(mode) => {
                Some(AwayMessage::Arm(*active))
            }
            _ => None,
        }
    }
}
