use std::time::Duration;

use ractor::Actor;
use uuid::Uuid;

use crate::{
    event_bus::{EventBusMessage, ForecastDay, WeatherMetric, WeatherReading, WeatherSource},
    integrations::willyweather::{WillyWeather, types::Forecast},
    settings::WillyWeatherSettings,
    state::AppState,
};

pub enum WillyWeatherMessage {
    Poll,
}

pub struct WillyWeatherActor {
    pub shared_actor_state: AppState,
    pub willyweather: WillyWeather,
}

fn forecast_readings(forecast: &Forecast) -> Vec<WeatherReading> {
    ForecastDay::ALL
        .iter()
        .filter_map(|day| {
            forecast
                .days
                .get(day.index())
                .map(|details| (*day, details))
        })
        .flat_map(|(day, details)| {
            WeatherMetric::WILLYWEATHER
                .iter()
                .filter_map(move |metric| {
                    details.metric(*metric).map(|value| WeatherReading {
                        metric: *metric,
                        day: Some(day),
                        value,
                    })
                })
        })
        .collect()
}

impl WillyWeatherActor {
    pub const NAME: &str = "willyweather";

    async fn poll_location(
        &self,
        settings: &WillyWeatherSettings,
        alias: &str,
        location_id: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let forecast = self.willyweather.fetch(location_id, settings.days).await?;

        let repo = self.shared_actor_state.repos.willyweather();
        let mut tx = self.shared_actor_state.db.begin().await?;

        let stored = repo.replace_forecast(&mut tx, alias, &forecast).await?;
        let appended = repo.append_history(&mut tx, alias, &forecast).await?;

        tx.commit().await?;

        tracing::info!(
            "stored {stored} willyweather forecast rows for {alias} and appended {appended} history rows"
        );

        if alias != settings.default_location {
            return Ok(());
        }

        let readings = forecast_readings(&forecast);

        if readings.is_empty() {
            tracing::warn!("willyweather forecast for {alias} has no readings to publish");
        } else {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Weather {
                    event_id: Uuid::new_v4(),
                    source: WeatherSource::WillyWeather,
                    readings,
                });
        }

        Ok(())
    }
}

impl Actor for WillyWeatherActor {
    type Msg = WillyWeatherMessage;
    type State = WillyWeatherSettings;
    type Arguments = WillyWeatherSettings;

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        settings: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        myself.send_after(Duration::ZERO, || WillyWeatherMessage::Poll);
        myself.send_interval(settings.refresh.to_std()?, || WillyWeatherMessage::Poll);

        Ok(settings)
    }

    #[tracing::instrument(parent = None, name = "willyweather-actor", skip(self, _myself, message, settings))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        settings: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WillyWeatherMessage::Poll => {
                let started = std::time::Instant::now();
                let mut failed = false;

                for (alias, location_id) in &settings.locations {
                    match self.poll_location(settings, alias, location_id).await {
                        Ok(()) => {
                            tracing::debug!("polled willyweather forecast for {alias}");
                        }
                        Err(e) => {
                            tracing::error!("error polling willyweather for {alias}: {e}");
                            failed = true;
                        }
                    }
                }

                let outcome = if failed { "error" } else { "success" };

                crate::metrics::record_integration_poll("willyweather", outcome, started.elapsed());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::willyweather::types::ForecastDetails;

    fn day(max: i64, rain_probability: Option<i64>) -> ForecastDetails {
        ForecastDetails {
            date_time: String::new(),
            code: String::new(),
            description: String::new(),
            emoji: String::new(),
            min: 10,
            max,
            uv: None,
            rain_probability,
            rain_start_range: None,
            rain_end_range: None,
            rain_range_code: None,
            wind_max_speed: None,
            first_light: None,
            sunrise: None,
            sunset: None,
            last_light: None,
        }
    }

    #[test]
    fn readings_cover_today_and_tomorrow_and_skip_missing_values() {
        let forecast = Forecast {
            days: vec![day(30, Some(10)), day(35, None), day(40, Some(90))],
            hours: Vec::new(),
        };

        let readings: Vec<_> = forecast_readings(&forecast)
            .into_iter()
            .map(|reading| (reading.day, reading.metric, reading.value))
            .collect();

        assert_eq!(
            readings,
            [
                (Some(ForecastDay::Today), WeatherMetric::MaxTemp, 30.0),
                (Some(ForecastDay::Today), WeatherMetric::MinTemp, 10.0),
                (
                    Some(ForecastDay::Today),
                    WeatherMetric::RainProbability,
                    10.0
                ),
                (Some(ForecastDay::Tomorrow), WeatherMetric::MaxTemp, 35.0),
                (Some(ForecastDay::Tomorrow), WeatherMetric::MinTemp, 10.0),
            ]
        );
    }
}
