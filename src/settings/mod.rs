use config::builder::{ConfigBuilder, DefaultState};
use config::{Config, ConfigError, Environment, File, FileFormat};
use schemars::JsonSchema;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub mod adhoc;
pub mod alarm;
pub mod auth;
pub mod de;
pub mod devices;
pub mod integrations;
pub mod location;
pub mod notify;
pub mod reconciler;
pub mod sun;
pub mod vacation;
pub mod watchdog;
pub mod workflow;

pub use adhoc::AdhocSettings;
pub use alarm::AlarmSettings;
pub use auth::{ApiKeySettings, OAuthSettings};
pub use devices::device::{BatterySettings, DeviceWatchdog, RawDeviceWatchdog};
pub use devices::door::{ArmedDoorStates, DoorSettings};
pub use devices::eink::{
    Album, DashboardView, EinkDisplaySettings, EinkGlobalSettings, EinkMode, EinkModeConfig,
    Orientation, PartialRefresh, RawEinkDisplayBlock, RedditFeed, RedditTimespan, SleepWindow,
};
pub use devices::environment::{
    EnvironmentSensorSettings, EnvironmentSensorType, Metric, RawEnvironmentBlock,
};
pub use devices::light::RawLightBlock;
pub use devices::media_player::{MediaPlayerSettings, RawMediaPlayerBlock};
pub use devices::plant::{PlantSensorSettings, RawPlantBlock};
pub use devices::presence::{PresenceSensorType, PresenceSettings, RawPresenceBlock};
pub use devices::roborock::{RawRoborockBlock, RoborockField, RoborockSettings};
pub use devices::switch::{RawSmartSwitchBlock, SwitchRole};
pub use devices::trmnl::{RawTrmnlBlock, TrmnlDeviceSettings, TrmnlSettings};
pub use devices::valetudo::{RawValetudoBlock, ValetudoSettings};
pub use devices::zigbee_model::{
    RawZigbeeModelProfile, ZigbeeField, ZigbeeFieldType, ZigbeeModelProfile,
};
pub use integrations::fuelwatch::FuelWatchSettings;
pub use integrations::home_assistant::{EntitySettings, HomeAssistantSettings};
pub use integrations::jellyfin::JellyfinSettings;
pub use integrations::s3::S3Settings;
pub use integrations::solar::SolarSettings;
pub use integrations::transperth::{
    PeakWindow, RawTransperthSettings, TransperthRoute, TransperthSettings,
};
pub use integrations::willyweather::WillyWeatherSettings;
pub use integrations::woolworths::WoolworthsSettings;
pub use location::LocationSettings;
pub use notify::{
    NotifyAcknowledge, NotifyAction, NotifyActionKind, NotifyCategory, NotifySource, NotifyTargets,
    validate_acknowledge,
};
pub use reconciler::ReconcilerSettings;
pub use sun::SunSettings;
pub use vacation::VacationSettings;
pub use watchdog::WatchdogSettings;
pub use workflow::{
    ReusableWorkflow, TriggerMatcher, Workflow, WorkflowDefinition, WorkflowSettings,
};

use crate::auth::scope::ScopePattern;
use crate::device_registry::{DeviceRegistry, RawSensor};

pub type IEEEAddress = String;

/// Named device aliases (`alias -> ieee address`) declared under the top-level
/// `devices:` key. Referenced from workflow steps so addresses are written once.
pub type DeviceAliases = HashMap<String, IEEEAddress>;

/// Validate a workflow device reference. References must be device registry ids;
/// the id is kept as-is and resolved to an address at runtime. Unknown ids are
/// rejected at load time so typos fail loudly.
pub(crate) fn validate_device(reference: &str, devices: &DeviceAliases) -> Result<(), String> {
    if devices.contains_key(reference) {
        Ok(())
    } else {
        Err(format!("unknown device registry id: {reference}"))
    }
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
    pub http_listen_addr: SocketAddr,
    pub fcm_project_id: String,
    pub fcm_service_account_json: String,
    pub mqtt_url: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub workflows: HashMap<String, WorkflowDefinition>,
    pub workflow: WorkflowSettings,
    pub reconciler: ReconcilerSettings,
    pub s3: S3Settings,
    pub watchdog: WatchdogSettings,
    pub oauth: Option<OAuthSettings>,
    pub api_keys: Vec<ApiKeySettings>,
    pub location: LocationSettings,
    pub sun: SunSettings,
    pub alarm: AlarmSettings,
    pub woolworths: WoolworthsSettings,
    pub trmnl: TrmnlSettings,
    pub trmnl_api_key: Option<String>,
    pub home_assistant: HomeAssistantSettings,
    pub jellyfin: Option<JellyfinSettings>,
    pub solar: Option<SolarSettings>,
    pub transperth: Option<TransperthSettings>,
    pub willyweather: WillyWeatherSettings,
    pub fuelwatch: Option<FuelWatchSettings>,
    pub eink_display: EinkGlobalSettings,
    pub adhoc: AdhocSettings,
    pub vacation: VacationSettings,
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
    http_listen_addr: SocketAddr,
    #[serde(default)]
    fcm_project_id: String,
    #[serde(default)]
    fcm_service_account_json: String,
    mqtt_url: String,
    mqtt_port: u16,
    mqtt_username: String,
    mqtt_password: String,
    unifi_webhook_secret: String,
    android_app_webhook_secret: String,
    #[serde(default)]
    notify_targets: NotifyTargets,
    #[serde(default)]
    devices: Vec<RawSensor>,
    zigbee_models: HashMap<String, RawZigbeeModelProfile>,
    #[serde(default)]
    workflows: Vec<Vec<WorkflowDefinition>>,
    s3: S3Settings,
    watchdog: WatchdogSettings,
    workflow: WorkflowSettings,
    reconciler: ReconcilerSettings,
    #[serde(default)]
    oauth: Option<OAuthSettings>,
    #[serde(default)]
    api_keys: Vec<ApiKeySettings>,
    location: LocationSettings,
    sun: SunSettings,
    #[serde(default)]
    alarm: AlarmSettings,
    #[serde(default)]
    woolworths: WoolworthsSettings,
    #[serde(default)]
    trmnl: TrmnlSettings,
    #[serde(default)]
    trmnl_api_key: Option<String>,
    #[serde(default)]
    home_assistant: HomeAssistantSettings,
    #[serde(default)]
    jellyfin: Option<JellyfinSettings>,
    #[serde(default)]
    solar: Option<SolarSettings>,
    #[serde(default)]
    transperth: Option<RawTransperthSettings>,
    willyweather: WillyWeatherSettings,
    #[serde(default)]
    fuelwatch: Option<FuelWatchSettings>,
    #[serde(default)]
    eink_display: devices::eink::RawEinkGlobal,
    adhoc: AdhocSettings,
    vacation: VacationSettings,
}

impl RawSettings {
    fn resolve(self) -> Result<(Settings, DeviceRegistry), String> {
        let RawSettings {
            version,
            api_key,
            database_url,
            http_listen_addr,
            fcm_project_id,
            fcm_service_account_json,
            mqtt_url,
            mqtt_port,
            mqtt_username,
            mqtt_password,
            unifi_webhook_secret,
            android_app_webhook_secret,
            notify_targets,
            devices,
            zigbee_models,
            workflows,
            s3,
            watchdog,
            workflow,
            reconciler,
            oauth,
            api_keys,
            location,
            sun,
            alarm,
            woolworths,
            trmnl,
            trmnl_api_key,
            home_assistant,
            jellyfin,
            solar,
            transperth,
            willyweather,
            fuelwatch,
            eink_display,
            adhoc,
            vacation,
        } = self;

        if willyweather
            .api_key
            .as_deref()
            .map(str::trim)
            .is_none_or(str::is_empty)
        {
            return Err("willyweather.api_key is required (set WILLYWEATHER__API_KEY)".to_owned());
        }

        if willyweather.locations.is_empty() {
            return Err("willyweather.locations must declare at least one location".to_owned());
        }

        if !willyweather
            .locations
            .contains_key(&willyweather.default_location)
        {
            return Err(format!(
                "willyweather.default_location `{}` is not one of willyweather.locations",
                willyweather.default_location
            ));
        }

        if willyweather.refresh <= chrono::TimeDelta::zero() {
            return Err("willyweather.refresh must be positive".to_owned());
        }

        if willyweather.days < 1 {
            return Err("willyweather.days must be at least 1".to_owned());
        }

        let transperth = transperth.map(RawTransperthSettings::resolve).transpose()?;

        if let Some(transperth) = &transperth {
            if transperth
                .reference_data_api_key
                .as_deref()
                .map(str::trim)
                .is_none_or(str::is_empty)
            {
                return Err(
                    "transperth.reference_data_api_key is required (set TRANSPERTH__REFERENCE_DATA_API_KEY)"
                        .to_owned(),
                );
            }

            if transperth.routes.is_empty() {
                return Err("transperth.routes must not be empty".to_owned());
            }

            let mut seen_route_ids = HashSet::new();
            for route in &transperth.routes {
                if !seen_route_ids.insert(route.id.clone()) {
                    return Err(format!("duplicate transperth route id: {}", route.id));
                }

                if route.from.trim().is_empty() || route.to.trim().is_empty() {
                    return Err(format!(
                        "transperth route {} has an empty from/to",
                        route.id
                    ));
                }

                if route.limit == 0 {
                    return Err(format!("transperth route {} must have limit > 0", route.id));
                }
            }
        }

        let mut seen_key_names = HashSet::new();
        for key in &api_keys {
            if !seen_key_names.insert(key.name.clone()) {
                return Err(format!("duplicate api_keys name: {}", key.name));
            }

            validate_scopes(&format!("api key '{}'", key.name), &key.scopes)?;
        }

        if let Some(oauth) = &oauth {
            for (group, scopes) in &oauth.group_scopes {
                validate_scopes(&format!("oauth group '{group}'"), scopes)?;
            }
        }

        let registry = DeviceRegistry::build(devices, &notify_targets, zigbee_models)?;
        let aliases = registry.aliases();

        let mut resolved = HashMap::new();
        let mut scopes = HashMap::new();
        let mut slugs = HashSet::new();
        for mut workflow in workflows.into_iter().flatten() {
            workflow.resolve_devices(aliases)?;

            let body = workflow.body();
            body.validate_capabilities(&registry)?;

            if body.context.contains(&workflow::ContextSource::Fuelwatch) && fuelwatch.is_none() {
                return Err(format!(
                    "workflow '{}' uses `context: [fuelwatch]` but fuelwatch is not configured",
                    body.name
                ));
            }

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

            let scope = workflow::scope::scope_for(&workflow, &registry)?;
            workflow::scope::check_steps(body, &scope)?;

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

        for workflow in resolved.values().map(WorkflowDefinition::body) {
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
                http_listen_addr,
                fcm_project_id,
                fcm_service_account_json,
                mqtt_url,
                mqtt_port,
                mqtt_username,
                mqtt_password,
                unifi_webhook_secret,
                android_app_webhook_secret,
                workflows: resolved,
                s3,
                watchdog,
                workflow,
                reconciler,
                oauth,
                api_keys,
                location,
                sun,
                alarm,
                woolworths,
                trmnl,
                trmnl_api_key,
                home_assistant,
                jellyfin,
                solar,
                transperth,
                willyweather,
                fuelwatch,
                eink_display: eink_display.resolve(),
                adhoc,
                vacation,
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
    fn build(config: Config) -> Result<(Settings, DeviceRegistry), ConfigError> {
        let raw: RawSettings = config.try_deserialize()?;
        raw.resolve().map_err(ConfigError::Message)
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

        Self::build(config)
    }

    pub fn new() -> Result<(Self, DeviceRegistry), ConfigError> {
        let override_dir =
            std::env::var("CONFIG_DIR").unwrap_or_else(|_| "/etc/home-gateway/config".to_string());
        let baked_dir = PathBuf::from("./config");

        let (source, (settings, registry)) = match Self::load_from_dir(Path::new(&override_dir)) {
            Ok(loaded) => ("override", loaded),
            Err(e) => {
                tracing::warn!(
                    config_dir = %override_dir,
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
    use crate::device_registry::{Capability, RawSensor};

    fn lamp_registry() -> DeviceRegistry {
        let devices: Vec<RawSensor> = serde_yaml::from_str(
            r#"
- id: living-room-table-lamp
  transport: zigbee
  model: ts011f_plug
  address: "0xa4c1389fe5cea26e"
  roles:
    - type: smart_switch
      config: { name: Living Room Table Lamp, as: light }
"#,
        )
        .unwrap();

        DeviceRegistry::build(devices, &NotifyTargets::default(), test_models()).unwrap()
    }

    fn test_models() -> HashMap<String, RawZigbeeModelProfile> {
        serde_yaml::from_str(
            r#"
ts011f_plug:
  smart_switch: [state, voltage, power, current, energy]
"#,
        )
        .unwrap()
    }

    fn build_devices(yaml: &str) -> Result<DeviceRegistry, String> {
        let devices: Vec<RawSensor> = serde_yaml::from_str(yaml).unwrap();

        DeviceRegistry::build(devices, &NotifyTargets::default(), test_models())
    }

    #[test]
    fn a_zigbee_device_without_a_model_is_rejected() {
        let err = build_devices(
            r#"
- id: mystery
  transport: zigbee
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
  transport: esphome
  address: living-room-tv
  roles:
    - type: media_player
      config:
        name: TV
"#,
        )
        .unwrap_err();

        assert!(err.contains("`home_assistant` transport"), "{err}");
    }

    #[test]
    fn a_media_player_address_that_is_not_an_entity_id_is_rejected() {
        let err = build_devices(
            r#"
- id: tv
  transport: home_assistant
  address: living_room_tv
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
  transport: zigbee
  model: not_a_real_model
  address: "0xdeadbeef"
  roles:
    - type: control_switch
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("unknown zigbee model `not_a_real_model`"),
            "{err}"
        );
        assert!(err.contains("ts011f_plug"), "{err}");
    }

    #[test]
    fn a_role_the_model_does_not_map_is_rejected() {
        let err = build_devices(
            r#"
- id: mystery
  transport: zigbee
  model: ts011f_plug
  address: "0xdeadbeef"
  roles:
    - type: door
      config: { name: Mystery Door, id: mystery, state: armed, timeout: 3m }
"#,
        )
        .unwrap_err();

        assert!(err.contains("has no `door` mapping"), "{err}");
    }

    #[test]
    fn a_model_on_a_non_zigbee_transport_is_rejected() {
        let err = build_devices(
            r#"
- id: living-room-mtr-1
  transport: esphome
  model: ts011f_plug
  address: apollo-mtr-1-livingroom
  roles:
    - type: presence
      config: { name: Living Room, motion_entity: [ld2450_presence] }
"#,
        )
        .unwrap_err();

        assert!(
            err.contains("only valid with the `zigbee` transport"),
            "{err}"
        );
    }

    #[test]
    fn esphome_light_without_entity_is_rejected() {
        let devices: Vec<RawSensor> = serde_yaml::from_str(
            r#"
- id: living-room-mtr-1
  transport: esphome
  address: apollo-mtr-1-livingroom
  roles:
    - type: light
      config: { name: Living Room MTR-1 RGB }
"#,
        )
        .unwrap();

        let err =
            DeviceRegistry::build(devices, &NotifyTargets::default(), test_models()).unwrap_err();
        assert!(err.contains("has no `entity` object_id"), "{err}");
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
  transport: zigbee
  model: ts011f_plug
  address: "0xa4c1389fe5cea26e"
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
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
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

        let (_settings, registry) = SettingsContainer::build(config).unwrap();

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
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
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

        let (settings, registry) = SettingsContainer::build(config).unwrap();

        let switch_workflow = settings
            .workflows
            .values()
            .filter_map(WorkflowDefinition::triggered)
            .find(|w| {
                matches!(&w.on, TriggerMatcher::Switch { ieee_addr, action }
                if ieee_addr == "small-switch" && action == "single")
            })
            .expect("expected a switch workflow for the small switch");
        let referenced = switch_workflow
            .run
            .iter()
            .map(|step| match step {
                workflow::Step::RunWorkflow { workflow, .. } => workflow.as_str(),
                other => panic!("expected a run_workflow step, got {}", other.kind()),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            referenced,
            ["living-room-lamps-off", "living-room-lamps-on"]
        );
        for name in referenced {
            assert!(
                settings.workflows.contains_key(name),
                "small switch references unknown workflow {name}"
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
            .roborock(roborock_address)
            .expect("roborock device resolves");
        assert_eq!(registry.room(roborock_address), Some("dining-room"));
        assert_eq!(roborock.battery_entity, "sensor.robot_battery");
        assert_eq!(roborock.start_service, "vacuum.start");
        assert_eq!(roborock.stop_service, "vacuum.stop");
        assert_eq!(roborock.dock_service, "vacuum.return_to_base");

        let valetudo_address = registry.address_or_self("valetudo");
        let valetudo = registry
            .valetudo(valetudo_address)
            .expect("valetudo device resolves");
        assert_eq!(registry.room(valetudo_address), Some("spare-room"));
        assert_eq!(valetudo.command_topic, "valetudo/rockrobo/command");
        assert_eq!(valetudo.dock_payload, "return_to_base");

        let tv_address = registry.address_or_self("living-room-tv");
        let tv = registry
            .media_player(tv_address)
            .expect("living room tv resolves");
        assert_eq!(tv_address, "media_player.living_room_tv");
        assert_eq!(tv.id, "living-room-tv");
        assert_eq!(tv.name, "Living Room TV");
        assert_eq!(tv.entity_id, "media_player.living_room_tv");
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

        assert_eq!(
            registry.roborock_entity("sensor.robot_battery"),
            Some(("roborock", RoborockField::Battery))
        );
        assert_eq!(
            registry.roborock_entity("sensor.robot_status"),
            Some(("roborock", RoborockField::Status))
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
        assert_eq!(
            registry.esphome_light(mtr).map(String::as_str),
            Some("rgb_light")
        );
        assert_eq!(
            registry.esphome_target("apollo-mtr-1-livingroom/light/rgb_light/state"),
            Some(&crate::integrations::esphome::EsphomeTarget::Light {
                node: mtr.to_owned(),
                object_id: "rgb_light".to_owned(),
            })
        );
        // it has no colour temperature, so those workflow steps are rejected
        assert!(!registry.capabilities(mtr).contains(&Capability::ColourTemp));

        // a zigbee presence sensor is keyed by its address in the registry
        assert!(registry.presence("0x54ef441000dbc81c").is_some());

        // esphome presence sensor keyed by node name carries its motion entities
        assert_eq!(
            registry
                .presence("apollo-mtr-1-livingroom")
                .unwrap()
                .motion_entities,
            vec![
                "ld2450_presence".to_owned(),
                "ld2450_moving_target".to_owned(),
                "ld2450_still_target".to_owned(),
            ]
        );
        assert!(registry.environment("apollo-mtr-1-livingroom").is_some());

        // a single device definition can carry multiple roles: the hallway plant
        // is registered as both an environment and a plant sensor at one address
        assert_eq!(
            registry
                .environment("apollo-plt-1-hallway")
                .unwrap()
                .sensor_type,
            EnvironmentSensorType::Esphome
        );
        assert!(registry.plant("apollo-plt-1-hallway").is_some());

        // an esphome environment sensor maps each configured object_id to a metric
        assert_eq!(
            registry
                .environment("apollo-plt-1-hallway")
                .unwrap()
                .entities
                .get("air_temperature"),
            Some(&Metric::Temperature)
        );

        // the esphome motion topic is registered for routing
        assert!(
            registry
                .esphome_target("apollo-mtr-1-livingroom/binary_sensor/ld2450_presence/state")
                .is_some()
        );

        // every zigbee device resolves to a model profile covering its roles
        let front = registry
            .zigbee_device(registry.address_or_self("front-door"))
            .expect("front-door is a zigbee device");
        assert_eq!(front.profile.slug, "aqara_mccgq12lm");
        let front_address = registry.address_or_self("front-door");
        assert!(
            registry.door(front_address).is_some() && registry.battery(front_address).is_some()
        );
        assert_eq!(front.profile.battery.as_deref(), Some("battery"));
        assert_eq!(
            front.profile.door.as_ref().map(|d| d.contact.as_str()),
            Some("contact")
        );

        let outdoor = registry
            .zigbee_device(registry.address_or_self("env-outdoor"))
            .expect("env-outdoor is a zigbee device");
        assert!(
            outdoor
                .profile
                .environment
                .as_ref()
                .expect("environment mapping")
                .iter()
                .any(|(metric, key)| *metric == Metric::Temperature && key == "temperature")
        );

        // the free-form metrics escape hatch carries its declared type
        let presence = registry
            .zigbee_device("0x54ef441000dbc81c")
            .expect("closet presence is a zigbee device");
        assert!(presence.profile.metrics.iter().any(|(name, field)| {
            name == "movement" && field.field_type == ZigbeeFieldType::String
        }));

        // esphome devices are not in the zigbee table
        assert!(registry.zigbee_device("apollo-mtr-1-livingroom").is_none());
    }

    #[test]
    fn api_keys_parse_and_validate_scopes() {
        let secrets = r#"
api_key: x
database_url: x
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
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

        let (settings, _registry) = SettingsContainer::build(config).unwrap();
        assert!(
            settings
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
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
api_keys:
  - name: bad-key
    scopes: ["bogus:read"]
"#,
        )
        .unwrap();

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("unknown resource `bogus`"), "{err}");
    }

    #[test]
    fn oauth_group_scopes_are_validated() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
oauth:
  issuer: i
  jwks_url: j
  userinfo_url: u
  audience: a
  group_scopes:
    admins@idm: ["graphql:solar:read"]
"#,
        )
        .unwrap();

        let err = raw.resolve().unwrap_err();
        assert!(
            err.contains("oauth group 'admins@idm' has invalid scope"),
            "{err}"
        );
    }

    #[test]
    fn run_workflow_rejects_an_unknown_target() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

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

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("does-not-exist"), "{err}");
    }

    #[test]
    fn run_workflow_accepts_a_known_target() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

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

        raw.resolve().expect("a known target resolves");
    }

    fn raw_with_workflows(workflows: &str) -> RawSettings {
        let base = r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }
"#;

        serde_yaml::from_str(&format!("{base}\nworkflows:\n{workflows}")).unwrap()
    }

    fn resolve_error(workflows: &str) -> String {
        raw_with_workflows(workflows).resolve().unwrap_err()
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
            .resolve()
            .expect("typed inputs resolve");
    }

    #[test]
    fn a_trigger_when_only_sees_event_variables() {
        let err = resolve_error(
            r#"
  - - name: Hot day
      slug: hot-day
      on: { type: cron, schedule: "0 7 * * *" }
      when: { type: var, var: willyweather.today.max, op: gt, value: 35 }
      context: [willyweather]
      modes: [home]
      run: []
"#,
        );

        assert!(
            err.contains("unknown variable `willyweather.today.max`"),
            "{err}"
        );
    }

    #[test]
    fn a_var_guard_is_type_checked_against_the_full_scope() {
        let workflows = |when: &str| {
            format!(
                r#"
  - - name: Hot day
      slug: hot-day
      on: {{ type: woolworths }}
      context: [willyweather]
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
            "{ type: var, var: willyweather.today.max, op: gt, value: 35 }",
        ))
        .resolve()
        .expect("guards see context variables");
    }

    #[test]
    fn a_hold_on_a_trigger_without_state_is_rejected() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

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

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("uses `for:`"), "{err}");
    }

    #[test]
    fn a_triggered_workflow_with_empty_modes_is_rejected() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Untagged cron
      slug: untagged-cron
      on: { type: cron, schedule: "0 13 * * TUE" }
      modes: []
      run: []
"#,
        )
        .unwrap();

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("empty `modes:`"), "{err}");
    }

    #[test]
    fn modes_on_a_reusable_workflow_are_rejected() {
        let err = serde_yaml::from_str::<RawSettings>(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

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
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

workflows:
  - - name: Any mode change
      slug: any-mode-change
      on: { type: mode }
      modes: [home]
      run: []
"#,
        )
        .unwrap();

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("needs `to` or `from`"), "{err}");
    }

    #[test]
    fn eink_display_modes_resolve() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

eink_display:
  views:
    home: { query: "view=home" }
  albums:
    family: { prefix: "eink-display/album/family/" }
    art: {}
devices:
  - id: epd
    transport: eink_display_firmware
    address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          firmware_version: v0.1.0
          refresh: "0 * * * *"
          grace: 10m
          orientation: landscape
          partial:
            enabled: true
            max_area_pct: 25
            max_consecutive: 8
          mode:
            name: album
            album: family
"#,
        )
        .unwrap();

        let (settings, registry) = raw.resolve().unwrap();
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
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

devices:
  - id: epd
    transport: eink_display_firmware
    address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          firmware_version: v0.1.0
          refresh: "0 * * * *"
          grace: 10m
          orientation: portrait
          partial:
            enabled: false
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

        let (_, registry) = raw.resolve().unwrap();
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
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
mqtt_port: 1883
http_listen_addr: "[::]:8000"
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12, timers: { catch_up_within: 10m } }
reconciler: { enabled: false, workers: 2, interval: 5s, grace: 3s, backoff: 10s, confirm_timeout: 5s, max_attempts: 3, batch_size: 64 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
willyweather: { api_key: x, refresh: 1h, days: 7, default_location: perth, locations: { perth: "14576" } }
adhoc: { recheck_interval: 15m }
vacation: { enabled: true, modes: [vacation], window: 672h, jitter: 12m, min_observations: 8, seed: 1 }

devices:
  - id: epd
    transport: eink_display_firmware
    address: "abc123"
    roles:
      - type: eink_display_firmware
        config:
          name: Test Display
          firmware_version: v0.1.0
          refresh: "0 * * * *"
          grace: 10m
          partial:
            enabled: false
            max_area_pct: 30
            max_consecutive: 12
          mode:
            name: dashboard
            settle: 10s
            lead: 15m
"#,
        )
        .unwrap();

        let (settings, registry) = raw.resolve().unwrap();
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

        for dir in ["", "workflows/"] {
            for entry in std::fs::read_dir(Path::new("./config").join(dir)).unwrap() {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                let is_yaml = name.ends_with(".yaml") && name != "kustomization.yaml";
                let path = format!("{dir}{name}");

                if is_yaml && !listed.contains(path.as_str()) {
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
