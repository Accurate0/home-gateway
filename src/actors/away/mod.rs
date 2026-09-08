//! Away mode: replays the home's own historical lighting while nobody is here.
//!
//! While one of the configured occupancy modes is active, the light history
//! aggregate is turned into a switching plan for the day (see
//! [`crate::away::plan`]) and each transition is armed as a one-shot timer, the
//! same shape as [`crate::actors::sun::SunActor`]. The plan is deterministic, so
//! a restart mid-day resumes it instead of re-rolling.

use chrono::{Duration, NaiveDate, Utc};
use chrono_tz::Australia::Perth;
use ractor::Actor;

use crate::actors::devices::light::{LightHandler, LightHandlerMessage};
use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::away::{PlannedAction, build_plan};
use crate::event_bus::{Recipient, Subscription};
use crate::state::AppState;

pub mod subscriber;

pub use subscriber::AwaySubscriber;

pub enum AwayMessage {
    Arm(bool),
    Rebuild,
    Fire { address: String, on: bool },
}

pub struct AwayActor {
    pub shared_actor_state: AppState,
}

#[derive(Default)]
pub struct AwayState {
    _subscription: Subscription,
    armed: bool,
    plan: Vec<PlannedAction>,
}

impl AwayActor {
    pub const NAME: &str = "away";

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&Perth).date_naive()
    }

    async fn armed_now(&self) -> bool {
        let manager = self.shared_actor_state.handles.expect::<WorkflowManager>();
        let away = &self.shared_actor_state.settings.away;

        for mode in &away.modes {
            if manager.mode_active(*mode).await {
                return true;
            }
        }

        false
    }

    /// Rebuild today's plan and arm a timer for every action still ahead of us,
    /// plus one at local midnight to roll onto the next day.
    async fn rebuild(
        &self,
        myself: &ractor::ActorRef<AwayMessage>,
        state: &mut AwayState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let settings = &self.shared_actor_state.settings.away;
        let day = self.today();

        let buckets = self
            .shared_actor_state
            .repos
            .light()
            .profile_all(settings.window)
            .await?;

        state.plan = build_plan(&buckets, day, settings);

        let now = Utc::now();
        let mut armed = 0;

        for action in &state.plan {
            let Ok(delay) = (action.at - now).to_std() else {
                continue;
            };

            let address = action.address.clone();
            let on = action.on;
            myself.send_after(delay, move || AwayMessage::Fire { address, on });
            armed += 1;
        }

        let midnight = day
            .succ_opt()
            .and_then(|next| next.and_hms_opt(0, 0, 0))
            .map(|local| local.and_utc() - Duration::hours(8));

        if let Some(midnight) = midnight
            && let Ok(delay) = (midnight - now).to_std()
        {
            myself.send_after(delay, || AwayMessage::Rebuild);
        }

        tracing::info!(
            "away plan for {day}: {} actions, {armed} still ahead across {} lights",
            state.plan.len(),
            self.shared_actor_state.devices.lights().count()
        );

        Ok(())
    }

    async fn fire(&self, address: String, on: bool) -> Result<(), ractor::ActorProcessingErr> {
        let current = self
            .shared_actor_state
            .repos
            .light()
            .is_on(&address)
            .await?;

        if current == Some(on) {
            tracing::info!(
                "away: {address} is already {}",
                if on { "on" } else { "off" }
            );

            return Ok(());
        }

        let ieee_addr = address.clone();
        let message = if on {
            LightHandlerMessage::TurnOn { ieee_addr }
        } else {
            LightHandlerMessage::TurnOff { ieee_addr }
        };

        match rpc::cast_factory(LightHandler::NAME, message) {
            Ok(()) => tracing::info!(
                "away: dispatched {address} {}",
                if on { "on" } else { "off" }
            ),
            Err(e) => tracing::warn!("away: could not switch {address}: {e}"),
        }

        Ok(())
    }
}

impl Actor for AwayActor {
    type Msg = AwayMessage;
    type State = AwayState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let subscription = self.shared_actor_state.event_bus.register(
            Self::NAME,
            Recipient::Actor(myself.clone()),
            AwaySubscriber {
                settings: self.shared_actor_state.settings.clone(),
            },
        );

        let mut state = AwayState {
            _subscription: subscription,
            armed: self.armed_now().await,
            plan: Vec::new(),
        };

        if state.armed {
            tracing::info!("away mode is already active, building the replay plan");
            self.rebuild(&myself, &mut state).await?;
        } else {
            tracing::info!("away mode is inactive, no lights will be replayed");
        }

        Ok(state)
    }

    #[tracing::instrument(name = "away-actor", skip(self, myself, message, state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            AwayMessage::Arm(true) => {
                tracing::info!("away mode armed");
                state.armed = true;
                self.rebuild(&myself, state).await?;
            }
            AwayMessage::Arm(false) => {
                tracing::info!("away mode disarmed, dropping the plan");
                state.armed = false;
                state.plan.clear();
            }
            AwayMessage::Rebuild => {
                if state.armed {
                    self.rebuild(&myself, state).await?;
                } else {
                    tracing::info!("away: skipping rebuild while disarmed");
                }
            }
            AwayMessage::Fire { address, on } => {
                if state.armed {
                    self.fire(address, on).await?;
                } else {
                    tracing::info!("away: dropping queued action for {address} while disarmed");
                }
            }
        }

        Ok(())
    }
}
