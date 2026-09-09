use std::sync::Arc;

use open_feature::{
    EvaluationContext, EvaluationError, OpenFeature, StructValue, provider::NoOpProvider,
};
use openfeature_provider::{EvaluationMode, FeatureFlagProvider};

pub use openfeature_provider::ProviderEvent;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::event_bus::{EventBus, EventBusMessage, FeatureFlagState};

const RESUBSCRIBE_BACKOFF: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn publish_provider_events(client: FeatureFlagClient, event_bus: EventBus) {
    let publish = |state: FeatureFlagState, version: Option<String>| {
        tracing::info!(
            "feature flag provider is {} (version {})",
            state.as_str(),
            version.as_deref().unwrap_or("unknown")
        );

        event_bus.publish_detached(EventBusMessage::FeatureFlag {
            event_id: Uuid::new_v4(),
            state,
            version,
        });
    };

    publish(FeatureFlagState::Ready, None);

    let mut events = match client.subscribe() {
        Some(events) => events,
        None => {
            tracing::debug!("no live flag provider, publishing no further flag events");

            return;
        }
    };

    loop {
        match events.recv().await {
            Ok(ProviderEvent::Ready) => publish(FeatureFlagState::Ready, None),
            Ok(ProviderEvent::ConfigurationChanged { version }) => {
                publish(FeatureFlagState::Changed, Some(version.to_string()))
            }
            Ok(ProviderEvent::Stale) => publish(FeatureFlagState::Stale, None),
            Ok(ProviderEvent::Error) => publish(FeatureFlagState::Error, None),
            Err(RecvError::Lagged(n)) => {
                tracing::warn!("missed {n} flag events, treating it as a change");
                publish(FeatureFlagState::Changed, None);
            }
            Err(RecvError::Closed) => {
                tracing::warn!("flag event stream closed, resubscribing after backoff");
                tokio::time::sleep(RESUBSCRIBE_BACKOFF).await;

                match client.subscribe() {
                    Some(next) => {
                        events = next;
                        publish(FeatureFlagState::Changed, None);
                    }
                    None => {
                        tracing::error!("flag provider is gone, stopping the flag watcher");

                        return;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct FeatureFlagClient {
    client: Arc<open_feature::Client>,
    evaluation_context: EvaluationContext,
    events: Option<Arc<broadcast::Receiver<ProviderEvent>>>,
}

impl FeatureFlagClient {
    pub async fn new() -> Self {
        let url = std::env::var("FEATURE_FLAGS_URL").ok();

        let mut client = OpenFeature::singleton_mut().await;

        let events = if let Some(url) = url {
            match FeatureFlagProvider::connect_with(url, "home-gateway", EvaluationMode::Local)
                .await
            {
                Ok(provider) => {
                    let events = provider.events();
                    client.set_provider(provider).await;

                    Some(Arc::new(events))
                }
                Err(e) => {
                    tracing::error!("error when connecting to feature-flags: {e}");
                    client.set_provider(NoOpProvider::default()).await;

                    None
                }
            }
        } else {
            tracing::warn!("fallback to noop feature provider");
            client.set_provider(NoOpProvider::default()).await;

            None
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
            events,
        }
    }

    pub fn has_live_provider(&self) -> bool {
        self.events.is_some()
    }

    pub fn subscribe(&self) -> Option<broadcast::Receiver<ProviderEvent>> {
        self.events.as_ref().map(|events| events.resubscribe())
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

    pub async fn get_struct(
        &self,
        feature_flag: &'static str,
        mut evaluation_context: EvaluationContext,
    ) -> Result<StructValue, EvaluationError> {
        evaluation_context.merge_missing(&self.evaluation_context);

        self.client
            .get_struct_value::<StructValue>(feature_flag, Some(&evaluation_context), None)
            .await
    }
}
