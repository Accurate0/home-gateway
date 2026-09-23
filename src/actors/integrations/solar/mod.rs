pub mod lua;

use crate::{
    event_bus::{EventBusMessage, WeatherMetric, WeatherReading, WeatherSource},
    integrations::solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI},
    repo::solar::SolarReading,
    state::AppState,
};
use chrono::{DateTime, Utc};
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

    async fn cached_temperature(&self) -> Option<f64> {
        let location = &self
            .shared_actor_state
            .settings
            .integrations
            .willyweather
            .default_location;

        let forecast = match self
            .shared_actor_state
            .repos
            .willyweather()
            .forecast(location)
            .await
        {
            Ok(forecast) => forecast?,
            Err(e) => {
                tracing::error!("error reading the cached willyweather forecast: {e}");
                return None;
            }
        };

        let now = Utc::now();

        forecast
            .hours
            .iter()
            .filter_map(|hour| {
                let time = DateTime::parse_from_rfc3339(&hour.date_time).ok()?;

                Some(((now - time.with_timezone(&Utc)).abs(), hour.temperature?))
            })
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, temperature)| temperature)
    }

    async fn poll(&self) -> Result<Vec<WeatherReading>, ractor::ActorProcessingErr> {
        let solar = async {
            let login_data = self.goodwe.get_new_or_cached_login_data().await?;

            self.goodwe.get_solar_data(login_data).await
        };

        let (solar_data, uv_level, temperature) = tokio::join!(
            solar,
            self.weather.get_uv_level(WeatherAPI::PERTH_NAME),
            self.cached_temperature(),
        );

        let solar_data = solar_data?;

        let current_kwh = solar_data.data.kpi.pac;
        let raw_data = serde_json::to_value(&solar_data)?;

        tracing::debug!("fetched solar data: {current_kwh}");

        let uv_level = match uv_level {
            Ok(uv_level) => Some(uv_level),
            Err(e) => {
                tracing::error!("error getting uv level: {e}");
                None
            }
        };

        tracing::debug!("fetched uv level:{uv_level:?}, temperature: {temperature:?}");

        self.shared_actor_state
            .repos
            .solar()
            .append_reading(SolarReading {
                current_kwh,
                today_kwh: solar_data.data.kpi.power,
                month_kwh: solar_data.data.kpi.month_generation,
                total_kwh: solar_data.data.kpi.total_power,
                raw_data,
                uv_level,
                temperature,
            })
            .await?;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Solar {
                event_id: Uuid::new_v4(),
                current_wh: current_kwh,
            });

        let readings: Vec<WeatherReading> = temperature
            .map(|temperature| (WeatherMetric::Temperature, temperature))
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
            .integrations
            .solar
            .refresh
            .to_std()
            .unwrap_or(Duration::from_secs(60));

        myself.send_interval(refresh, || SolarMessage::Poll);

        Ok(Vec::new())
    }

    #[tracing::instrument(
        parent = None,
        name = "actor.solar",
        skip(self, _myself, message, state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
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
                        crate::tracing_context::record_current_error(&e.to_string());
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
