use std::sync::{Arc, Mutex};
use std::time::Duration;

use home_gateway::device_registry::DeviceRegistry;
use home_gateway::integrations::mqtt::{Mqtt, MqttClient};
use testcontainers::{ContainerAsync, ImageExt, ReuseDirective, runners::AsyncRunner};
use testcontainers_modules::mosquitto::Mosquitto;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

use super::wait_for;

const CONTAINER_NAME: &str = "home-gateway-test-broker";
const RECORDER_CAPACITY: usize = 100;
const PUBLISH_TIMEOUT: Duration = Duration::from_secs(10);

static BROKER: OnceCell<Broker> = OnceCell::const_new();

struct Broker {
    host: String,
    port: u16,
    _container: ContainerAsync<Mosquitto>,
}

type Published = Vec<(String, Vec<u8>)>;

#[derive(Clone, Default)]
pub struct Recorder {
    messages: Arc<Mutex<Published>>,
}

impl Recorder {
    fn record(&self, topic: String, payload: Vec<u8>) {
        self.messages.lock().unwrap().push((topic, payload));
    }

    pub fn published(&self) -> Published {
        self.messages.lock().unwrap().clone()
    }

    pub async fn expect_publish(&self, topic: &str) -> serde_json::Value {
        let payload = wait_for(
            PUBLISH_TIMEOUT,
            &format!("a publish on {topic}"),
            || async {
                self.published()
                    .into_iter()
                    .find(|(published, _)| published == topic)
                    .map(|(_, payload)| payload)
            },
        )
        .await;

        serde_json::from_slice(&payload).unwrap_or_else(|e| {
            panic!(
                "publish on {topic} was not valid json ({e}): {}",
                String::from_utf8_lossy(&payload)
            )
        })
    }

    pub fn assert_no_publish(&self, topic: &str) {
        let published = self.published();
        assert!(
            !published.iter().any(|(published, _)| published == topic),
            "expected no publish on {topic}, got {:?}",
            published.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
    }
}

async fn broker() -> &'static Broker {
    BROKER
        .get_or_init(|| async {
            let container = Mosquitto::default()
                .with_container_name(CONTAINER_NAME)
                .with_reuse(ReuseDirective::Always)
                .start()
                .await
                .expect("failed to start the mosquitto container");

            let host = container
                .get_host()
                .await
                .expect("failed to resolve the mosquitto host")
                .to_string();
            let port = container
                .get_host_port_ipv4(1883)
                .await
                .expect("failed to resolve the mapped mosquitto port");

            Broker {
                host,
                port,
                _container: container,
            }
        })
        .await
}

pub struct TestBroker {
    pub client: MqttClient,
    pub recorder: Recorder,
    cancellation_token: CancellationToken,
}

impl TestBroker {
    pub fn shutdown(&self) {
        self.cancellation_token.cancel();
    }
}

impl Drop for TestBroker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Connects the gateway's own `Mqtt` client to a real broker and, alongside it,
/// a second subscriber on `#` that records everything the gateway publishes.
pub async fn start(devices: DeviceRegistry) -> TestBroker {
    let broker = broker().await;
    let cancellation_token = CancellationToken::new();

    let (client, mut mqtt) = Mqtt::new(
        broker.host.clone(),
        broker.port,
        String::new(),
        String::new(),
    )
    .await
    .expect("failed to build the gateway mqtt client");

    let ingest_token = cancellation_token.child_token();
    tokio::spawn(async move {
        if let Err(e) = mqtt.process_events(ingest_token, devices).await {
            tracing::error!("test mqtt event loop stopped: {e}");
        }
    });

    let recorder = start_recorder(broker, cancellation_token.child_token()).await;

    TestBroker {
        client,
        recorder,
        cancellation_token,
    }
}

async fn start_recorder(broker: &Broker, cancellation_token: CancellationToken) -> Recorder {
    let mut options =
        rumqttc::MqttOptions::new("integration-recorder", broker.host.clone(), broker.port);
    options.set_keep_alive(Duration::from_secs(5));
    options.set_max_packet_size(100_000, 100_000);

    let (client, mut connection) = rumqttc::AsyncClient::new(options, RECORDER_CAPACITY);
    let recorder = Recorder::default();
    let sink = recorder.clone();

    let (subscribed_tx, subscribed_rx) = tokio::sync::oneshot::channel();
    let mut subscribed_tx = Some(subscribed_tx);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => return,
                event = connection.poll() => match event {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => {
                        if let Err(e) = client.subscribe("#", rumqttc::QoS::AtLeastOnce).await {
                            tracing::error!("recorder failed to subscribe: {e}");
                        }
                    }
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::SubAck(_))) => {
                        if let Some(tx) = subscribed_tx.take() {
                            let _ = tx.send(());
                        }
                    }
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                        sink.record(publish.topic, publish.payload.to_vec());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("recorder connection error: {e}");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    });

    tokio::time::timeout(PUBLISH_TIMEOUT, subscribed_rx)
        .await
        .expect("timed out waiting for the recorder to subscribe")
        .expect("the recorder task stopped before subscribing");

    recorder
}
