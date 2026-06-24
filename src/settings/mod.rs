use arc_swap::{ArcSwap, Guard};
use config::builder::{ConfigBuilder, DefaultState};
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

pub mod appliance;
pub mod door;
pub mod environment;
pub mod graphql;
pub mod maccas;
pub mod notify;
pub mod plant;
pub mod presence;
pub mod trigger;
pub mod workflow;

pub use appliance::ApplianceSettings;
pub use door::{ArmedDoorStates, DoorSettings};
pub use environment::{EnvironmentSensorSettings, EnvironmentSensorType};
pub use graphql::GraphqlSettings;
pub use maccas::MaccasSettings;
pub use notify::{NotifySource, NotifyTargets};
pub use plant::PlantSensorSettings;
pub use presence::{PresenceSensorType, PresenceSettings};
pub use trigger::{Trigger, TriggerMatcher};
pub use workflow::WorkflowSettings;

use appliance::RawApplianceSettings;
use door::RawDoorSettings;
use environment::RawEnvironmentSensor;
use maccas::RawMaccasSettings;
use plant::RawPlantSensor;
use presence::RawPresenceSettings;

pub type IEEEAddress = String;

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

/// Named device aliases (`alias -> ieee address`) declared under the top-level
/// `devices:` key. Referenced from workflow steps so addresses are written once.
pub type DeviceAliases = HashMap<String, IEEEAddress>;

/// Resolve a workflow device reference: a named alias, or a raw `0x…` address.
/// Anything else is rejected at load time so typos fail loudly.
pub(crate) fn resolve_device(
    reference: &str,
    devices: &DeviceAliases,
) -> Result<IEEEAddress, String> {
    if let Some(addr) = devices.get(reference) {
        Ok(addr.clone())
    } else if reference.starts_with("0x") {
        Ok(reference.to_string())
    } else {
        Err(format!("unknown device alias: {reference}"))
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
    pub doors: HashMap<IEEEAddress, DoorSettings>,
    pub environment_sensors: HashMap<String, EnvironmentSensorSettings>,
    pub appliances: HashMap<IEEEAddress, ApplianceSettings>,
    pub maccas: MaccasSettings,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub presence_sensors: HashMap<String, PresenceSettings>,
    pub plant_sensors: HashMap<String, PlantSensorSettings>,
    pub triggers: Vec<Trigger>,
    /// Named, reusable workflows referenced by `run_workflow` steps.
    pub workflows: HashMap<String, WorkflowSettings>,
    pub graphql: GraphqlSettings,
    pub s3: S3Settings,
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
    devices: DeviceAliases,
    #[serde(default)]
    notify_targets: NotifyTargets,
    #[serde(default)]
    doors: HashMap<IEEEAddress, RawDoorSettings>,
    #[serde(default)]
    environment_sensors: Vec<RawEnvironmentSensor>,
    #[serde(default)]
    appliances: HashMap<IEEEAddress, RawApplianceSettings>,
    maccas: RawMaccasSettings,
    #[serde(default)]
    presence_sensors: Vec<RawPresenceSettings>,
    #[serde(default)]
    plant_sensors: Vec<RawPlantSensor>,
    // triggers are split into per-device files, each a list, so the included
    // shape is a list-of-lists; flattened during resolve
    #[serde(default)]
    triggers: Vec<Vec<Trigger>>,
    #[serde(default)]
    workflows: HashMap<String, WorkflowSettings>,
    #[serde(default)]
    graphql: GraphqlSettings,
    s3: S3Settings,
}

impl RawSettings {
    fn resolve(self) -> Result<Settings, String> {
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
            devices,
            notify_targets,
            doors,
            environment_sensors,
            appliances,
            maccas,
            presence_sensors,
            plant_sensors,
            triggers,
            mut workflows,
            graphql,
            s3,
        } = self;

        let mut triggers: Vec<Trigger> = triggers.into_iter().flatten().collect();

        let doors = doors
            .into_iter()
            .map(|(addr, d)| Ok((addr, d.resolve(&notify_targets)?)))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let appliances = appliances
            .into_iter()
            .map(|(addr, a)| Ok((addr, a.resolve(&notify_targets)?)))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let environment_sensors = environment_sensors
            .into_iter()
            .map(|s| s.resolve(&devices))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let presence_sensors = presence_sensors
            .into_iter()
            .map(|p| p.resolve(&devices))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let plant_sensors = plant_sensors
            .into_iter()
            .map(|p| p.resolve(&devices))
            .collect::<Result<HashMap<_, _>, String>>()?;

        for trigger in &mut triggers {
            trigger.resolve_devices(&devices)?;
        }

        for workflow in workflows.values_mut() {
            workflow.resolve_devices(&devices)?;
        }

        let maccas = maccas.resolve(&notify_targets)?;

        Ok(Settings {
            api_key,
            database_url,
            fcm_project_id,
            fcm_service_account_json,
            mqtt_url,
            mqtt_username,
            mqtt_password,
            unifi_webhook_secret,
            android_app_webhook_secret,
            doors,
            environment_sensors,
            appliances,
            maccas,
            presence_sensors,
            plant_sensors,
            triggers,
            workflows,
            graphql,
            s3,
        })
    }
}

#[derive(Clone)]
pub struct SettingsContainer {
    inner: Arc<ArcSwap<Settings>>,
}

impl SettingsContainer {
    fn build(config: Config) -> Result<Settings, ConfigError> {
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

    pub fn new() -> Result<Self, ConfigError> {
        // TODO: fetch config from catalog, falling back to file if not found or other error
        let config = Self::config_sources(&PathBuf::from("./config"))?
            .add_source(Environment::default().separator("__"))
            .build()?;

        Ok(Self {
            inner: Arc::new(ArcSwap::new(Arc::new(Self::build(config)?))),
        })
    }

    pub fn load_full(&self) -> Arc<Settings> {
        self.inner.load_full()
    }

    pub fn load(&self) -> Guard<Arc<Settings>> {
        self.inner.load()
    }

    #[allow(unused)]
    pub fn reload(&self, new_settings: String) -> Result<(), ConfigError> {
        let config = Config::builder()
            .add_source(File::from_str(&new_settings, FileFormat::Yaml))
            .add_source(Environment::default().separator("__"))
            .build()?;

        self.inner.store(Arc::new(Self::build(config)?));

        Ok(())
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
maccas:
  webhook_secret: x
"#;
        let config = SettingsContainer::config_sources(Path::new("./config"))
            .unwrap()
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        let settings = SettingsContainer::build(config).unwrap();

        // a switch trigger resolves device aliases on both its matcher and steps
        let switch_trigger = settings
            .triggers
            .iter()
            .find(|t| {
                matches!(&t.on, trigger::TriggerMatcher::Switch { ieee_addr, action }
                if ieee_addr == "0x00158d008bbe0316" && action == "single")
            })
            .expect("expected a switch trigger for the small switch");
        let workflow::Step::Light { ieee_addr, .. } = &switch_trigger.run[0] else {
            panic!("expected a light step");
        };
        assert_eq!(ieee_addr, "0x94a081fffe2eedc0");

        // a cron trigger parses its schedule and anchor
        assert!(settings.triggers.iter().any(|t| t.name == "Bins"
            && matches!(&t.on, trigger::TriggerMatcher::Cron { .. })));

        // a zigbee source written as a device alias resolves to its address
        assert!(settings.presence_sensors.contains_key("0x54ef441000dbc81c"));

        // esphome sensor keyed by node name with explicit source
        assert_eq!(
            settings.presence_sensors["apollo-mtr-1-livingroom"]
                .motion_entity
                .as_deref(),
            Some("ld2450_moving_target")
        );
        assert_eq!(
            settings.environment_sensors["apollo-mtr-1-livingroom"].sensor_type,
            EnvironmentSensorType::Esphome
        );
    }
}
