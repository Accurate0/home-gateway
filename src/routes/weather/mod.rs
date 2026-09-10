use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Query, extract::State};
use http::StatusCode;
use serde::Deserialize;

use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};
use crate::integrations::willyweather::types::Forecast;
use crate::repo::willyweather::WillyWeatherRepoError;
use crate::state::AppState;

pub enum WeatherError {
    Forbidden,
    UnknownLocation(String),
    NotStored(String),
    Database(WillyWeatherRepoError),
}

impl IntoResponse for WeatherError {
    fn into_response(self) -> Response {
        match self {
            WeatherError::Forbidden => {
                tracing::warn!("weather forecast request missing the required scope");

                StatusCode::FORBIDDEN.into_response()
            }
            WeatherError::UnknownLocation(location) => {
                tracing::warn!("weather forecast requested for unknown location {location}");

                StatusCode::NOT_FOUND.into_response()
            }
            WeatherError::NotStored(location) => {
                tracing::warn!("no stored weather forecast for {location} yet");

                StatusCode::SERVICE_UNAVAILABLE.into_response()
            }
            WeatherError::Database(e) => {
                tracing::error!("weather forecast lookup error: {e:?}");

                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}

impl From<WillyWeatherRepoError> for WeatherError {
    fn from(e: WillyWeatherRepoError) -> Self {
        Self::Database(e)
    }
}

#[derive(Deserialize)]
pub struct ForecastQueryParams {
    location: Option<String>,
}

pub async fn forecast(
    Auth(auth): Auth,
    State(state): State<AppState>,
    params: Query<ForecastQueryParams>,
) -> Result<Json<Forecast>, WeatherError> {
    if auth
        .require(&Scope::new(Resource::Weather, Action::Read))
        .is_err()
    {
        return Err(WeatherError::Forbidden);
    }

    let settings = &state.settings.willyweather;
    let requested = params
        .location
        .as_deref()
        .unwrap_or(&settings.default_location);

    let Some(alias) = settings.resolve_location(requested) else {
        return Err(WeatherError::UnknownLocation(requested.to_owned()));
    };

    state
        .repos
        .willyweather()
        .forecast(alias)
        .await?
        .map(Json)
        .ok_or_else(|| WeatherError::NotStored(alias.to_owned()))
}
