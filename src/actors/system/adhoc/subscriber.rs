use crate::event_bus::{BusEvent, EventBusMessage, EventSubscriber};

use super::AdhocTaskActorMessage;

pub struct AdhocSubscriber;

impl EventSubscriber for AdhocSubscriber {
    type Msg = AdhocTaskActorMessage;

    const KINDS: &'static [&'static str] = &["feature_flag"];

    fn to_actor_message(&self, event: &BusEvent) -> Option<AdhocTaskActorMessage> {
        match &event.message {
            EventBusMessage::FeatureFlag { state, .. } if state.should_reevaluate() => {
                Some(AdhocTaskActorMessage::Recheck)
            }
            _ => None,
        }
    }
}
