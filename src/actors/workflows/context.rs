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
        }
    }

    Ok(resolved)
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
