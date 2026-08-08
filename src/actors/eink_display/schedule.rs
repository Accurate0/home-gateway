use super::{EInkActorState, EInkDisplayActor, EInkDisplayMessage};
use crate::eink::manager::resolve::ResolvedDisplay;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
enum RenderSchedule {
    Now,
    After(Duration),
}

impl EInkDisplayActor {
    fn render_schedule(
        next_wake_at: Option<chrono::DateTime<chrono::Utc>>,
        lead: chrono::TimeDelta,
        now: chrono::DateTime<chrono::Utc>,
    ) -> RenderSchedule {
        let Some(next_wake_at) = next_wake_at else {
            return RenderSchedule::Now;
        };

        match (next_wake_at - lead - now).to_std() {
            Ok(delay) => RenderSchedule::After(delay),
            Err(_) => RenderSchedule::Now,
        }
    }

    pub(super) fn cancel_render(state: &mut EInkActorState, device_id: &str) {
        if let Some(handle) = state.scheduled_renders.remove(device_id) {
            handle.abort();
        }
    }

    pub(super) fn schedule_render(
        &self,
        myself: &ractor::ActorRef<EInkDisplayMessage>,
        state: &mut EInkActorState,
        display: &ResolvedDisplay,
        next_wake_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let device_id = display.device_id.as_str();

        Self::cancel_render(state, device_id);

        let render_now = || {
            myself.send_message(EInkDisplayMessage::TakeScreenshot {
                device_id: Some(device_id.to_owned()),
            })
        };

        let Some(lead) = display.lead else {
            tracing::warn!(
                device_id = %device_id,
                "dashboard mode came from a flag override with no lead configured, rendering now"
            );
            render_now()?;
            return Ok(());
        };

        match Self::render_schedule(next_wake_at, lead, chrono::Utc::now()) {
            RenderSchedule::Now => {
                tracing::info!(
                    device_id = %device_id,
                    ?next_wake_at,
                    "next wake is not far enough out to pre-render, rendering now"
                );
                render_now()?;
            }
            RenderSchedule::After(delay) => {
                tracing::info!(
                    device_id = %device_id,
                    ?next_wake_at,
                    delay_secs = delay.as_secs(),
                    "scheduled a dashboard pre-render"
                );

                let id = device_id.to_owned();
                let handle = myself.send_after(delay, move || EInkDisplayMessage::TakeScreenshot {
                    device_id: Some(id.clone()),
                });

                state
                    .scheduled_renders
                    .insert(device_id.to_owned(), handle.abort_handle());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wake_beyond_the_lead_is_scheduled() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            EInkDisplayActor::render_schedule(Some(now + chrono::TimeDelta::hours(1)), lead, now),
            RenderSchedule::After(Duration::from_secs(45 * 60))
        );
    }

    #[test]
    fn a_wake_inside_the_lead_renders_now() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            EInkDisplayActor::render_schedule(Some(now + chrono::TimeDelta::minutes(5)), lead, now),
            RenderSchedule::Now
        );
    }

    #[test]
    fn a_wake_in_the_past_renders_now() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            EInkDisplayActor::render_schedule(Some(now - chrono::TimeDelta::days(3)), lead, now),
            RenderSchedule::Now
        );
    }

    #[test]
    fn a_display_that_never_polled_renders_now() {
        let now = chrono::Utc::now();

        assert_eq!(
            EInkDisplayActor::render_schedule(None, chrono::TimeDelta::minutes(15), now),
            RenderSchedule::Now
        );
    }
}
