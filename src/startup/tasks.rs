use axum::Router;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::device_registry::DeviceRegistry;
use crate::eink::EinkDisplayManager;
use crate::error::MainError;
use crate::event_bus::EventBus;
use crate::integrations::feature_flag::{self, FeatureFlagClient};
use crate::integrations::home_assistant::{self, HomeAssistant};
use crate::integrations::mqtt::Mqtt;
use crate::settings::HomeAssistantWebsocketSettings;
use crate::utils::axum_shutdown_signal;

pub struct Tasks {
    pub router: Router,
    pub mqtt: Mqtt,
    pub home_assistant: Option<HomeAssistant>,
    pub home_assistant_websocket: HomeAssistantWebsocketSettings,
    pub devices: DeviceRegistry,
    pub cancellation_token: CancellationToken,
    pub feature_flag_client: FeatureFlagClient,
    pub event_bus: EventBus,
    pub root_supervisor: JoinHandle<()>,
    pub eink: EinkDisplayManager,
}

pub async fn run(listen_addr: std::net::SocketAddr, tasks: Tasks) -> anyhow::Result<()> {
    let Tasks {
        router,
        mut mqtt,
        home_assistant,
        home_assistant_websocket,
        devices,
        cancellation_token,
        feature_flag_client,
        event_bus,
        root_supervisor,
        eink,
    } = tasks;

    tokio::spawn(async move { eink.prefill_packed().await });

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    tracing::info!("starting api server {listen_addr}");

    let mut task_set = JoinSet::new();

    task_set.spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(axum_shutdown_signal())
            .await
            .map_err(MainError::from)
    });

    let mqtt_cancellation_token = cancellation_token.child_token();
    task_set.spawn(async move {
        mqtt.process_events(mqtt_cancellation_token, devices)
            .await?;
        Ok::<(), MainError>(())
    });

    if let Some(home_assistant) = home_assistant {
        let home_assistant_cancellation_token = cancellation_token.child_token();
        task_set.spawn(async move {
            home_assistant::websocket::process_events(
                home_assistant,
                home_assistant_websocket,
                home_assistant_cancellation_token,
            )
            .await;
            Ok::<(), MainError>(())
        });
    }

    task_set.spawn(async move {
        feature_flag::publish_provider_events(feature_flag_client, event_bus).await;
        tracing::warn!("the feature flag watcher stopped");
        Ok::<(), MainError>(())
    });

    task_set.spawn(async move {
        root_supervisor.await?;
        tracing::error!("the root supervisor stopped, shutting down");
        Ok::<(), MainError>(())
    });

    tracing::info!("startup completed");

    if let Some(result) = task_set.join_next().await {
        match result {
            Ok(Ok(_)) => tracing::warn!("task ended without error"),
            Ok(Err(e)) => tracing::error!("task ended with {e}"),
            Err(e) => tracing::error!("join error: {e}"),
        }
    }

    tracing::info!("shutting down all tasks");
    task_set.shutdown().await;

    Ok(())
}
