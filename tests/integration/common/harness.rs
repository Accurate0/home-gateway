use std::path::Path;
use std::time::Duration;

use home_gateway::actors::root::RootMessage;
use home_gateway::actors::workflows::manager::WorkflowManager;
use home_gateway::api::{build_router, build_schema};
use home_gateway::auth::AuthManager;
use home_gateway::device_registry::DeviceRegistry;
use home_gateway::event_bus::{EventBus, EventBusMessage};
use home_gateway::integrations::feature_flag::FeatureFlagClient;
use home_gateway::integrations::reddit::Reddit;
use home_gateway::integrations::s3::S3;
use home_gateway::integrations::willyweather::WillyWeather;
use home_gateway::settings::SettingsContainer;
use home_gateway::state::{ApiState, SharedActorState};
use ractor::{Actor, ActorProcessingErr, ActorRef};
use sqlx::{Pool, Postgres};
use tokio::sync::broadcast;

use super::broker::{self, Recorder, TestBroker};
use super::db::{self, TestDb};

const FIXTURE_CONFIG: &str = "tests/fixtures/config";
const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Stands in for `RootSupervisor` so a test can link only the actors it needs
/// instead of the whole tree.
struct TestRoot;

impl Actor for TestRoot {
    type Msg = RootMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        _message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Ok(())
    }
}

pub struct Harness {
    pub state: SharedActorState,
    pub db: Pool<Postgres>,
    pub settings: SettingsContainer,
    pub devices: DeviceRegistry,
    pub event_bus: EventBus,
    pub recorder: Recorder,
    pub root: ActorRef<RootMessage>,
    _broker: TestBroker,
    _test_db: TestDb,
}

impl Harness {
    pub async fn start() -> Self {
        // S3 builds its credentials from the environment and the eink manager
        // holds one, so give it something to find.
        unsafe {
            std::env::set_var("AWS_ACCESS_KEY_ID", "test");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
        }

        let test_db = db::fresh_database().await;
        let db = test_db.pool.clone();

        let (settings, devices) = SettingsContainer::load_from_dir(Path::new(FIXTURE_CONFIG))
            .expect("failed to load the fixture config");
        let settings: SettingsContainer = settings.into();

        let test_broker = broker::start(devices.clone()).await;
        let recorder = test_broker.recorder.clone();

        let feature_flag_client = FeatureFlagClient::new().await;
        let s3 = S3::new(&settings.s3.bucket, &settings.s3.region, None)
            .expect("failed to build the s3 client");

        let eink = home_gateway::eink::EinkDisplayManager::new(
            db.clone(),
            s3.clone(),
            feature_flag_client.clone(),
            devices.clone(),
            settings.clone(),
            Reddit::new(),
        );

        let event_bus = EventBus::default();

        let state = SharedActorState {
            settings: settings.clone(),
            devices: devices.clone(),
            db: db.clone(),
            mqtt: test_broker.client.clone(),
            feature_flag_client,
            s3,
            event_bus: event_bus.clone(),
            workflows: WorkflowManager::new(db.clone()),
            home_assistant: None,
            jellyfin: None,
            transperth: None,
            eink,
        };

        let (root, _) = Actor::spawn(None, TestRoot, ())
            .await
            .expect("failed to spawn the test root supervisor");

        Self {
            state,
            db,
            settings,
            devices,
            event_bus,
            recorder,
            root,
            _broker: test_broker,
            _test_db: test_db,
        }
    }

    pub fn api_state(&self) -> ApiState {
        let http_client =
            home_gateway::http::get_traced_http_client().expect("failed to build the http client");

        let willyweather = WillyWeather::new(&self.settings.willyweather)
            .expect("failed to build the willyweather client");

        ApiState {
            feature_flag_client: self.state.feature_flag_client.clone(),
            schema: build_schema(&self.state, http_client, willyweather.clone(), None, None),
            settings: self.settings.clone(),
            db: self.db.clone(),
            s3: self.state.s3.clone(),
            auth: AuthManager::new(self.db.clone(), None),
            devices: self.devices.clone(),
            eink: self.state.eink.clone(),
            willyweather,
        }
    }

    pub fn router(&self) -> axum::Router {
        build_router(self.api_state(), prometheus::Registry::new())
    }

    pub fn subscribe(&self) -> EventStream {
        EventStream {
            receiver: self.event_bus.subscribe(),
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.root.stop(None);
    }
}

pub struct EventStream {
    receiver: broadcast::Receiver<EventBusMessage>,
}

impl EventStream {
    /// Drains the bus until `predicate` matches, so unrelated traffic from other
    /// actors doesn't break an assertion.
    pub async fn next_matching<F>(&mut self, label: &str, mut predicate: F) -> EventBusMessage
    where
        F: FnMut(&EventBusMessage) -> bool,
    {
        let deadline = tokio::time::Instant::now() + EVENT_TIMEOUT;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let received = tokio::time::timeout(remaining, self.receiver.recv())
                .await
                .unwrap_or_else(|_| panic!("timed out waiting for {label} on the event bus"));

            match received {
                Ok(message) if predicate(&message) => return message,
                Ok(_) => continue,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => panic!("event bus closed while waiting for {label}: {e}"),
            }
        }
    }
}
