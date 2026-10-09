use super::{EInkActorState, EInkDisplayActor, EInkDisplayMessage};
use crate::eink::manager::render_schedule::{RenderSchedule, render_schedule};
use crate::eink::manager::resolve::ResolvedDisplay;

impl EInkDisplayActor {
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

        match render_schedule(next_wake_at, lead, chrono::Utc::now()) {
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
