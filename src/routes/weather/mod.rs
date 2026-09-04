use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Query, extract::State};
use http::StatusCode;
use serde::Deserialize;

use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};
use crate::integrations::willyweather::WillyWeatherError;
use crate::integrations::willyweather::types::Forecast;
use crate::state::ApiState;

pub enum WeatherError {
    Forbidden,
    Upstream(WillyWeatherError),
}

impl IntoResponse for WeatherError {
    fn into_response(self) -> Response {
        match self {
            WeatherError::Forbidden => {
                tracing::warn!("weather forecast request missing the required scope");

                StatusCode::FORBIDDEN.into_response()
            }
            WeatherError::Upstream(e) => {
                tracing::error!("weather api error: {e:?}");

                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}

impl From<WillyWeatherError> for WeatherError {
    fn from(e: WillyWeatherError) -> Self {
        Self::Upstream(e)
    }
}

#[derive(Deserialize)]
pub struct ForecastQueryParams {
    location: Option<String>,
}

pub async fn forecast(
    Auth(auth): Auth,
    State(ApiState {
        willyweather,
        settings,
        ..
    }): State<ApiState>,
    params: Query<ForecastQueryParams>,
) -> Result<Json<Forecast>, WeatherError> {
    if auth
        .require(&Scope::new(Resource::Weather, Action::Read))
        .is_err()
    {
        return Err(WeatherError::Forbidden);
    }

    let location = params
        .location
        .clone()
        .unwrap_or_else(|| settings.willyweather.default_location.clone());

    Ok(Json(willyweather.forecast(&location).await?))
}
