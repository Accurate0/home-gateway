use crate::event_bus::{BusEvent, EventBusMessage, EventSubscriber};

use super::SamplingMessage;

pub struct SamplingSubscriber;

impl EventSubscriber for SamplingSubscriber {
    type Msg = SamplingMessage;

    const KINDS: &'static [&'static str] = &["feature_flag"];

    fn to_actor_message(&self, event: &BusEvent) -> Option<SamplingMessage> {
        match &event.message {
            EventBusMessage::FeatureFlag { state, .. } if state.should_reevaluate() => {
                Some(SamplingMessage::Reevaluate(*state))
            }
            _ => None,
        }
    }
}
