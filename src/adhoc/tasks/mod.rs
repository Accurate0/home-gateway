use super::seal::SealedTask;

pub mod backfill_solar_kpis;
pub mod convert_api_key_scopes;
pub mod rekey_device_ids;

pub fn all() -> Vec<SealedTask> {
    vec![
        SealedTask::locked(&convert_api_key_scopes::ConvertApiKeyScopes),
        SealedTask::locked(&backfill_solar_kpis::BackfillSolarKpis),
        SealedTask::unlocked(&rekey_device_ids::RekeyDeviceIds),
    ]
}
