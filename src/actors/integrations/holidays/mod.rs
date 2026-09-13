use ractor::Actor;

use crate::integrations::holidays::Holidays;
use crate::settings::HolidaySettings;
use crate::state::AppState;

pub mod lua;

pub enum HolidaysMessage {
    Poll,
}

pub struct HolidaysActor {
    pub shared_actor_state: AppState,
    pub holidays: Holidays,
}

impl HolidaysActor {
    pub const NAME: &str = "holidays";

    async fn poll(&self) -> Result<(), ractor::ActorProcessingErr> {
        let holidays = self.holidays.fetch().await?;

        if holidays.is_empty() {
            tracing::warn!("holiday calendar returned no events, leaving the stored set alone");
            return Ok(());
        }

        let stored = self
            .shared_actor_state
            .repos
            .holiday()
            .replace(&holidays)
            .await?;

        tracing::info!("stored {stored} holidays");

        Ok(())
    }
}

impl Actor for HolidaysActor {
    type Msg = HolidaysMessage;
    type State = ();
    type Arguments = HolidaySettings;

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        settings: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        myself.send_message(HolidaysMessage::Poll)?;
        myself.send_interval(settings.refresh.to_std()?, || HolidaysMessage::Poll);

        Ok(())
    }

    #[tracing::instrument(parent = None, name = "holidays-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            HolidaysMessage::Poll => {
                let started = std::time::Instant::now();

                match self.poll().await {
                    Ok(()) => {
                        crate::metrics::record_integration_poll(
                            "holidays",
                            "success",
                            started.elapsed(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("error polling holidays: {e}");
                        crate::metrics::record_integration_poll(
                            "holidays",
                            "error",
                            started.elapsed(),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
