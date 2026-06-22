use serde::Deserialize;

/// GraphQL-facing config (loaded from `config/graphql.yaml`). Drives which
/// environment sensors the API exposes without hard-coding resolvers.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GraphqlSettings {
    /// Environment sensor ids (matching `environment_sensors[].id`) to expose.
    #[serde(default)]
    pub environments: Vec<String>,
}
