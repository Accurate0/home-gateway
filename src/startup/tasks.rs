use std::time::Instant;

use axum::Router;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::device_registry::DeviceRegistry;
use crate::eink::EinkDisplayManager;
use crate::error::MainError;
use crate::event_bus::EventBus;
use crate::integrations::esphome_native_api::{self, Node};
use crate::integrations::feature_flag::{self, FeatureFlagClient};
use crate::integrations::home_assistant::{self, HomeAssistant};
use crate::integrations::mqtt::Mqtt;
use crate::settings::{EsphomeSettings, HomeAssistantWebsocketSettings};
use crate::utils::axum_shutdown_signal;

pub struct Tasks {
    pub router: Router,
    pub mqtt: Mqtt,
    pub home_assistant: Option<HomeAssistant>,
    pub home_assistant_websocket: HomeAssistantWebsocketSettings,
    pub esphome: EsphomeSettings,
    pub esphome_nodes: Vec<Node>,
    pub devices: DeviceRegistry,
    pub cancellation_token: CancellationToken,
    pub feature_flag_client: FeatureFlagClient,
    pub event_bus: EventBus,
    pub root_supervisor: JoinHandle<()>,
    pub eink: EinkDisplayManager,
    pub started: Instant,
}

pub async fn run(listen_addr: std::net::SocketAddr, tasks: Tasks) -> anyhow::Result<()> {
    let Tasks {
        router,
        mut mqtt,
        home_assistant,
        home_assistant_websocket,
        esphome,
        esphome_nodes,
        devices,
        cancellation_token,
        feature_flag_client,
        event_bus,
        root_supervisor,
        eink,
        started,
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

    for node in esphome_nodes {
        let Some(key) = esphome.encryption_key.clone() else {
            tracing::error!(
                "esphome node {} has no encryption key, skipping it",
                node.address
            );
            continue;
        };

        let esphome = esphome.clone();
        let node_cancellation_token = cancellation_token.child_token();

        task_set.spawn(async move {
            esphome_native_api::process_events(node, esphome, key, node_cancellation_token).await;
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

    tracing::info!("startup completed in {}ms", started.elapsed().as_millis());

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
