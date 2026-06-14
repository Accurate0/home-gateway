use std::sync::Arc;

use open_feature::{EvaluationContext, OpenFeature, provider::NoOpProvider};
use openfeature_provider::{EvaluationMode, FeatureFlagProvider};

#[derive(Clone)]
pub struct FeatureFlagClient {
    client: Arc<open_feature::Client>,
    evaluation_context: EvaluationContext,
}

impl FeatureFlagClient {
    pub async fn new() -> Self {
        let url = std::env::var("FEATURE_FLAGS_URL").ok();

        let mut client = OpenFeature::singleton_mut().await;

        if let Some(url) = url {
            match FeatureFlagProvider::connect_with(url, "home-gateway", EvaluationMode::Local)
                .await
            {
                Ok(provider) => client.set_provider(provider).await,
                Err(e) => {
                    tracing::error!("error when connecting to feature-flags: {e}");
                    client.set_provider(NoOpProvider::default()).await
                }
            };
        } else {
            tracing::warn!("fallback to noop feature provider");
            client.set_provider(NoOpProvider::default()).await;
        };

        let client = client.create_client();
        let evaluation_context = EvaluationContext::default().with_custom_field(
            "environment",
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        );

        Self {
            client: Arc::new(client),
            evaluation_context,
        }
    }

    pub async fn is_feature_enabled(
        &self,
        feature_flag: &'static str,
        default: bool,
        mut evaluation_context: EvaluationContext,
    ) -> bool {
        evaluation_context.merge_missing(&self.evaluation_context);

        let ff_eval_result = self
            .client
            .get_bool_value(feature_flag, Some(&evaluation_context), None)
            .await;

        match ff_eval_result {
            Ok(eval) => eval,
            Err(e) => {
                tracing::error!("error evaluating: {feature_flag} because {e:?}");
                default
            }
        }
    }
}
