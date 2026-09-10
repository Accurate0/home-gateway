//! Vacation mode: replays the home's own historical lighting while nobody is here.
//!
//! While one of the configured occupancy modes is active, the light history
//! aggregate is turned into a switching plan for the day (see
//! [`crate::vacation::plan`]) and each transition is armed as a one-shot timer, the
//! same shape as [`crate::actors::sun::SunActor`]. The plan is deterministic, so
//! a restart mid-day resumes it instead of re-rolling.

use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Australia::Perth;
use ractor::Actor;

use crate::actors::devices::light::{LightHandler, LightHandlerMessage};
use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::event_bus::{Recipient, Subscription};
use crate::state::AppState;
use crate::vacation::{PlannedAction, build_plan, target_at};

pub mod subscriber;

pub use subscriber::VacationSubscriber;

pub enum VacationMessage {
    Arm(bool),
    Rebuild,
    Fire { address: String, on: bool },
}

pub struct VacationActor {
    pub shared_actor_state: AppState,
}

#[derive(Default)]
pub struct VacationState {
    _subscription: Subscription,
    armed: bool,
    plan: Vec<PlannedAction>,
    timers: Vec<ractor::concurrency::JoinHandle<Result<(), ractor::MessagingErr<VacationMessage>>>>,
}

impl VacationState {
    fn cancel_timers(&mut self) {
        for timer in self.timers.drain(..) {
            timer.abort();
        }
    }
}

impl VacationActor {
    pub const NAME: &str = "vacation";

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&Perth).date_naive()
    }

    async fn armed_now(&self) -> bool {
        let mode = self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .current_mode()
            .await;

        self.shared_actor_state
            .settings
            .vacation
            .modes
            .contains(&mode)
    }

    /// Rebuild today's plan and arm a timer for every action still ahead of us,
    /// plus one at local midnight to roll onto the next day.
    async fn rebuild(
        &self,
        myself: &ractor::ActorRef<VacationMessage>,
        state: &mut VacationState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        state.cancel_timers();

        let settings = &self.shared_actor_state.settings.vacation;
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
            state
                .timers
                .push(myself.send_after(delay, move || VacationMessage::Fire { address, on }));
            armed += 1;
        }

        let midnight = day
            .succ_opt()
            .and_then(|next| next.and_hms_opt(0, 0, 0))
            .and_then(|local| Perth.from_local_datetime(&local).earliest())
            .map(|local| local.with_timezone(&Utc));

        if let Some(midnight) = midnight
            && let Ok(delay) = (midnight - now).to_std()
        {
            state
                .timers
                .push(myself.send_after(delay, || VacationMessage::Rebuild));
        }

        let mut addresses: Vec<&str> = state
            .plan
            .iter()
            .map(|action| action.address.as_str())
            .collect();

        addresses.sort_unstable();
        addresses.dedup();

        tracing::info!(
            "vacation plan for {day}: {} actions, {armed} still ahead across {} lights",
            state.plan.len(),
            addresses.len()
        );

        for address in addresses {
            let Some(on) = target_at(&state.plan, address, now) else {
                continue;
            };

            self.fire(address.to_owned(), on).await?;
        }

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
                "vacation: {address} is already {}",
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
                "vacation: dispatched {address} {}",
                if on { "on" } else { "off" }
            ),
            Err(e) => tracing::warn!("vacation: could not switch {address}: {e}"),
        }

        Ok(())
    }
}

impl Actor for VacationActor {
    type Msg = VacationMessage;
    type State = VacationState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let subscription = self.shared_actor_state.event_bus.register(
            Self::NAME,
            Recipient::Actor(myself.clone()),
            VacationSubscriber {
                settings: self.shared_actor_state.settings.clone(),
            },
        );

        let mut state = VacationState {
            _subscription: subscription,
            armed: self.armed_now().await,
            plan: Vec::new(),
            timers: Vec::new(),
        };

        if state.armed {
            tracing::info!("vacation mode is already active, building the replay plan");
            self.rebuild(&myself, &mut state).await?;
        } else {
            tracing::info!("vacation mode is inactive, no lights will be replayed");
        }

        Ok(state)
    }

    #[tracing::instrument(parent = None, name = "vacation-actor", skip(self, myself, message, state))]
    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            VacationMessage::Arm(true) => {
                tracing::info!("vacation mode armed");
                state.armed = true;
                self.rebuild(&myself, state).await?;
            }
            VacationMessage::Arm(false) if !state.armed => {
                tracing::debug!("vacation mode is already disarmed");
            }
            VacationMessage::Arm(false) => {
                tracing::info!("vacation mode disarmed, dropping the plan");
                state.armed = false;
                state.cancel_timers();
                state.plan.clear();
            }
            VacationMessage::Rebuild => {
                if state.armed {
                    self.rebuild(&myself, state).await?;
                } else {
                    tracing::info!("vacation: skipping rebuild while disarmed");
                }
            }
            VacationMessage::Fire { address, on } => {
                if state.armed {
                    self.fire(address, on).await?;
                } else {
                    tracing::info!("vacation: dropping queued action for {address} while disarmed");
                }
            }
        }

        Ok(())
    }
}
