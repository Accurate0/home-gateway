use crate::{
    event_bus::EventBusMessage,
    integrations::solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI},
    state::SharedActorState,
};
use ractor::Actor;
use std::time::Duration;
use uuid::Uuid;

pub enum SolarMessage {
    Poll,
}

pub struct SolarActor {
    pub shared_actor_state: SharedActorState,
    pub goodwe: GoodWeSemsAPI,
    pub weather: WeatherAPI,
}

impl SolarActor {
    pub const NAME: &str = "solar";

    async fn poll(&self) -> Result<(), ractor::ActorProcessingErr> {
        let login_data = self.goodwe.get_new_or_cached_login_data().await?;
        let solar_data = self.goodwe.get_solar_data(login_data).await?;

        let current_kwh = solar_data.data.kpi.pac;
        let raw_data = serde_json::to_value(&solar_data)?;

        tracing::info!("fetched solar data: {current_kwh}");

        let uv_level = match self.weather.get_uv_level(WeatherAPI::PERTH_NAME).await {
            Ok(uv_level) => Some(uv_level),
            Err(e) => {
                tracing::error!("error getting uv level: {e}");
                None
            }
        };

        let temperature = match self
            .weather
            .get_weather_details(WeatherAPI::JANDAKOT_GEOCODE)
            .await
        {
            Ok(weather) => Some(weather.data.temp),
            Err(e) => {
                tracing::error!("error getting weather details: {e}");
                None
            }
        };

        tracing::info!("fetched uv level: {uv_level:?}, temperature: {temperature:?}");

        sqlx::query!(
            "INSERT INTO solar_data_tsdb (current_kwh, raw_data, uv_level, temperature) \
             VALUES ($1, $2, $3, $4)",
            current_kwh,
            raw_data,
            uv_level,
            temperature
        )
        .execute(&self.shared_actor_state.db)
        .await?;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Solar {
                event_id: Uuid::new_v4(),
                current_wh: current_kwh,
            });

        Ok(())
    }
}

impl Actor for SolarActor {
    type Msg = SolarMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let refresh = self
            .shared_actor_state
            .settings
            .solar
            .as_ref()
            .and_then(|s| s.refresh.to_std().ok())
            .unwrap_or(Duration::from_secs(60));

        myself.send_interval(refresh, || SolarMessage::Poll);

        Ok(())
    }

    #[tracing::instrument(name = "solar-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SolarMessage::Poll => {
                if let Err(e) = self.poll().await {
                    tracing::error!("error polling solar data: {e}");
                }
            }
        }

        Ok(())
    }
}
