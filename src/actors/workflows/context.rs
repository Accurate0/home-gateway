use super::WorkflowError;
use crate::integrations::fuelwatch::variables::FuelwatchVariables;
use crate::integrations::willyweather::day_variables::WillyweatherDayVariables;
use crate::integrations::willyweather::variables::WillyweatherVariables;
use crate::settings::workflow::ContextSource;
use crate::state::AppState;
use crate::variables::{Node, Vars, WorkflowContextVariables};

pub async fn resolve(
    state: &AppState,
    sources: &[ContextSource],
    vars: &mut Vars,
) -> Result<(), WorkflowError> {
    for source in sources {
        if vars.contains(source.as_str()) {
            continue;
        }

        let node = match source {
            ContextSource::Fuelwatch => fuelwatch(state).await?,
            ContextSource::Willyweather => willyweather(state).await?,
        };

        vars.insert(source.as_str(), node);
    }

    Ok(())
}

async fn willyweather(state: &AppState) -> Result<Node, WorkflowError> {
    let location = &state.settings.willyweather.default_location;

    let forecast = state
        .repos
        .willyweather()
        .forecast(location)
        .await
        .map_err(anyhow::Error::from)?;

    let mut days = forecast
        .map(|forecast| forecast.days)
        .unwrap_or_default()
        .into_iter();

    let (Some(today), Some(tomorrow)) = (days.next(), days.next()) else {
        tracing::warn!("willyweather forecast for {location} is missing today or tomorrow");
        return Err(WorkflowError::ContextUnavailable(
            ContextSource::Willyweather.as_str(),
        ));
    };

    let variables = WillyweatherVariables {
        today: WillyweatherDayVariables::from(today),
        tomorrow: WillyweatherDayVariables::from(tomorrow),
    };

    Ok(variables.to_node())
}

async fn fuelwatch(state: &AppState) -> Result<Node, WorkflowError> {
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

    Ok(FuelwatchVariables::from(cheapest).to_node())
}
