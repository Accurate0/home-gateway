use std::collections::HashMap;

use super::WorkflowError;
use crate::settings::workflow_context::ContextSource;
use crate::state::AppState;

pub async fn resolve(
    state: &AppState,
    sources: &[ContextSource],
    vars: &HashMap<String, String>,
) -> Result<HashMap<String, String>, WorkflowError> {
    let mut resolved = vars.clone();

    for source in sources {
        match source {
            ContextSource::Fuelwatch => fuelwatch(state, &mut resolved).await?,
            ContextSource::Willyweather => willyweather(state, &mut resolved).await?,
        }
    }

    Ok(resolved)
}

async fn willyweather(
    state: &AppState,
    vars: &mut HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let location = &state.settings.willyweather.default_location;

    let forecast = state
        .repos
        .willyweather()
        .forecast(location)
        .await
        .map_err(anyhow::Error::from)?;

    let Some(today) = forecast.and_then(|forecast| forecast.days.into_iter().next()) else {
        tracing::warn!("no willyweather forecast stored for {location}");
        return Err(WorkflowError::ContextUnavailable(
            ContextSource::Willyweather.as_str(),
        ));
    };

    let unknown = || "n/a".to_owned();

    let sunset = today.sunset.as_deref().and_then(|sunset| {
        chrono::DateTime::parse_from_rfc3339(sunset)
            .ok()
            .map(|sunset| sunset.format("%H:%M").to_string())
    });

    vars.insert("forecast_description".to_owned(), today.description);
    vars.insert("forecast_emoji".to_owned(), today.emoji);
    vars.insert("forecast_min".to_owned(), today.min.to_string());
    vars.insert("forecast_max".to_owned(), today.max.to_string());
    vars.insert(
        "forecast_uv".to_owned(),
        today.uv.map_or_else(unknown, |uv| format!("{uv:.1}")),
    );
    vars.insert(
        "forecast_rain_probability".to_owned(),
        today
            .rain_probability
            .map_or_else(unknown, |probability| probability.to_string()),
    );
    vars.insert(
        "forecast_rain_range".to_owned(),
        today.rain_range_code.unwrap_or_else(unknown),
    );
    vars.insert(
        "forecast_wind_max_speed".to_owned(),
        today
            .wind_max_speed
            .map_or_else(unknown, |speed| format!("{speed:.0}")),
    );
    vars.insert("forecast_sunset".to_owned(), sunset.unwrap_or_else(unknown));

    Ok(())
}

async fn fuelwatch(
    state: &AppState,
    vars: &mut HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let Some(settings) = state.settings.fuelwatch.as_ref() else {
        tracing::warn!("fuelwatch context requested but fuelwatch is not configured");
        return Err(WorkflowError::ContextUnavailable(
            ContextSource::Fuelwatch.as_str(),
        ));
    };

    let sites = state
        .repos
        .fuelwatch()
        .sites_for_postcode(settings.postcode, Some(1))
        .await
        .map_err(anyhow::Error::from)?;

    let Some(cheapest) = sites.into_iter().next() else {
        tracing::warn!(
            "no fuelwatch sites stored for postcode {}",
            settings.postcode
        );
        return Err(WorkflowError::ContextUnavailable(
            ContextSource::Fuelwatch.as_str(),
        ));
    };

    vars.insert("fuel_price".to_owned(), format!("{:.1}", cheapest.price));
    vars.insert("fuel_brand".to_owned(), cheapest.brand);
    vars.insert("fuel_name".to_owned(), cheapest.name);
    vars.insert("fuel_suburb".to_owned(), cheapest.suburb);
    vars.insert("fuel_address".to_owned(), cheapest.address);

    Ok(())
}
