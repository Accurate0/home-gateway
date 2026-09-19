use super::task::AdhocTask;

pub mod backfill_solar_kpis;
pub mod convert_api_key_scopes;

pub fn all() -> Vec<&'static dyn AdhocTask> {
    vec![
        &convert_api_key_scopes::ConvertApiKeyScopes,
        &backfill_solar_kpis::BackfillSolarKpis,
    ]
}
