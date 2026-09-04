use super::task::AdhocTask;

pub mod convert_api_key_scopes;

pub fn all() -> Vec<&'static dyn AdhocTask> {
    vec![&convert_api_key_scopes::ConvertApiKeyScopes]
}
