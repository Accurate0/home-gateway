use arc_swap::{ArcSwap, Guard};
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

pub mod appliance;
pub mod door;
pub mod maccas;
pub mod notify;
pub mod presence;
pub mod reminder;
pub mod switch;
pub mod temperature;
pub mod workflow;

pub use appliance::ApplianceSettings;
pub use door::{ArmedDoorStates, DoorSettings};
pub use maccas::MaccasSettings;
pub use notify::{NotifySource, NotifyTargets};
pub use presence::{PresenceActionId, PresenceSensorType, PresenceSettings};
pub use reminder::{ReminderSettings, ReminderState};
pub use switch::SwitchSettings;
pub use temperature::{TemperatureSensorSettings, TemperatureSensorType};

use appliance::RawApplianceSettings;
use door::RawDoorSettings;
use maccas::RawMaccasSettings;
use presence::RawPresenceSettings;
use reminder::RawReminderSettings;
use temperature::RawTemperatureSensor;

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
    pub reminders: Vec<ReminderSettings>,
    pub doors: HashMap<IEEEAddress, DoorSettings>,
    pub temperature_sensors: HashMap<String, TemperatureSensorSettings>,
    pub appliances: HashMap<IEEEAddress, ApplianceSettings>,
    pub maccas: MaccasSettings,
    pub discord_token: String,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub switches: HashMap<IEEEAddress, SwitchSettings>,
    pub presence_sensors: HashMap<String, PresenceSettings>,
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
    discord_token: String,
    unifi_webhook_secret: String,
    android_app_webhook_secret: String,
    #[serde(default)]
    devices: DeviceAliases,
    #[serde(default)]
    notify_targets: NotifyTargets,
    #[serde(default)]
    reminders: Vec<RawReminderSettings>,
    #[serde(default)]
    doors: HashMap<IEEEAddress, RawDoorSettings>,
    #[serde(default)]
    temperature_sensors: Vec<RawTemperatureSensor>,
    #[serde(default)]
    appliances: HashMap<IEEEAddress, RawApplianceSettings>,
    maccas: RawMaccasSettings,
    #[serde(default)]
    switches: HashMap<IEEEAddress, SwitchSettings>,
    #[serde(default)]
    presence_sensors: Vec<RawPresenceSettings>,
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
            discord_token,
            unifi_webhook_secret,
            android_app_webhook_secret,
            devices,
            notify_targets,
            reminders,
            doors,
            temperature_sensors,
            appliances,
            maccas,
            mut switches,
            presence_sensors,
            s3,
        } = self;

        let reminders = reminders
            .into_iter()
            .map(|r| r.resolve(&notify_targets))
            .collect::<Result<Vec<_>, String>>()?;

        let doors = doors
            .into_iter()
            .map(|(addr, d)| Ok((addr, d.resolve(&notify_targets)?)))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let appliances = appliances
            .into_iter()
            .map(|(addr, a)| Ok((addr, a.resolve(&notify_targets)?)))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let temperature_sensors = temperature_sensors
            .into_iter()
            .map(|s| s.resolve(&devices))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let presence_sensors = presence_sensors
            .into_iter()
            .map(|p| p.resolve(&devices))
            .collect::<Result<HashMap<_, _>, String>>()?;

        for switch in switches.values_mut() {
            switch.resolve_devices(&devices)?;
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
            discord_token,
            unifi_webhook_secret,
            android_app_webhook_secret,
            reminders,
            doors,
            temperature_sensors,
            appliances,
            maccas,
            switches,
            presence_sensors,
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

    pub fn new() -> Result<Self, ConfigError> {
        // TODO: fetch config from catalog, falling back to file if not found or other error
        let file_path = PathBuf::from("./config.yaml");
        let file = File::from(file_path).required(true);

        let config = Config::builder()
            .add_source(file)
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
        let file_contents = std::fs::read_to_string("./config.yaml").unwrap();
        // secrets normally come from the environment; supply dummies for the test
        let secrets = r#"
api_key: x
database_url: x
mqtt_url: x
mqtt_username: x
mqtt_password: x
discord_token: x
unifi_webhook_secret: x
android_app_webhook_secret: x
maccas:
  webhook_secret: x
"#;
        let config = Config::builder()
            .add_source(File::from_str(&file_contents, FileFormat::Yaml))
            .add_source(File::from_str(secrets, FileFormat::Yaml))
            .build()
            .unwrap();

        let settings = SettingsContainer::build(config).unwrap();

        // device alias resolved to its raw address
        let small_switch = &settings.switches["0x00158d008bbe0316"].actions["single"].workflow;
        let workflow::WorkflowEntityType::Light { ieee_addr, .. } = &small_switch.run[0] else {
            panic!("expected a light step");
        };
        assert_eq!(ieee_addr, "0x94a081fffe2eedc0");

        // notify target resolved
        let bins = settings.reminders.iter().find(|r| r.id == "bins").unwrap();
        assert!(matches!(bins.notify[0], NotifySource::AndroidApp));

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
            settings.temperature_sensors["apollo-mtr-1-livingroom"].sensor_type,
            TemperatureSensorType::Esphome
        );
    }
}
