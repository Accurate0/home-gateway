use std::time::{Duration, Instant};

use axum::Router;
use ractor::ActorRef;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

use crate::actors::root::RootMessage;
use crate::device_registry::DeviceRegistry;
use crate::eink::EinkDisplayManager;
use crate::error::MainError;
use crate::event_bus::EventBus;
use crate::integrations::esphome_native_api::{self, Node};
use crate::integrations::feature_flag::{self, FeatureFlagClient};
use crate::integrations::home_assistant::{self, HomeAssistant};
use crate::integrations::jellyfin::{self, Jellyfin};
use crate::integrations::mqtt::Mqtt;
use crate::settings::{EsphomeSettings, HomeAssistantWebsocketSettings, JellyfinWebsocketSettings};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Tasks {
    pub router: Router,
    pub mqtt: Mqtt,
    pub home_assistant: Option<HomeAssistant>,
    pub home_assistant_websocket: HomeAssistantWebsocketSettings,
    pub esphome: EsphomeSettings,
    pub esphome_nodes: Vec<Node>,
    pub jellyfin: Option<Jellyfin>,
    pub jellyfin_websocket: JellyfinWebsocketSettings,
    pub devices: DeviceRegistry,
    pub cancellation_token: CancellationToken,
    pub feature_flag_client: FeatureFlagClient,
    pub event_bus: EventBus,
    pub root_supervisor: ActorRef<RootMessage>,
    pub root_supervisor_handle: JoinHandle<()>,
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
        jellyfin,
        jellyfin_websocket,
        devices,
        cancellation_token,
        feature_flag_client,
        event_bus,
        root_supervisor,
        root_supervisor_handle,
        eink,
        started,
    } = tasks;

    tokio::spawn(async move { eink.prefill_packed().await });

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    tracing::info!("starting api server {listen_addr}");

    let mut task_set = JoinSet::new();

    let axum_cancellation_token = cancellation_token.child_token();
    task_set.spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(axum_cancellation_token.cancelled_owned())
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

    if let Some(jellyfin) = jellyfin {
        let jellyfin_cancellation_token = cancellation_token.child_token();
        task_set.spawn(async move {
            jellyfin::websocket::process_events(
                jellyfin,
                jellyfin_websocket,
                jellyfin_cancellation_token,
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

    let supervisor_cancellation_token = cancellation_token.child_token();
    task_set.spawn(async move {
        root_supervisor_handle.await?;

        match supervisor_cancellation_token.is_cancelled() {
            true => tracing::info!("the root supervisor stopped"),
            false => tracing::error!("the root supervisor stopped, shutting down"),
        }

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

    cancellation_token.cancel();

    tracing::info!("stopping the actor tree");

    if let Err(e) = root_supervisor
        .stop_and_wait(None, Some(SHUTDOWN_TIMEOUT))
        .await
    {
        tracing::error!("the root supervisor did not stop cleanly: {e}");
    }

    tracing::info!("waiting for all tasks to finish");

    let drained = tokio::time::timeout(SHUTDOWN_TIMEOUT, async {
        while task_set.join_next().await.is_some() {}
    })
    .await;

    if drained.is_err() {
        tracing::warn!(
            "tasks did not finish within {}s",
            SHUTDOWN_TIMEOUT.as_secs()
        );
    }

    task_set.shutdown().await;

    Ok(())
}
