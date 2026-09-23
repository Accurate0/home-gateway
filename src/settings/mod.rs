use config::builder::{ConfigBuilder, DefaultState};
use config::{Config, ConfigError, Environment, File, FileFormat};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

pub mod actor_workers;
pub mod actors;
pub mod adhoc;
pub mod adhoc_cron_task;
pub mod adhoc_cron_task_config;
pub mod adhoc_tasks;
pub mod alarm;
pub mod api_key;
pub mod auth;
pub mod backoff;
pub mod cache;
pub mod database;
pub mod device_scope;
pub mod devices;
pub mod enabled_state;
pub mod endpoint;
pub mod graphql;
pub mod http;
pub mod http_client;
pub mod http_client_kind;
pub mod http_client_override;
pub mod http_clients;
pub mod ingest;
pub mod integrations;
pub mod location;
pub mod lua;
pub mod mqtt;
pub mod mqtt_protocol;
pub mod mqtt_protocols;
pub mod no_parameters;
pub mod notify;
pub mod notify_android;
pub mod notify_filter;
pub mod notify_source;
pub mod oauth;
pub mod oauth_cache;
pub mod reconciler;
pub mod restart;
pub mod retention_parameters;
pub mod sampling;
pub mod sun;
pub mod tracing_settings;
pub mod vacation;
pub mod watchdog;
pub mod workflow;

pub use actor_workers::ActorWorkerSettings;
pub use actors::ActorSettings;
pub use adhoc::AdhocSettings;
pub use alarm::AlarmSettings;
pub use api_key::ApiKeySettings;
pub use auth::AuthSettings;
pub use backoff::BackoffSettings;
pub use cache::CacheSettings;
pub use database::DatabaseSettings;
pub use devices::device::{BatterySettings, DeviceWatchdog, RawDeviceWatchdog};
pub use devices::door::{ArmedDoorStates, DoorSettings};
pub use devices::eink::{
    Album, DashboardView, EinkDisplaySettings, EinkGlobalSettings, EinkMode, EinkModeConfig,
    Orientation, PartialRefresh, RawEinkDisplayBlock, RedditFeed, RedditTimespan, SleepWindow,
};
pub use devices::eink_defaults::EinkDefaults;
pub use devices::environment::{EnvironmentSensorSettings, Metric, RawEnvironmentBlock};
pub use devices::light::RawLightBlock;
pub use devices::media_player::{MediaPlayerSettings, RawMediaPlayerBlock};
pub use devices::plant::{PlantSensorSettings, RawPlantBlock};
pub use devices::presence::{PresenceSettings, RawPresenceBlock};
pub use devices::robot_vacuum::{RawRobotVacuumBlock, RobotVacuumSettings, VacuumCommands};
pub use devices::switch::{RawSmartSwitchBlock, SwitchRole};
pub use devices::trmnl::{RawTrmnlBlock, TrmnlDeviceSettings};
pub use endpoint::{Endpoint, EndpointSettings, ParamType};
pub use graphql::GraphqlSettings;
pub use http::HttpSettings;
pub use http_client::HttpClientSettings;
pub use http_client_kind::HttpClientKind;
pub use http_client_override::HttpClientOverride;
pub use http_clients::HttpClientsSettings;
pub use ingest::{IngestSettings, IngestSource};
pub use integrations::esphome::EsphomeSettings;
pub use integrations::fuelwatch::FuelWatchSettings;
pub use integrations::holidays::HolidaySettings;
pub use integrations::home_assistant::{EntitySettings, HomeAssistantSettings};
pub use integrations::home_assistant_websocket::HomeAssistantWebsocketSettings;
pub use integrations::integration_settings::IntegrationSettings;
pub use integrations::raw_integration_settings::RawIntegrationSettings;
pub use integrations::s3::S3Settings;
pub use integrations::solar::SolarSettings;
pub use integrations::transperth::{
    PeakWindow, RawTransperthSettings, TransperthRoute, TransperthSettings,
};
pub use integrations::transperth_cache::TransperthCacheSettings;
pub use integrations::trmnl::TrmnlSettings;
pub use integrations::willyweather::WillyWeatherSettings;
pub use integrations::woolworths::WoolworthsSettings;
pub use location::LocationSettings;
pub use lua::LuaSettings;
pub use mqtt::MqttSettings;
pub use mqtt_protocol::MqttProtocolSettings;
pub use mqtt_protocols::MqttProtocols;
pub use notify::{
    NotifyAcknowledge, NotifyAction, NotifyActionKind, NotifyCategory, NotifySource, NotifyTargets,
    RawNotifySettings, validate_acknowledge,
};
pub use notify_android::NotifyAndroidSettings;
pub use notify_filter::NotifyFilter;
pub use notify_source::NotificationSource;
pub use oauth::OAuthSettings;
pub use oauth_cache::OAuthCacheSettings;
pub use reconciler::ReconcilerSettings;
pub use restart::RestartSettings;
pub use sampling::SamplingSettings;
pub use sun::SunSettings;
pub use tracing_settings::TracingSettings;
pub use vacation::VacationSettings;
pub use watchdog::WatchdogSettings;
pub use workflow::{
    ReusableWorkflow, TriggerMatcher, Workflow, WorkflowDefinition, WorkflowSettings,
};

use crate::auth::scope::ScopePattern;
use crate::decoding::ModelSources;
use crate::device_registry::{DeviceRegistry, RawDevice};
use crate::settings::device_scope::DeviceScope;

pub type IEEEAddress = String;

/// Named device aliases (`alias -> ieee address`) declared under the top-level
/// `devices:` key. Referenced from workflow steps so addresses are written once.
pub type DeviceAliases = HashMap<String, IEEEAddress>;

/// Validate a workflow device reference. References must be device registry ids;
/// the id is kept as-is and resolved to an address at runtime. Unknown ids are
/// rejected at load time so typos fail loudly, while references to a device
/// configured as `state: disabled` are recorded so the workflow can be disabled.
pub(crate) fn validate_device(reference: &str, devices: &DeviceScope) -> Result<(), String> {
    devices.validate(reference)
}

/// Default for `#[serde(default = ...)]` flags that are opt-out (default `true`).
pub(crate) fn yes() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub version: String,
    pub api_key: String,
    pub database_url: String,
    pub database: DatabaseSettings,
    pub http: HttpSettings,
    pub graphql: GraphqlSettings,
    pub actors: ActorSettings,
    pub tracing: TracingSettings,
    pub mqtt: MqttSettings,
    pub notify: NotifyFilter,
    pub notify_android: NotifyAndroidSettings,
    pub workflows: HashMap<String, WorkflowDefinition>,
    pub workflow: WorkflowSettings,
    pub reconciler: ReconcilerSettings,
    pub integrations: IntegrationSettings,
    pub watchdog: WatchdogSettings,
    pub auth: AuthSettings,
    pub location: LocationSettings,
    pub sun: SunSettings,
    pub alarm: AlarmSettings,
    pub home_assistant: HomeAssistantSettings,
    pub eink_display: EinkGlobalSettings,
    pub adhoc: AdhocSettings,
    pub vacation: VacationSettings,
    pub lua: LuaSettings,
    pub ingest: IngestSettings,
    pub endpoints: EndpointSettings,
    pub model_sources: ModelSources,
}

/// On-disk shape of the config. Deserialized first, then [`RawSettings::resolve`]
/// resolves device aliases / notify targets and unifies sensor keying so the rest
/// of the app only ever sees the fully-resolved [`Settings`].
#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct RawSettings {
    #[serde(default)]
    version: String,
    api_key: String,
    database_url: String,
    database: DatabaseSettings,
    http: HttpSettings,
    graphql: GraphqlSettings,
    actors: ActorSettings,
    tracing: TracingSettings,
    mqtt: MqttSettings,
    notify: RawNotifySettings,
    #[serde(default)]
    devices: Vec<Vec<RawDevice>>,
    #[serde(default)]
    workflows: Vec<Vec<WorkflowDefinition>>,
    integrations: RawIntegrationSettings,
    watchdog: WatchdogSettings,
    workflow: WorkflowSettings,
    reconciler: ReconcilerSettings,
    auth: AuthSettings,
    #[serde(default)]
    location: LocationSettings,
    sun: SunSettings,
    #[serde(default)]
    alarm: AlarmSettings,
    home_assistant: HomeAssistantSettings,
    eink_display: devices::eink::RawEinkGlobal,
    adhoc: AdhocSettings,
    vacation: VacationSettings,
    #[serde(default)]
    lua: LuaSettings,
    #[serde(default)]
    ingest: IngestSettings,
    #[serde(default)]
    endpoints: EndpointSettings,
}

impl RawSettings {
    fn resolve(self, model_sources: &ModelSources) -> Result<(Settings, DeviceRegistry), String> {
        let RawSettings {
            version,
            api_key,
            database_url,
            database,
            http,
            graphql,
            actors,
            tracing: tracing_settings,
            mqtt,
            notify,
            devices,
            workflows,
            integrations,
            watchdog,
            workflow,
            reconciler,
            auth,
            location,
            sun,
            alarm,
            home_assistant,
            eink_display,
            adhoc,
            vacation,
            lua,
            ingest,
            endpoints,
        } = self;

        adhoc.validate()?;
        endpoints.validate()?;
        actors.workers.validate()?;

        let integrations = integrations.resolve()?;

        tracing_settings.sampling.validate()?;

        let mut seen_key_names = HashSet::new();
        for key in &auth.api_keys {
            if !seen_key_names.insert(key.name.clone()) {
                return Err(format!("duplicate api_keys name: {}", key.name));
            }

            validate_scopes(&format!("api key '{}'", key.name), &key.scopes)?;
        }

        if let Some(oauth) = &auth.oauth {
            for (group, scopes) in &oauth.group_scopes {
                validate_scopes(&format!("oauth group '{group}'"), scopes)?;
            }
        }

        mqtt.protocols.validate()?;

        let models = model_sources.load(&lua, &mqtt.protocols)?;

        let registry = DeviceRegistry::build(
            devices.into_iter().flatten().collect(),
            &notify.targets,
            &models,
            &mqtt.protocols,
        )?;
        let scope = DeviceScope::new(registry.aliases(), registry.disabled());

        let mut resolved = HashMap::new();
        let mut scopes = HashMap::new();
        let mut slugs = HashSet::new();
        let mut disabled_by_device = HashSet::new();
        for mut workflow in workflows.into_iter().flatten() {
            workflow.resolve_devices(&scope)?;

            let disabled_devices = scope.take_disabled();
            let references_disabled = !disabled_devices.is_empty();

            if references_disabled {
                let referenced = disabled_devices.into_iter().collect::<Vec<_>>().join(", ");
                let body = workflow.body_mut();

                body.enabled = false;

                tracing::warn!(
                    "workflow '{}' references disabled devices [{referenced}], disabling it",
                    body.name
                );

                disabled_by_device.insert(body.name.clone());
            } else {
                workflow.body().validate_capabilities(&registry)?;
            }

            let body = workflow.body();

            if let Some(triggered) = workflow.triggered() {
                if triggered.modes.is_empty() {
                    return Err(format!(
                        "workflow '{}' has an empty `modes:`",
                        triggered.name
                    ));
                }

                if triggered.hold.is_some() && !triggered.on.supports_hold() {
                    return Err(format!(
                        "workflow '{}' uses `for:` but its trigger is not a state that can be held",
                        triggered.name
                    ));
                }
            }

            let scope = if references_disabled {
                crate::variables::Scope::default()
            } else {
                let scope = workflow::scope::scope_for(&workflow, &registry)?;
                workflow::scope::check_steps(body, &scope)?;

                scope
            };

            if body.slug.trim().is_empty() {
                return Err(format!("workflow '{}' has an empty slug", body.name));
            }
            if !slugs.insert(body.slug.clone()) {
                return Err(format!("duplicate workflow slug: {}", body.slug));
            }
            let name = body.name.clone();
            scopes.insert(name.clone(), scope);
            if resolved.insert(name.clone(), workflow).is_some() {
                return Err(format!("duplicate workflow name: {name}"));
            }
        }

        for workflow in resolved
            .values()
            .map(WorkflowDefinition::body)
            .filter(|workflow| !disabled_by_device.contains(&workflow.name))
        {
            workflow::scope::check_calls(workflow, &scopes[&workflow.name], &resolved)?;

            workflow
                .validate_acknowledgements()
                .map_err(|e| format!("workflow '{}': {e}", workflow.name))?;
        }

        if let Some(definition) = resolved.get(&alarm.workflow) {
            let inputs = workflow::scope::callable_inputs(definition)
                .map_err(|error| format!("alarm workflow {error}"))?;

            if !inputs.is_empty() {
                return Err(format!(
                    "alarm workflow '{}' needs inputs the alarm cannot pass",
                    alarm.workflow
                ));
            }
        }

        Ok((
            Settings {
                version,
                api_key,
                database_url,
                database,
                http,
                graphql,
                actors,
                tracing: tracing_settings,
                mqtt,
                notify: NotifyFilter::new(&notify.disabled),
                notify_android: notify.android,
                workflows: resolved,
                integrations,
                watchdog,
                workflow,
                reconciler,
                auth,
                location,
                sun,
                alarm,
                home_assistant,
                eink_display: eink_display.resolve(),
                adhoc,
                vacation,
                lua,
                ingest,
                endpoints,
                model_sources: model_sources.clone(),
            },
            registry,
        ))
    }
}

fn validate_scopes(owner: &str, scopes: &[String]) -> Result<(), String> {
    for scope in scopes {
        if let Err(e) = ScopePattern::parse(scope) {
            return Err(format!("{owner} has invalid scope '{scope}': {e}"));
        }
    }

    Ok(())
}

#[derive(Clone)]
pub struct SettingsContainer {
    inner: Arc<Settings>,
}

impl SettingsContainer {
    fn build(config: Config, dir: &Path) -> Result<(Settings, DeviceRegistry), ConfigError> {
        let raw: RawSettings = config.try_deserialize()?;

        let load = |kind: &str, relative: &Path| {
            let path = dir.join(relative);

            crate::lua::sources::load_directory(&path).map_err(|e| {
                ConfigError::Message(format!(
                    "failed to load {kind} models from {}: {e}",
                    path.display()
                ))
            })
        };

        let sources = ModelSources {
            mqtt: load("mqtt", &raw.mqtt.models)?,
            esphome_native_api: load("esphome_native_api", &raw.integrations.esphome.models)?,
            home_assistant: load("home_assistant", &raw.home_assistant.models)?,
            library: match &raw.lua.model_library {
                Some(relative) => load("library", relative)?,
                None => BTreeMap::new(),
            },
        };

        raw.resolve(&sources).map_err(ConfigError::Message)
    }

    /// Build a config from a directory of YAML files. `base.yaml` is the entry
    /// point; its `!include <file>` tags (resolved by the `yaml-include` crate)
    /// pull in the per-domain files (`plants.yaml`, `switches.yaml`, …) so each
    /// concern lives in its own file.
    fn config_sources(dir: &Path) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
        let base = dir.join("base.yaml");
        if !base.is_file() {
            return Err(ConfigError::Message(format!(
                "config entry point not found: {}",
                base.display()
            )));
        }
        let transformer = yaml_include::Transformer::new(base.clone(), true).map_err(|e| {
            ConfigError::Message(format!(
                "failed to process includes in {}: {e}",
                base.display()
            ))
        })?;

        let merged =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| transformer.to_string()))
                .map_err(|panic| {
                    let reason = panic
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| panic.downcast_ref::<&str>().copied())
                        .unwrap_or("unknown include error");

                    ConfigError::Message(format!(
                        "failed to process includes in {}: {reason}",
                        base.display()
                    ))
                })?;

        Ok(Config::builder().add_source(File::from_str(&merged, FileFormat::Yaml)))
    }

    pub fn load_from_dir(dir: &Path) -> Result<(Settings, DeviceRegistry), ConfigError> {
        let config = Self::config_sources(dir)?
            .add_source(Environment::default().separator("__"))
            .build()?;

        Self::build(config, dir)
    }

    pub fn override_dir() -> PathBuf {
        std::env::var("CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/etc/home-gateway/config"))
    }

    pub fn baked_dir() -> PathBuf {
        PathBuf::from("./config")
    }

    pub fn new() -> Result<(Self, DeviceRegistry), ConfigError> {
        let override_dir = Self::override_dir();
        let baked_dir = Self::baked_dir();

        let (source, (settings, registry)) = match Self::load_from_dir(&override_dir) {
            Ok(loaded) => ("override", loaded),
            Err(e) => {
                tracing::warn!(
                    config_dir = %override_dir.display(),
                    error = %e,
                    "failed to load config from override dir, falling back to baked-in config"
                );
                ("baked-in", Self::load_from_dir(&baked_dir)?)
            }
        };

        tracing::info!(
            source,
            version = %settings.version,
            "loaded config"
        );

        Ok((
            Self {
                inner: Arc::new(settings),
            },
            registry,
        ))
    }
}

impl From<Settings> for SettingsContainer {
    fn from(settings: Settings) -> Self {
        Self {
            inner: Arc::new(settings),
        }
    }
}

impl std::ops::Deref for SettingsContainer {
    type Target = Settings;

    fn deref(&self) -> &Settings {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoding::{DeviceModels, DeviceRoleName, ReadingMetric};
    use crate::device_registry::{Capability, RawDevice};
    use crate::event_bus::SensorMetric;
    use crate::integrations::mqtt::MqttProtocol;
    use std::collections::BTreeMap;

    fn lamp_registry() -> DeviceRegistry {
        let devices: Vec<RawDevice> = serde_yaml::from_str(
            r#"
- id: living-room-table-lamp
  state: enabled
  transport:
    type: mqtt
    address: "0xa4c1389fe5cea26e"
  model: ts011f_plug
  roles:
    - type: smart_switch
      config: { name: Living Room Table Lamp, as: light }
"#,
        )
        .unwrap();

        DeviceRegistry::build(
            devices,
            &NotifyTargets::default(),
            &test_models(),
            &test_protocols(),
        )
        .unwrap()
    }

    fn test_protocols() -> MqttProtocols {
        MqttProtocols::committed()
    }

    fn test_models() -> DeviceModels {
        let sources = ModelSources {
            mqtt: BTreeMap::from([
                (
                    "ts011f_plug".to_owned(),
                    include_str!("../../config/lua/mqtt/ts011f_plug.lua").to_owned(),
                ),
                (
                    "apollo_mtr_1".to_owned(),
                    include_str!("../../config/lua/mqtt/apollo_mtr_1.lua").to_owned(),
                ),
                (
                    "valetudo".to_owned(),
                    include_str!("../../config/lua/mqtt/valetudo.lua").to_owned(),
                ),
            ]),
            esphome_native_api: BTreeMap::from([(
                "apollo_cast_1".to_owned(),
                include_str!("../../config/lua/esphome_native_api/apollo_cast_1.lua").to_owned(),
            )]),
            home_assistant: BTreeMap::from([
                (
                    "roborock".to_owned(),
                    include_str!("../../config/lua/home_assistant/roborock.lua").to_owned(),
                ),
                (
                    "media_player".to_owned(),
                    include_str!("../../config/lua/home_assistant/media_player.lua").to_owned(),
                ),
            ]),
            library: BTreeMap::from([
                (
                    "esphome_light".to_owned(),
                    include_str!("../../config/lua/model_lib/esphome_light.lua").to_owned(),
                ),
                (
                    "zigbee_switch".to_owned(),
                    include_str!("../../config/lua/model_lib/zigbee_switch.lua").to_owned(),
                ),
            ]),
        };

        sources
            .load(&LuaSettings::default(), &test_protocols())
            .unwrap()
    }

    #[test]
    fn a_home_assistant_device_without_a_model_is_rejected() {
        let err = build_devices(
            r#"
- id: roborock
  state: enabled
  transport:
    type: home_assistant
    address: vacuum.robot
  roles:
    - type: robot_vacuum
      config: { name: Roborock }
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("home_assistant transport requires a `model:`"),
            "{err}"
        );
    }

    #[test]
    fn a_home_assistant_entity_can_only_belong_to_one_device() {
        let err = build_devices(
            r#"
- id: roborock
  state: enabled
  transport:
    type: home_assistant
    address: vacuum.robot
  model: roborock
  roles:
    - type: robot_vacuum
      config: { name: Roborock }

- id: impostor
  state: enabled
  transport:
    type: home_assistant
    address: sensor.robot_status
  model: media_player
  roles: []
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("`sensor.robot_status` is claimed by another device"),
            "{err}"
        );
    }

    #[test]
    fn a_device_rejects_roles_its_transport_or_model_cannot_carry() {
        for (yaml, expected) in [
            (
                r#"
- id: robot
  state: enabled
  transport: { type: mqtt, address: rockrobo }
  model: valetudo
  roles:
    - type: robot_vacuum
      config: { name: Vacuum }
    - type: environment
      config: { id: robot, name: Robot }
"#,
                "has no `environment` mapping",
            ),
            (
                r#"
- id: node
  state: enabled
  transport: { type: mqtt, address: apollo-mtr-1-livingroom }
  model: apollo_mtr_1
  roles:
    - type: door
      config: { name: Node Door, id: node, state: unarmed }
"#,
                "has no `door` mapping",
            ),
            (
                r#"
- id: fridge
  state: enabled
  transport: { type: trmnl, address: "653VZN" }
  roles:
    - type: trmnl
      config: { name: Fridge }
    - type: door
      config: { name: Fridge Door, id: fridge, state: unarmed }
"#,
                "can't declare the `door` role",
            ),
        ] {
            let err = build_devices(yaml).unwrap_err();

            assert!(err.contains(expected), "{err}");
        }
    }

    #[test]
    fn a_one_to_one_transport_needs_its_own_role() {
        let err = build_devices(
            r#"
- id: fridge
  state: enabled
  transport: { type: trmnl, address: "653VZN" }
  roles:
    - type: battery
"#,
        )
        .unwrap_err();

        assert!(err.contains("must declare the `trmnl` role"), "{err}");
    }

    #[test]
    fn a_valetudo_vacuum_takes_its_commands_from_the_model() {
        let registry = build_devices(
            r#"
- id: valetudo
  state: enabled
  transport: { type: mqtt, address: rockrobo }
  model: valetudo
  roles:
    - type: robot_vacuum
      config: { name: Vacuum }
    - type: battery
"#,
        )
        .unwrap();

        let vacuum = registry.robot_vacuum("rockrobo").expect("vacuum resolves");

        assert_eq!(vacuum.commands.dock, "return_to_base");
        assert_eq!(vacuum.address, "rockrobo");
        assert_eq!(
            registry.mqtt_topics_for("rockrobo"),
            ["valetudo/rockrobo/attributes", "valetudo/rockrobo/state"]
        );
    }

    #[test]
    fn a_role_declared_twice_is_rejected() {
        let err = build_devices(
            r#"
- id: lamp
  state: enabled
  transport: { type: mqtt, address: "0xabc" }
  model: ts011f_plug
  roles:
    - type: smart_switch
      config: { name: Lamp }
    - type: smart_switch
      config: { name: Lamp again }
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("declares the `smart_switch` role twice"),
            "{err}"
        );
    }

    #[test]
    fn two_devices_cannot_share_an_address() {
        let err = build_devices(
            r#"
- id: lamp
  state: enabled
  transport: { type: mqtt, address: "0xabc" }
  model: ts011f_plug
  roles: []

- id: lamp-again
  state: enabled
  transport: { type: mqtt, address: "0xabc" }
  model: ts011f_plug
  roles: []
"#,
        )
        .unwrap_err();

        assert!(err.contains("already used by device lamp"), "{err}");
    }

    #[test]
    fn id_for_address_is_the_reverse_of_the_alias() {
        let registry = lamp_registry();

        assert_eq!(
            registry.id_for_address("0xa4c1389fe5cea26e"),
            Some("living-room-table-lamp")
        );
        assert_eq!(registry.id_for_address("0xnope"), None);
    }

    #[test]
    fn a_home_assistant_device_cannot_declare_a_zigbee_only_role() {
        let err = build_devices(
            r#"
- id: roborock
  state: enabled
  transport:
    type: home_assistant
    address: vacuum.robot
  model: roborock
  roles:
    - type: light
      config: { name: Robot Light }
"#,
        )
        .unwrap_err();

        assert!(err.contains("can't declare the `light` role"), "{err}");
    }

    fn build_devices(yaml: &str) -> Result<DeviceRegistry, String> {
        let devices: Vec<RawDevice> = serde_yaml::from_str(yaml).unwrap();

        DeviceRegistry::build(
            devices,
            &NotifyTargets::default(),
            &test_models(),
            &test_protocols(),
        )
    }

    #[test]
    fn an_mqtt_device_without_a_model_is_rejected() {
        let err = build_devices(
            r#"
- id: mystery
  state: enabled
  transport:
    type: mqtt
    address: "0xdeadbeef"
  roles:
    - type: control_switch
"#,
        )
        .unwrap_err();

        assert!(err.contains("requires a `model:`"), "{err}");
    }

    #[test]
    fn a_media_player_on_the_wrong_transport_is_rejected() {
        let err = build_devices(
            r#"
- id: tv
  state: enabled
  transport:
    type: mqtt
    address: living-room-tv
  model: apollo_mtr_1
  roles:
    - type: media_player
      config:
        name: TV
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("`mqtt` device can't declare the `media_player` role"),
            "{err}"
        );
    }

    #[test]
    fn a_media_player_address_that_is_not_an_entity_id_is_rejected() {
        let err = build_devices(
            r#"
- id: tv
  state: enabled
  transport:
    type: home_assistant
    address: living_room_tv
  model: media_player
  roles:
    - type: media_player
      config:
        name: TV
"#,
        )
        .unwrap_err();

        assert!(err.contains("must be a home assistant"), "{err}");
    }

    #[test]
    fn an_unknown_model_slug_is_rejected_and_lists_known_models() {
        let err = build_devices(
            r#"
- id: mystery
  state: enabled
  transport:
    type: mqtt
    address: "0xdeadbeef"
  model: not_a_real_model
  roles:
    - type: control_switch
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("unknown mqtt model `not_a_real_model`"),
            "{err}"
        );
        assert!(err.contains("ts011f_plug"), "{err}");
    }

    #[test]
    fn a_role_the_model_does_not_map_is_rejected() {
        let err = build_devices(
            r#"
- id: mystery
  state: enabled
  transport:
    type: mqtt
    address: "0xdeadbeef"
  model: ts011f_plug
  roles:
    - type: door
      config: { name: Mystery Door, id: mystery, state: armed, timeout: 3m }
"#,
        )
        .unwrap_err();

        assert!(err.contains("has no `door` mapping"), "{err}");
    }

    #[test]
    fn a_model_on_a_transport_without_decoders_is_rejected() {
        let err = build_devices(
            r#"
- id: fridge
  state: enabled
  transport:
    type: trmnl
    address: "653VZN"
  model: ts011f_plug
  roles:
    - type: trmnl
      config: { name: Fridge }
"#,
        )
        .unwrap_err();

        assert!(err.contains("does not take a `model:`"), "{err}");
    }

    #[test]
    fn a_model_from_another_transport_is_unknown() {
        let err = build_devices(
            r#"
- id: living-room-mtr-1
  state: enabled
  transport:
    type: home_assistant
    address: sensor.living_room
  model: ts011f_plug
  roles:
    - type: presence
      config: { name: Living Room }
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("unknown home_assistant model `ts011f_plug`"),
            "{err}"
        );
    }

    #[test]
    fn entity_lists_on_a_role_are_rejected() {
        let err = serde_yaml::from_str::<Vec<RawDevice>>(
            r#"
- id: living-room-mtr-1
  state: enabled
  transport:
    type: mqtt
    address: apollo-mtr-1-livingroom
  model: apollo_mtr_1
  roles:
    - type: light
      config: { name: Living Room MTR-1 RGB, entity: rgb_light }
"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("entity"), "{err}");
    }

    fn light_step(state: &str) -> workflow::Step {
        serde_yaml::from_str(&format!(
            "type: light\ndevice: living-room-table-lamp\nstate: {state}\nvalue: 50\n"
        ))
        .unwrap()
    }

    #[test]
    fn switch_as_light_accepts_on_off_but_not_brightness() {
        let registry = lamp_registry();

        light_step("TOGGLE")
            .validate_capabilities(&registry)
            .unwrap();

        let err = light_step("SET_BRIGHTNESS")
            .validate_capabilities(&registry)
            .unwrap_err();
        assert!(err.contains("does not support Brightness"), "{err}");

        let err = light_step("INCREASE_COLOUR_TEMPERATURE")
            .validate_capabilities(&registry)
            .unwrap_err();
        assert!(err.contains("does not support ColourTemp"), "{err}");
    }

    fn switch_step(device: &str) -> workflow::Step {
        serde_yaml::from_str(&format!("type: switch\ndevice: {device}\nstate: ON\n")).unwrap()
    }

    #[test]
    fn a_switch_step_needs_a_switch_declared_as_a_light() {
        switch_step("living-room-table-lamp")
            .validate_capabilities(&lamp_registry())
            .unwrap();

        let plain_plug = build_devices(
            r#"
- id: living-room-table-lamp
  state: enabled
  transport:
    type: mqtt
    address: "0xa4c1389fe5cea26e"
  model: ts011f_plug
  roles:
    - type: smart_switch
      config: { name: Living Room Table Lamp }
"#,
        )
        .unwrap();

        let err = switch_step("living-room-table-lamp")
            .validate_capabilities(&plain_plug)
            .unwrap_err();
        assert!(err.contains("cannot be driven"), "{err}");
    }

    #[test]
    fn tmp_hallway_epd_registers_battery() {
        let secrets = r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
http:
  listen_address: "[::]:8000"
integrations:
  esphome:
    encryption_key: x
  willyweather:
    api_key: x
  transperth:
    reference_data_api_key: x
"#;
        let config = SettingsContainer::config_sources(Path::new("./config"))
            .unwrap()
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        let (_settings, registry) =
            SettingsContainer::build(config, Path::new("./config")).unwrap();

        assert!(registry.eink_display("94a990cf8384").is_some(), "eink");
        assert!(registry.battery("94a990cf8384").is_some(), "battery");
        assert_eq!(registry.address_or_self("94a990cf8384"), "94a990cf8384");
        assert_eq!(registry.address_or_self("hallway-epd"), "94a990cf8384");
    }

    #[test]
    fn config_yaml_parses_and_resolves() {
        // secrets normally come from the environment; supply dummies for the test
        let secrets = r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
http:
  listen_address: "[::]:8000"
integrations:
  esphome:
    encryption_key: x
  willyweather:
    api_key: x
  transperth:
    reference_data_api_key: x
"#;
        let config = SettingsContainer::config_sources(Path::new("./config"))
            .unwrap()
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        let (settings, registry) = SettingsContainer::build(config, Path::new("./config")).unwrap();

        let switch_workflow = settings
            .workflows
            .values()
            .filter_map(WorkflowDefinition::triggered)
            .find(|w| {
                matches!(&w.on, TriggerMatcher::Switch { ieee_addr, action }
                if ieee_addr == "small-switch" && action.as_deref() == Some("single"))
            })
            .expect("expected a switch workflow for the small switch");
        assert!(
            switch_workflow
                .run
                .iter()
                .all(|step| matches!(step, workflow::Step::Lua { .. })),
            "expected the small switch to toggle through lua"
        );
        for name in ["living-room-lamps-off", "living-room-lamps-on"] {
            assert!(
                settings.workflows.contains_key(name),
                "small switch toggles unknown workflow {name}"
            );
        }

        assert_eq!(
            registry.address_or_self("small-switch"),
            "0x00158d008bbe0316"
        );
        assert_eq!(
            registry.address_or_self("floor-lamp-living-room"),
            "0x94a081fffe2eedc0"
        );

        assert!(
            settings
                .workflows
                .values()
                .filter_map(WorkflowDefinition::triggered)
                .any(|w| w.name == "Bins" && matches!(w.on, TriggerMatcher::Cron { .. }))
        );

        let roborock_address = registry.address_or_self("roborock");
        let roborock = registry
            .robot_vacuum(roborock_address)
            .expect("roborock device resolves");
        assert_eq!(registry.room(roborock_address), Some("dining-room"));
        assert_eq!(roborock.address, "vacuum.robot");
        assert_eq!(roborock.commands.start, "vacuum.start");
        assert_eq!(roborock.commands.stop, "vacuum.stop");
        assert_eq!(roborock.commands.dock, "vacuum.return_to_base");

        let valetudo_address = registry.address_or_self("valetudo");
        let valetudo = registry
            .robot_vacuum(valetudo_address)
            .expect("valetudo device resolves");
        assert_eq!(registry.room(valetudo_address), Some("spare-room"));
        assert_eq!(valetudo.address, "rockrobo");

        let tv_address = registry.address_or_self("living-room-tv");
        let tv = registry
            .media_player(tv_address)
            .expect("living room tv resolves");
        assert_eq!(tv_address, "media_player.living_room_tv");
        assert_eq!(tv.id, "living-room-tv");
        assert_eq!(tv.name, "Living Room TV");
        assert_eq!(tv.address, "media_player.living_room_tv");
        assert_eq!(registry.room(tv_address), Some("living-room"));

        assert!(
            registry
                .battery(registry.address_or_self("front-door"))
                .is_some(),
            "front-door has a battery kind"
        );
        assert!(
            registry.battery(roborock_address).is_some(),
            "roborock has a battery kind"
        );
        assert!(
            registry.battery(valetudo_address).is_some(),
            "valetudo has a battery kind"
        );
        assert!(
            registry
                .battery(registry.address_or_self("closet-light"))
                .is_none(),
            "mains-powered light has no battery kind"
        );

        for entity_id in [
            "vacuum.robot",
            "sensor.robot_battery",
            "sensor.robot_status",
            "binary_sensor.robot_water_shortage",
        ] {
            let device = registry
                .home_assistant_device(entity_id)
                .unwrap_or_else(|| panic!("{entity_id} routes to a decoded device"));

            assert_eq!(device.id, "roborock");
            assert_eq!(device.address, "vacuum.robot");
            assert_eq!(device.profile.slug, "roborock");
        }

        assert_eq!(
            registry
                .home_assistant_device("media_player.living_room_tv")
                .map(|device| device.profile.slug.clone()),
            Some("media_player".to_owned())
        );

        let mut seen = HashSet::new();
        for wf in settings.workflows.values().map(WorkflowDefinition::body) {
            assert!(
                !wf.slug.trim().is_empty(),
                "workflow '{}' has empty slug",
                wf.name
            );
            assert!(
                seen.insert(&wf.slug),
                "duplicate workflow slug: {}",
                wf.slug
            );
        }

        // a smart switch declared `as: light` is addressable as both
        let lamp = "0xa4c1389fe5cea26e";
        assert_eq!(registry.address_or_self("living-room-table-lamp"), lamp);
        assert_eq!(
            registry.smart_switch(lamp).map(String::as_str),
            Some("Living Room Table Lamp")
        );
        assert_eq!(
            registry.light(lamp).map(String::as_str),
            Some("Living Room Table Lamp")
        );
        // ...but with no capabilities, so it is on/off/toggle only
        assert!(registry.capabilities(lamp).is_empty());

        // an esphome light is addressable and routes by its own command topic
        let mtr = "apollo-mtr-1-livingroom";
        assert_eq!(
            registry.light(mtr).map(String::as_str),
            Some("Living Room MTR-1 RGB")
        );
        assert_eq!(registry.esphome_light(mtr), Some("rgb_light"));
        assert!(registry.mqtt_device(MqttProtocol::Esphome, mtr).is_some());
        // it has no colour temperature, so those workflow steps are rejected
        assert!(!registry.capabilities(mtr).contains(&Capability::ColourTemp));
        assert!(registry.capabilities(mtr).contains(&Capability::Rgb));

        // a zigbee presence sensor is keyed by its address in the registry
        assert!(registry.presence("0x54ef441000dbc81c").is_some());

        assert!(registry.presence(mtr).is_some());
        assert!(registry.environment(mtr).is_some());

        assert_eq!(
            registry.mqtt_topics_for(mtr),
            [
                "apollo-mtr-1-livingroom/binary_sensor/ld2450_moving_target/state",
                "apollo-mtr-1-livingroom/binary_sensor/ld2450_presence/state",
                "apollo-mtr-1-livingroom/binary_sensor/ld2450_still_target/state",
                "apollo-mtr-1-livingroom/light/rgb_light/state",
                "apollo-mtr-1-livingroom/sensor/dps310_pressure/state",
                "apollo-mtr-1-livingroom/sensor/dps310_temperature/state",
                "apollo-mtr-1-livingroom/sensor/ltr390_light/state",
            ]
        );

        assert_eq!(
            registry.sensor_metrics(mtr),
            [
                SensorMetric::from(Metric::Temperature),
                SensorMetric::from(Metric::Pressure),
                SensorMetric::from(Metric::Lux),
            ]
        );

        assert!(registry.plant("apollo-plt-1b-livingroom").is_some());

        let subscriptions = registry.mqtt_subscriptions();

        for topic in [
            "apollo-mtr-1-livingroom/binary_sensor/ld2450_presence/state",
            "zigbee2mqtt/+",
            "zigbee2mqtt/bridge/devices",
            "esphome/discover/+",
            "valetudo/rockrobo/state",
            "valetudo/rockrobo/attributes",
        ] {
            assert!(
                subscriptions.contains(topic),
                "missing subscription {topic}"
            );
        }

        let front = registry
            .mqtt_device(MqttProtocol::Zigbee, registry.address_or_self("front-door"))
            .expect("front-door is a zigbee device");
        assert_eq!(front.profile.slug, "aqara_mccgq12lm");
        let front_address = registry.address_or_self("front-door");
        assert!(
            registry.door(front_address).is_some() && registry.battery(front_address).is_some()
        );
        assert!(front.profile.roles.contains(&DeviceRoleName::Door));
        assert!(front.profile.roles.contains(&DeviceRoleName::Battery));

        let outdoor = registry
            .mqtt_device(
                MqttProtocol::Zigbee,
                registry.address_or_self("env-outdoor"),
            )
            .expect("env-outdoor is a zigbee device");
        assert!(outdoor.profile.environment.contains(&Metric::Temperature));

        let presence = registry
            .mqtt_device(MqttProtocol::Zigbee, "0x54ef441000dbc81c")
            .expect("closet presence is a zigbee device");
        let decoder = settings
            .model_sources
            .decoder(crate::device_registry::Transport::Mqtt, &settings.lua)
            .expect("mqtt decoder");
        let reading = presence
            .profile
            .decode(
                &decoder,
                &serde_json::json!({
                    "topic": "report",
                    "vars": { "name": "closet" },
                    "payload": { "presence": true, "movement": "approach" },
                }),
            )
            .expect("closet presence decodes");
        assert_eq!(
            reading.metrics.get("movement"),
            Some(&ReadingMetric::Text("approach".to_owned()))
        );

        assert!(
            registry
                .mqtt_device(MqttProtocol::Zigbee, "apollo-mtr-1-livingroom")
                .is_none()
        );

        // watchdog keys are `transport:device_id`, resolvable by address or id
        assert_eq!(
            registry.watchdog_key("0x54ef441000d2b0b0"),
            Some("zigbee:front-door")
        );
        assert_eq!(
            registry.watchdog_key("front-door"),
            Some("zigbee:front-door")
        );
        assert_eq!(
            registry.watchdog_key("apollo-mtr-1-livingroom"),
            Some("esphome:livingroom-motion")
        );
        assert_eq!(
            registry.watchdog_key("media_player.living_room_tv"),
            Some("home_assistant:living-room-tv")
        );
        assert_eq!(
            registry.watchdog_key("roborock"),
            Some("home_assistant:roborock")
        );
        assert_eq!(registry.watchdog_key("rockrobo"), Some("valetudo:valetudo"));
        assert_eq!(
            registry.watchdog_key("e83dc1fb1c98"),
            Some("eink_display_firmware:living-room-epd")
        );
        assert_eq!(registry.watchdog_key("653VZN"), Some("trmnl:fridge-trmnl"));
        assert_eq!(registry.watchdog_key("0xdeadbeef"), None);

        // every device is watched, and under the same key the heartbeat writes
        let watched: Vec<&String> = registry.watchdog_devices().map(|(key, _)| key).collect();
        assert_eq!(
            watched.len(),
            22,
            "every enabled device has a watchdog (23 configured, 1 disabled)"
        );
        for key in watched {
            assert!(
                key.contains(':'),
                "watchdog key {key} is not in transport:device_id form"
            );
        }
    }

    #[test]
    fn api_keys_parse_and_validate_scopes() {
        let secrets = r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
http:
  listen_address: "[::]:8000"
integrations:
  esphome:
    encryption_key: x
  willyweather:
    api_key: x
  transperth:
    reference_data_api_key: x
"#;
        let config = SettingsContainer::config_sources(Path::new("./config"))
            .unwrap()
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        let (settings, _registry) =
            SettingsContainer::build(config, Path::new("./config")).unwrap();
        assert!(
            settings
                .auth
                .api_keys
                .iter()
                .any(|k| k.name == "eink-display-living-room" && k.scopes == ["epd:read"]),
            "expected the eink config key to parse"
        );
    }

    #[test]
    fn api_keys_reject_an_invalid_scope() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
auth:
  api_key_cache: { capacity: 1024, ttl: 1h }
  api_keys:
    - name: bad-key
      scopes: ["bogus:read"]
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(err.contains("unknown resource `bogus`"), "{err}");
    }

    #[test]
    fn oauth_group_scopes_are_validated() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
auth:
  api_key_cache: { capacity: 1024, ttl: 1h }
  oauth:
    issuer: i
    jwks_url: j
    userinfo_url: u
    audience: a
    cache:
      keys: { capacity: 32, ttl: 1h }
      keys_refresh_cooldown: 1m
      userinfo: { capacity: 256, ttl: 15m }
    group_scopes:
      admins@idm: ["graphql:solar:read"]
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(
            err.contains("oauth group 'admins@idm' has invalid scope"),
            "{err}"
        );
    }

    fn secrets_over_config(secrets: &str) -> Result<(Settings, DeviceRegistry), ConfigError> {
        let base = r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  username: x
  password: x
integrations:
  esphome:
    encryption_key: x
  willyweather:
    api_key: x
  transperth:
    reference_data_api_key: x
"#;

        let config = SettingsContainer::config_sources(Path::new("./config"))
            .unwrap()
            .add_source(File::from_str(base, FileFormat::Yaml))
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        SettingsContainer::build(config, Path::new("./config"))
    }

    #[test]
    fn an_out_of_range_sampling_ratio_is_rejected() {
        let err = secrets_over_config("tracing: { sampling: { default: 1.5 } }")
            .map(|_| ())
            .unwrap_err();

        assert!(
            err.to_string().contains("tracing.sampling.default"),
            "{err}"
        );
    }

    #[test]
    fn a_device_watchdog_inherits_its_model_timeout() {
        let (_settings, registry) = secrets_over_config("").unwrap();

        let watchdog = |key: &str| {
            registry
                .watchdog_devices()
                .find(|(device_key, _)| device_key.as_str() == key)
                .map(|(_, watchdog)| watchdog.clone())
                .unwrap_or_else(|| panic!("{key} is watched"))
        };

        let garage = watchdog("zigbee:garage-door");
        assert_eq!(garage.timeout, Some(chrono::TimeDelta::hours(24)));
        assert!(garage.notify.is_empty());

        let front = watchdog("zigbee:front-door");
        assert_eq!(front.timeout, Some(chrono::TimeDelta::hours(24)));
        assert_eq!(front.notify.len(), 1);

        let display = watchdog("eink_display_firmware:hallway-epd");
        assert_eq!(display.timeout, Some(chrono::TimeDelta::hours(12)));
    }

    #[test]
    fn fuelwatch_postcode_and_location_fall_back_to_perth() {
        let (settings, _registry) = secrets_over_config("").unwrap();

        assert_eq!(settings.integrations.fuelwatch.postcode, 6000);
        assert_eq!(settings.location.latitude, -31.952429);
        assert_eq!(settings.location.longitude, 115.842283);
    }

    #[test]
    fn http_clients_fall_back_to_the_default_timeout() {
        let clients: HttpClientsSettings = serde_yaml::from_str(
            "{ default: { timeout: 7s }, fuelwatch: { timeout: 2s }, bom: {} }",
        )
        .unwrap();

        assert_eq!(
            clients.timeout_for(HttpClientKind::FuelWatch),
            std::time::Duration::from_secs(2)
        );
        assert_eq!(
            clients.timeout_for(HttpClientKind::Bom),
            std::time::Duration::from_secs(7)
        );
    }

    #[test]
    fn run_workflow_rejects_an_unknown_target() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Caller
      slug: caller
      inputs: {}
      run:
        - type: run_workflow
          workflow: does-not-exist
          with: {}
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(err.contains("does-not-exist"), "{err}");
    }

    #[test]
    fn run_workflow_accepts_a_known_target() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Callee
      slug: callee
      inputs: {}
      run: []
    - name: Caller
      slug: caller
      inputs: {}
      run:
        - type: run_workflow
          workflow: Callee
          with: {}
"#,
        )
        .unwrap();

        raw.resolve(&ModelSources::default())
            .expect("a known target resolves");
    }

    fn raw_with_workflows(workflows: &str) -> RawSettings {
        let base = r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
"#;

        serde_yaml::from_str(&format!("{base}\nworkflows:\n{workflows}")).unwrap()
    }

    fn resolve_error(workflows: &str) -> String {
        raw_with_workflows(workflows)
            .resolve(&ModelSources::default())
            .unwrap_err()
    }

    #[test]
    fn an_unknown_template_variable_is_rejected() {
        let err = resolve_error(
            r#"
  - - name: Cron notify
      slug: cron-notify
      on: { type: cron, schedule: "0 13 * * TUE" }
      modes: [home]
      run:
        - type: notify
          notify: { type: android_app }
          category: general
          message: "${event.bogus}"
"#,
        );

        assert!(err.contains("unknown variable `event.bogus`"), "{err}");
        assert!(err.contains("event.name"), "{err}");
    }

    #[test]
    fn an_optional_variable_without_a_default_is_rejected() {
        let err = resolve_error(
            r#"
  - - name: Low battery
      slug: low-battery
      on: { type: device_battery, below: 3.4 }
      modes: [home]
      run:
        - type: notify
          notify: { type: android_app }
          category: general
          message: "${event.name} at ${event.battery_percent}%"
"#,
        );

        assert!(err.contains("may be missing"), "{err}");
    }

    #[test]
    fn a_reusable_workflow_must_declare_inputs() {
        let err = resolve_error(
            r#"
  - - name: Callee
      slug: callee
      run: []
"#,
        );

        assert!(err.contains("must declare `inputs:`"), "{err}");
    }

    #[test]
    fn a_workflow_reading_event_variables_cannot_be_run_workflow_target() {
        let err = resolve_error(
            r#"
  - - name: Cron notify
      slug: cron-notify
      on: { type: cron, schedule: "0 13 * * TUE" }
      modes: [home]
      run:
        - type: notify
          notify: { type: android_app }
          category: general
          message: "${event.name}"
    - name: Caller
      slug: caller
      inputs: {}
      run:
        - type: run_workflow
          workflow: Cron notify
          with: {}
"#,
        );

        assert!(err.contains("reads `event.*`"), "{err}");
    }

    #[test]
    fn run_workflow_inputs_are_type_checked() {
        let workflows = |with: &str| {
            format!(
                r#"
  - - name: Callee
      slug: callee
      inputs: {{ count: int }}
      run:
        - type: notify
          notify: {{ type: android_app }}
          category: general
          message: "count ${{input.count}}"
    - name: Caller
      slug: caller
      on: {{ type: woolworths }}
      modes: [home]
      run:
        - type: run_workflow
          workflow: Callee
          with: {with}
"#
            )
        };

        let err = resolve_error(&workflows(r#"{ count: "${event.name}" }"#));
        assert!(err.contains("is a string but the input is a int"), "{err}");

        let err = resolve_error(&workflows("{}"));
        assert!(err.contains("missing input `count`"), "{err}");

        let err = resolve_error(&workflows(
            r#"{ count: "${event.product_id}", extra: "x" }"#,
        ));
        assert!(err.contains("unknown input `extra`"), "{err}");

        raw_with_workflows(&workflows(r#"{ count: "${event.product_id}" }"#))
            .resolve(&ModelSources::default())
            .expect("typed inputs resolve");
    }

    #[test]
    fn a_trigger_when_only_sees_event_variables() {
        let err = resolve_error(
            r#"
  - - name: Hot day
      slug: hot-day
      on: { type: cron, schedule: "0 7 * * *" }
      when: { type: var, var: lua.message, op: gt, value: 35 }
      modes: [home]
      run: []
"#,
        );

        assert!(err.contains("unknown variable `lua.message`"), "{err}");
    }

    #[test]
    fn a_var_guard_is_type_checked_against_the_full_scope() {
        let workflows = |when: &str| {
            format!(
                r#"
  - - name: Hot day
      slug: hot-day
      on: {{ type: woolworths }}
      modes: [home]
      run:
        - type: delay
          seconds: 1
          when: {when}
"#
            )
        };

        let err = resolve_error(&workflows(
            "{ type: var, var: event.name, op: gt, value: 3 }",
        ));
        assert!(err.contains("cannot compare `event.name`"), "{err}");

        raw_with_workflows(&workflows(
            "{ type: var, var: event.drop, op: gt, value: 35 }",
        ))
        .resolve(&ModelSources::default())
        .expect("guards see event variables");
    }

    #[test]
    fn a_hold_on_a_trigger_without_state_is_rejected() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Held cron
      slug: held-cron
      on: { type: cron, schedule: "0 13 * * TUE" }
      for: 10m
      modes: [home]
      run: []
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(err.contains("uses `for:`"), "{err}");
    }

    #[test]
    fn a_triggered_workflow_with_empty_modes_is_rejected() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Untagged cron
      slug: untagged-cron
      on: { type: cron, schedule: "0 13 * * TUE" }
      modes: []
      run: []
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(err.contains("empty `modes:`"), "{err}");
    }

    #[test]
    fn modes_on_a_reusable_workflow_are_rejected() {
        let err = serde_yaml::from_str::<RawSettings>(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Reusable
      slug: reusable
      modes: [home]
      run: []
"#,
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("`modes` needs an `on:` trigger"),
            "{err}"
        );
    }

    #[test]
    fn a_mode_trigger_without_to_or_from_is_rejected() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Any mode change
      slug: any-mode-change
      on: { type: mode }
      modes: [home]
      run: []
"#,
        )
        .unwrap();

        let err = raw.resolve(&ModelSources::default()).unwrap_err();
        assert!(err.contains("needs `to` or `from`"), "{err}");
    }

    #[test]
    fn eink_display_modes_resolve() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

eink_display:
  firmware_version: v0.1.0
  prepare_render_timeout: 15s
  defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s }
  views:
    home: { query: "view=home" }
  albums:
    family: { prefix: "eink-display/album/family/" }
    art: {}
devices:
-
  - id: epd
    state: enabled
    transport:
      type: eink_display_firmware
      address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          refresh: "0 * * * *"
          grace: 10m
          orientation: landscape
          partial:
            state: enabled
            max_area_pct: 25
            max_consecutive: 8
          mode:
            name: album
            album: family
"#,
        )
        .unwrap();

        let (settings, registry) = raw.resolve(&ModelSources::default()).unwrap();
        let display = registry.eink_display("abc123").expect("display resolved");

        assert_eq!(display.mode.name(), crate::settings::EinkMode::Album);
        assert!(display.partial.enabled);
        assert_eq!(display.partial.max_area_pct, 25);
        assert_eq!(display.partial.max_consecutive, 8);
        assert_eq!(display.target_dims(), (1600, 1200));
        assert_eq!(display.orientation_str(), "landscape");
        assert_eq!(display.mode.album(), Some("family"));

        let global = &settings.eink_display;
        assert_eq!(
            global.view("home").unwrap().query.as_deref(),
            Some("view=home")
        );
        assert_eq!(
            global.album("family").unwrap().prefix,
            "eink-display/album/family/"
        );
        assert_eq!(
            global.album("art").unwrap().prefix,
            "eink-display/album/art/"
        );
    }

    #[test]
    fn eink_display_reddit_mode_resolves() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

devices:
-
  - id: epd
    state: enabled
    transport:
      type: eink_display_firmware
      address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          refresh: "0 * * * *"
          grace: 10m
          orientation: portrait
          partial:
            state: disabled
            max_area_pct: 30
            max_consecutive: 5
          mode:
            name: reddit
            subreddit: EarthPorn
            timespan: week
            limit: 40
"#,
        )
        .unwrap();

        let (_, registry) = raw.resolve(&ModelSources::default()).unwrap();
        let display = registry.eink_display("abc123").expect("display resolved");

        assert_eq!(display.mode.name(), crate::settings::EinkMode::Reddit);

        let feed = display.mode.feed().expect("reddit feed resolved");
        assert_eq!(feed.subreddit, "EarthPorn");
        assert_eq!(feed.timespan, crate::settings::RedditTimespan::Week);
        assert_eq!(feed.limit, 40);
        assert_eq!(display.mode.album(), None);
        assert_eq!(display.mode.view(), None);
    }

    #[test]
    fn eink_display_defaults_resolve() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
notify: { targets: {}, disabled: [], android: { fcm_project_id: x } }
mqtt:
  url: x
  port: 1883
  username: x
  password: x
  keep_alive: 5s
  max_packet_size: 100000
  channel_capacity: 100
  reconnect: { min: 1s, max: 60s }
  models: lua/mqtt
  protocols:
    zigbee: { payload: json, entities: disabled, roles: [door], topics: { report: "zigbee2mqtt/{name}" } }
    esphome: { payload: esphome_domain, entities: enabled, roles: [environment], topics: { state: "{address}/{domain}/{object_id}/state" } }
    valetudo: { payload: json, entities: disabled, roles: [robot_vacuum], topics: { state: "valetudo/{address}/state" } }
http:
  listen_address: "[::]:8000"
  clients: { default: { timeout: 30s } }
database: { min_connections: 0, max_connections: 10, slow_statement_threshold: 6s }
graphql: { max_depth: 20, max_complexity: 5000, query_timeout: 10s }
actors: { restart: { backoff_base: 1s, backoff_max: 60s, healthy_after: 5m }, workers: { mqtt_ingest: 5, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: { control_switch: 3, door: 1, environment: 1, light: 1, media_player: 1, plant: 1, presence: 1, robot_vacuum: 2, smart_switch: 3 } } }
tracing: { sampling: { default: 1.0, spans: {} } }
home_assistant: { models: lua/home_assistant, websocket: { keep_alive: 30s, silence_timeout: 90s, reconnect_delay: 5s } }
auth: { api_key_cache: { capacity: 1024, ttl: 1h } }
watchdog: { state: disabled, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, enabled_cache: { capacity: 1024, ttl: 5m }, condition_timeout: 10s, timers: { catch_up_within: 10m } }
reconciler: { state: disabled, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
integrations: { s3: { bucket: b, region: r }, esphome: { state: disabled, models: lua/esphome_native_api, port: 6053, keep_alive: 20s, silence_timeout: 90s, reconnect_delay: 5s }, holidays: { url: x, regions: [Western Australia] }, woolworths: { state: disabled, refresh: 1h }, trmnl: { state: disabled, refresh: 3h, base_url: x }, willyweather: { state: enabled, api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }, fuelwatch: { state: disabled, postcode: 6000, refresh: 1h }, solar: { state: disabled, refresh: 1m }, transperth: { state: disabled, refresh_peak: 3m, refresh_off_peak: 15m, horizon: 2h, routes: [], cache: { routes: { capacity: 1, ttl: 1h }, timetables: { capacity: 1, ttl: 1h } } } }
adhoc: { recheck_interval: 15m, task_timeout: 5m, cron_jitter: 60s, batch_size: 10000, tasks: { refresh_public_holidays: { state: enabled, schedule: "0 4 1 * *", parameters: {} }, refresh_transperth_timetable: { state: enabled, schedule: "20 3 * * *", parameters: {} }, sample_light_state: { state: enabled, schedule: "*/5 * * * *", parameters: {} }, trim_derived_door_events: { state: enabled, schedule: "0 3 * * *", parameters: { retention: 8760h } }, trim_device_intent: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 336h } }, trim_device_metric: { state: enabled, schedule: "15 3 * * *", parameters: { retention: 4320h } }, trim_door_sensor: { state: enabled, schedule: "5 3 * * *", parameters: { retention: 8760h } }, trim_home_assistant_events: { state: enabled, schedule: "25 3 * * *", parameters: { retention: 2160h } }, trim_light_history: { state: enabled, schedule: "50 3 * * *", parameters: { retention: 2160h } }, trim_robot_vacuum_events: { state: enabled, schedule: "40 3 * * *", parameters: { retention: 4320h } }, trim_smart_switch: { state: enabled, schedule: "30 3 * * *", parameters: { retention: 4320h } }, trim_temperature_sensor: { state: enabled, schedule: "20 3 * * *", parameters: { retention: 4320h } }, trim_workflow_runs: { state: enabled, schedule: "45 3 * * *", parameters: { retention: 2160h } } } }
eink_display: { firmware_version: v0.1.0, prepare_render_timeout: 15s, defaults: { reddit_limit: 25, settle: 10s, fallback_refresh: 15m, min_refresh: 60s } }
vacation: { state: enabled, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

devices:
-
  - id: epd
    state: enabled
    transport:
      type: eink_display_firmware
      address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          refresh: "0 * * * *"
          grace: 10m
          partial:
            state: disabled
            max_area_pct: 30
            max_consecutive: 12
          mode:
            name: dashboard
            settle: 10s
            lead: 15m
"#,
        )
        .unwrap();

        let (settings, registry) = raw.resolve(&ModelSources::default()).unwrap();
        let display = registry.eink_display("abc123").expect("display resolved");

        assert_eq!(display.mode.name(), crate::settings::EinkMode::Dashboard);
        assert_eq!(display.target_dims(), (1200, 1600));
        assert!(display.mode.view().is_none());
        assert_eq!(
            settings.eink_display.default_album().prefix,
            "eink-display/album/"
        );
    }

    #[test]
    fn load_from_dir_errors_on_missing_dir() {
        let result = SettingsContainer::load_from_dir(Path::new("./does-not-exist"));
        assert!(result.is_err());
    }

    #[test]
    fn a_missing_include_is_an_error_not_a_panic() {
        let dir =
            std::env::temp_dir().join(format!("home-gateway-config-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("base.yaml"),
            "workflows: !include workflows/missing.yaml\n",
        )
        .unwrap();

        let result = SettingsContainer::load_from_dir(&dir);
        std::fs::remove_dir_all(&dir).unwrap();

        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to process includes"), "{err}");
        assert!(err.contains("missing.yaml"), "{err}");
    }

    #[test]
    fn every_config_file_is_shipped_in_the_config_map() {
        let kustomization = std::fs::read_to_string("./config/kustomization.yaml").unwrap();
        let listed: HashSet<&str> = kustomization
            .lines()
            .filter_map(|line| line.trim().strip_prefix("- "))
            .collect();

        let mut missing = Vec::new();

        for dir in [
            "",
            "sections/",
            "devices/",
            "workflows/",
            "lua/lib/",
            "lua/workflows/",
            "lua/mqtt/",
            "lua/home_assistant/",
        ] {
            for entry in std::fs::read_dir(Path::new("./config").join(dir)).unwrap() {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                let shipped = (name.ends_with(".yaml") && name != "kustomization.yaml")
                    || name.ends_with(".lua");
                let path = format!("{dir}{name}");

                if shipped && !listed.contains(path.as_str()) {
                    missing.push(path);
                }
            }
        }

        missing.sort();
        assert!(
            missing.is_empty(),
            "config files not listed in config/kustomization.yaml, the deployed config would miss them: {missing:?}"
        );
    }
}
