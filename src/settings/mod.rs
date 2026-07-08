use config::builder::{ConfigBuilder, DefaultState};
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

pub mod appliance;
pub mod door;
pub mod environment;
pub mod notify;
pub mod plant;
pub mod presence;
pub mod trigger;
pub mod workflow;

pub use appliance::ApplianceSettings;
pub use door::{ArmedDoorStates, DoorSettings};
pub use environment::{EnvironmentSensorSettings, EnvironmentSensorType};
pub use notify::{NotifySource, NotifyTargets};
pub use plant::PlantSensorSettings;
pub use presence::{PresenceSensorType, PresenceSettings};
pub use trigger::TriggerMatcher;
pub use workflow::Workflow;

use crate::device_registry::{DeviceRegistry, RawSensor};
use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;

pub type IEEEAddress = String;

#[derive(Debug, Clone, Deserialize)]
pub struct WatchdogSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(with = "time_delta_from_str")]
    pub timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    pub check_interval: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    pub realert_after: TimeDelta,
}

/// S3 / object-storage config. Credentials are taken from the standard AWS
/// environment, never from this file. `endpoint` is only set for
/// S3-compatible stores (MinIO/R2/…); omit it for plain AWS S3.
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthSettings {
    pub issuer: String,
    pub jwks_url: String,
    pub audience: String,
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    /// group SPN -> granted scope strings (`domain:resource:action`).
    pub group_scopes: HashMap<String, Vec<String>>,
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
}

/// On-disk shape of the config. Deserialized first, then [`RawSettings::resolve`]
/// resolves device aliases / notify targets and unifies sensor keying so the rest
/// of the app only ever sees the fully-resolved [`Settings`].
#[derive(Debug, Deserialize, Clone)]
struct RawSettings {
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
}

impl RawSettings {
    fn resolve(self) -> Result<(Settings, DeviceRegistry), String> {
        let RawSettings {
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
        } = self;

        let registry = DeviceRegistry::build(devices, &notify_targets)?;
        let aliases = registry.aliases();

        let mut resolved = HashMap::new();
        for mut workflow in workflows.into_iter().flatten() {
            workflow.resolve_devices(aliases)?;
            let name = workflow.name.clone();
            if resolved.insert(name.clone(), workflow).is_some() {
                return Err(format!("duplicate workflow name: {name}"));
            }
        }

        Ok((
            Settings {
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

    pub fn new() -> Result<(Self, DeviceRegistry), ConfigError> {
        // TODO: fetch config from catalog, falling back to file if not found or other error
        let config = Self::config_sources(&PathBuf::from("./config"))?
            .add_source(Environment::default().separator("__"))
            .build()?;

        let (settings, registry) = Self::build(config)?;

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
        let workflow::Step::Light { ieee_addr, .. } = &switch_workflow.run[0] else {
            panic!("expected a light step");
        };
        assert_eq!(ieee_addr, "floor-lamp-living-room");

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

        // a zigbee presence sensor is keyed by its address in the registry
        assert!(registry.presence("0x54ef441000dbc81c").is_some());

        // esphome presence sensor keyed by node name carries its motion entity
        assert_eq!(
            registry
                .presence("apollo-mtr-1-livingroom")
                .unwrap()
                .motion_entity
                .as_deref(),
            Some("ld2450_moving_target")
        );
        // apollo-mtr is presence-only, not an environment sensor
        assert!(registry.environment("apollo-mtr-1-livingroom").is_none());

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

        // an esphome environment sensor with no explicit `entities:` falls back to
        // the default temperature object_ids, which drive its subscriptions
        assert_eq!(
            registry
                .environment("apollo-plt-1-hallway")
                .unwrap()
                .entities,
            crate::esphome::TEMPERATURE_SENSOR_OBJECT_IDS
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );

        // the esphome motion topic is registered for routing
        assert!(
            registry
                .esphome_target("apollo-mtr-1-livingroom/binary_sensor/ld2450_moving_target/state")
                .is_some()
        );
    }
}
