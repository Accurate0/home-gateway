use config::builder::{ConfigBuilder, DefaultState};
use config::{Config, ConfigError, Environment, File, FileFormat};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub mod door;
pub mod environment;
pub mod home_assistant;
pub mod location;
pub mod notify;
pub mod plant;
pub mod presence;
pub mod roborock;
pub mod template;
pub mod trigger;
pub mod workflow;

pub use door::{ArmedDoorStates, DoorSettings};
pub use environment::{EnvironmentSensorSettings, EnvironmentSensorType, Metric};
pub use home_assistant::{EntitySettings, HomeAssistantSettings};
pub use location::LocationSettings;
pub use notify::{NotifySource, NotifyTargets};
pub use plant::PlantSensorSettings;
pub use presence::{PresenceSensorType, PresenceSettings};
pub use roborock::{RawRoborockBlock, RoborockSettings};
pub use template::TemplateString;
pub use trigger::TriggerMatcher;
pub use workflow::Workflow;

use crate::auth::scope::ScopePattern;
use crate::device_registry::{DeviceRegistry, RawSensor};
use crate::timedelta_format::time_delta_from_str;
use chrono::{DateTime, TimeDelta, Utc};

pub type IEEEAddress = String;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WatchdogSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub check_interval: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub realert_after: TimeDelta,
}

pub(crate) fn default_alarm_offset() -> TimeDelta {
    TimeDelta::minutes(5)
}

pub(crate) fn default_alarm_workflow() -> String {
    "alarm-wakeup".to_owned()
}

pub(crate) fn default_alarm_poll_interval() -> TimeDelta {
    TimeDelta::seconds(60)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AlarmSettings {
    #[serde(default = "default_alarm_offset", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub offset: TimeDelta,
    #[serde(default = "default_alarm_workflow")]
    pub workflow: String,
    #[serde(default = "default_alarm_poll_interval", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub poll_interval: TimeDelta,
}

impl Default for AlarmSettings {
    fn default() -> Self {
        Self {
            offset: default_alarm_offset(),
            workflow: default_alarm_workflow(),
            poll_interval: default_alarm_poll_interval(),
        }
    }
}

pub(crate) fn default_woolworths_refresh() -> TimeDelta {
    TimeDelta::hours(1)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WoolworthsSettings {
    #[serde(default = "default_woolworths_refresh", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
}

impl Default for WoolworthsSettings {
    fn default() -> Self {
        Self {
            refresh: default_woolworths_refresh(),
        }
    }
}

pub(crate) fn default_trmnl_refresh() -> TimeDelta {
    TimeDelta::hours(3)
}

pub(crate) fn default_trmnl_base_url() -> String {
    "https://trmnl.com".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TrmnlSettings {
    #[serde(default = "default_trmnl_refresh", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    #[serde(default = "default_trmnl_base_url")]
    pub base_url: String,
}

impl Default for TrmnlSettings {
    fn default() -> Self {
        Self {
            refresh: default_trmnl_refresh(),
            base_url: default_trmnl_base_url(),
        }
    }
}

/// S3 / object-storage config. Credentials are taken from the standard AWS
/// environment, never from this file. `endpoint` is only set for
/// S3-compatible stores (MinIO/R2/…); omit it for plain AWS S3.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct S3Settings {
    pub bucket: String,
    pub region: String,
    #[serde(default)]
    pub endpoint: Option<String>,
}

/// Default for the OAuth `groups` claim name.
pub(crate) fn default_groups_claim() -> String {
    "groups".to_owned()
}

/// OAuth (OIDC) settings. Access tokens are JWTs validated locally against the
/// provider's JWKS — no client secret is needed. A caller's scopes are derived
/// from their group memberships (the `groups_claim`) via `group_scopes`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OAuthSettings {
    pub issuer: String,
    pub jwks_url: String,
    pub userinfo_url: String,
    pub audience: String,
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    /// group SPN -> granted scope strings (`domain:resource:action`).
    pub group_scopes: HashMap<String, Vec<String>>,
}

/// Declarative API key: config is the source of truth for a key's name + scope.
/// Secret material is never here — the admin API mints/regenerates the token and
/// startup reconciles these scopes onto the matching DB row by `name`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ApiKeySettings {
    pub name: String,
    pub scopes: Vec<String>,
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
}

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
    pub fcm_project_id: String,
    pub fcm_service_account_json: String,
    pub mqtt_url: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub workflows: HashMap<String, Workflow>,
    pub s3: S3Settings,
    pub watchdog: WatchdogSettings,
    pub oauth: Option<OAuthSettings>,
    pub api_keys: Vec<ApiKeySettings>,
    pub location: LocationSettings,
    pub alarm: AlarmSettings,
    pub woolworths: WoolworthsSettings,
    pub trmnl: TrmnlSettings,
    pub trmnl_api_key: Option<String>,
    pub home_assistant: HomeAssistantSettings,
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
    #[serde(default)]
    fcm_project_id: String,
    #[serde(default)]
    fcm_service_account_json: String,
    mqtt_url: String,
    mqtt_username: String,
    mqtt_password: String,
    unifi_webhook_secret: String,
    android_app_webhook_secret: String,
    #[serde(default)]
    notify_targets: NotifyTargets,
    #[serde(default)]
    devices: Vec<RawSensor>,
    #[serde(default)]
    workflows: Vec<Vec<Workflow>>,
    s3: S3Settings,
    watchdog: WatchdogSettings,
    #[serde(default)]
    oauth: Option<OAuthSettings>,
    #[serde(default)]
    api_keys: Vec<ApiKeySettings>,
    location: LocationSettings,
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
}

impl RawSettings {
    fn resolve(self) -> Result<(Settings, DeviceRegistry), String> {
        let RawSettings {
            version,
            api_key,
            database_url,
            fcm_project_id,
            fcm_service_account_json,
            mqtt_url,
            mqtt_username,
            mqtt_password,
            unifi_webhook_secret,
            android_app_webhook_secret,
            notify_targets,
            devices,
            workflows,
            s3,
            watchdog,
            oauth,
            api_keys,
            location,
            alarm,
            woolworths,
            trmnl,
            trmnl_api_key,
            home_assistant,
        } = self;

        let mut seen_key_names = HashSet::new();
        for key in &api_keys {
            if !seen_key_names.insert(key.name.clone()) {
                return Err(format!("duplicate api_keys name: {}", key.name));
            }
            for scope in &key.scopes {
                if ScopePattern::parse(scope).is_none() {
                    return Err(format!("api key '{}' has invalid scope: {scope}", key.name));
                }
            }
        }

        let registry = DeviceRegistry::build(devices, &notify_targets)?;
        let aliases = registry.aliases();

        let mut resolved = HashMap::new();
        let mut slugs = HashSet::new();
        for mut workflow in workflows.into_iter().flatten() {
            workflow.resolve_devices(aliases)?;
            workflow.validate_capabilities(&registry)?;
            if let Some(trigger) = workflow.on() {
                let available = trigger.available_vars();
                for var in workflow.template_placeholders() {
                    if !available.contains(&var) {
                        tracing::warn!(
                            "workflow '{}' references unknown template var ${{{var}}}; \
                             its trigger provides: [{}]",
                            workflow.name,
                            available.join(", ")
                        );
                    }
                }
            }
            if workflow.slug.trim().is_empty() {
                return Err(format!("workflow '{}' has an empty slug", workflow.name));
            }
            if !slugs.insert(workflow.slug.clone()) {
                return Err(format!("duplicate workflow slug: {}", workflow.slug));
            }
            let name = workflow.name.clone();
            if resolved.insert(name.clone(), workflow).is_some() {
                return Err(format!("duplicate workflow name: {name}"));
            }
        }

        Ok((
            Settings {
                version,
                api_key,
                database_url,
                fcm_project_id,
                fcm_service_account_json,
                mqtt_url,
                mqtt_username,
                mqtt_password,
                unifi_webhook_secret,
                android_app_webhook_secret,
                workflows: resolved,
                s3,
                watchdog,
                oauth,
                api_keys,
                location,
                alarm,
                woolworths,
                trmnl,
                trmnl_api_key,
                home_assistant,
            },
            registry,
        ))
    }
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
        let merged = yaml_include::Transformer::new(base.clone(), true)
            .map_err(|e| {
                ConfigError::Message(format!(
                    "failed to process includes in {}: {e}",
                    base.display()
                ))
            })?
            .to_string();

        Ok(Config::builder().add_source(File::from_str(&merged, FileFormat::Yaml)))
    }

    fn load_from_dir(dir: &Path) -> Result<(Settings, DeviceRegistry), ConfigError> {
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
  address: "0xa4c1389fe5cea26e"
  kinds:
    - kind: smart_switch
      config: { name: Living Room Table Lamp, as: light }
"#,
        )
        .unwrap();

        DeviceRegistry::build(devices, &NotifyTargets::default()).unwrap()
    }

    #[test]
    fn esphome_light_without_entity_is_rejected() {
        let devices: Vec<RawSensor> = serde_yaml::from_str(
            r#"
- id: living-room-mtr-1
  transport: esphome
  address: apollo-mtr-1-livingroom
  kinds:
    - kind: light
      config: { name: Living Room MTR-1 RGB }
"#,
        )
        .unwrap();

        let err = DeviceRegistry::build(devices, &NotifyTargets::default()).unwrap_err();
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

    #[test]
    fn config_yaml_parses_and_resolves() {
        // secrets normally come from the environment; supply dummies for the test
        let secrets = r#"
api_key: x
database_url: x
mqtt_url: x
mqtt_username: x
mqtt_password: x
unifi_webhook_secret: x
android_app_webhook_secret: x
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
            .find(|w| {
                matches!(w.on(), Some(trigger::TriggerMatcher::Switch { ieee_addr, action })
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
            settings.workflows.values().any(|w| w.name == "Bins"
                && matches!(w.on(), Some(trigger::TriggerMatcher::Cron { .. })))
        );

        let roborock_address = registry.address_or_self("roborock");
        let roborock = registry
            .roborock(roborock_address)
            .expect("roborock device resolves");
        assert_eq!(registry.room(roborock_address), Some("dining-room"));
        assert_eq!(roborock.battery_entity, "sensor.robot_battery");
        assert_eq!(roborock.stop_service, "vacuum.stop");
        assert_eq!(roborock.dock_service, "vacuum.return_to_base");

        let mut seen = HashSet::new();
        for wf in settings.workflows.values() {
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
            Some(&crate::esphome::EsphomeTarget::Light {
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

        // a single device definition can carry multiple kinds: the hallway plant
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
    }

    #[test]
    fn api_keys_parse_and_validate_scopes() {
        let secrets = r#"
api_key: x
database_url: x
mqtt_url: x
mqtt_username: x
mqtt_password: x
unifi_webhook_secret: x
android_app_webhook_secret: x
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
                .any(|k| k.name == "eink-display-living-room" && k.scopes == ["rest:epd:read"]),
            "expected the eink config key to parse"
        );
    }

    #[test]
    fn api_keys_reject_invalid_scope() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
mqtt_url: x
mqtt_username: x
mqtt_password: x
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
location: { latitude: 0.0, longitude: 0.0 }
api_keys:
  - name: bad-key
    scopes: ["graphql:bogus:read"]
"#,
        )
        .unwrap();

        let err = raw.resolve().unwrap_err();
        assert!(err.contains("invalid scope"), "{err}");
    }

    #[test]
    fn load_from_dir_errors_on_missing_dir() {
        let result = SettingsContainer::load_from_dir(Path::new("./does-not-exist"));
        assert!(result.is_err());
    }
}
