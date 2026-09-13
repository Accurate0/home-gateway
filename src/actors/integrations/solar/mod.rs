pub mod lua;

use crate::{
    event_bus::{EventBusMessage, WeatherMetric, WeatherReading, WeatherSource},
    integrations::solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI},
    state::AppState,
};
use ractor::{Actor, RpcReplyPort};
use std::time::Duration;
use uuid::Uuid;

pub enum SolarMessage {
    Poll,
    LatestWeather {
        reply: RpcReplyPort<Vec<WeatherReading>>,
    },
}

pub struct SolarActor {
    pub shared_actor_state: AppState,
    pub goodwe: GoodWeSemsAPI,
    pub weather: WeatherAPI,
}

impl SolarActor {
    pub const NAME: &str = "solar";

    async fn poll(&self) -> Result<Vec<WeatherReading>, ractor::ActorProcessingErr> {
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

        let observation = match self
            .weather
            .get_weather_details(WeatherAPI::JANDAKOT_GEOCODE)
            .await
        {
            Ok(weather) => Some(weather.data),
            Err(e) => {
                tracing::error!("error getting weather details: {e}");
                None
            }
        };

        let temperature = observation.as_ref().map(|observation| observation.temp);

        tracing::info!("fetched uv level: {uv_level:?}, temperature: {temperature:?}");

        self.shared_actor_state
            .repos
            .solar()
            .append_reading(current_kwh, raw_data, uv_level, temperature)
            .await?;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Solar {
                event_id: Uuid::new_v4(),
                current_wh: current_kwh,
            });

        let observed = observation
            .map(|observation| {
                vec![
                    (WeatherMetric::Temperature, observation.temp),
                    (WeatherMetric::FeelsLike, observation.temp_feels_like),
                    (WeatherMetric::Humidity, observation.humidity as f64),
                    (
                        WeatherMetric::WindSpeed,
                        observation.wind.speed_kilometre as f64,
                    ),
                    (
                        WeatherMetric::GustSpeed,
                        observation.gust.speed_kilometre as f64,
                    ),
                    (
                        WeatherMetric::MaxGustSpeed,
                        observation.max_gust.speed_kilometre as f64,
                    ),
                    (WeatherMetric::RainSince9am, observation.rain_since_9am),
                    (WeatherMetric::MaxTemp, observation.max_temp.value),
                    (WeatherMetric::MinTemp, observation.min_temp.value),
                ]
            })
            .unwrap_or_default();

        let readings: Vec<WeatherReading> = observed
            .into_iter()
            .chain(uv_level.map(|uv| (WeatherMetric::Uv, uv)))
            .map(|(metric, value)| WeatherReading {
                metric,
                day: None,
                value,
            })
            .collect();

        if !readings.is_empty() {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Weather {
                    event_id: Uuid::new_v4(),
                    source: WeatherSource::Bom,
                    readings: readings.clone(),
                });
        }

        Ok(readings)
    }
}

impl Actor for SolarActor {
    type Msg = SolarMessage;
    type State = Vec<WeatherReading>;
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

        Ok(Vec::new())
    }

    #[tracing::instrument(parent = None, name = "solar-actor", skip(self, _myself, message, state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SolarMessage::LatestWeather { reply } => {
                reply.send(state.clone())?;
            }
            SolarMessage::Poll => {
                let started = std::time::Instant::now();
                match self.poll().await {
                    Ok(readings) => {
                        if !readings.is_empty() {
                            *state = readings;
                        }

                        crate::metrics::record_integration_poll(
                            "solar",
                            "success",
                            started.elapsed(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("error polling solar data: {e}");
                        crate::metrics::record_integration_poll(
                            "solar",
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
