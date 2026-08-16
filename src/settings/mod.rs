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

pub mod alarm;
pub mod auth;
pub mod bom;
pub mod de;
pub mod device;
pub mod door;
pub mod eink;
pub mod environment;
pub mod home_assistant;
pub mod jellyfin;
pub mod light;
pub mod location;
pub mod media_player;
pub mod notify;
pub mod plant;
pub mod presence;
pub mod roborock;
pub mod s3;
pub mod solar;
pub mod sun;
pub mod switch;
pub mod template;
pub mod trigger;
pub mod trmnl;
pub mod valetudo;
pub mod watchdog;
pub mod woolworths;
pub mod workflow;
pub mod zigbee_model;

pub use alarm::AlarmSettings;
pub use auth::{ApiKeySettings, OAuthSettings};
pub use bom::BomSettings;
pub use device::{BatterySettings, DeviceWatchdog, RawDeviceWatchdog};
pub use door::{ArmedDoorStates, DoorSettings};
pub use eink::{
    Album, DashboardView, EinkDisplaySettings, EinkGlobalSettings, EinkMode, EinkModeConfig,
    Orientation, PaletteColor, PartialRefresh, RawEinkDisplayBlock, RedditFeed, RedditTimespan,
    SleepWindow,
};
pub use environment::{
    EnvironmentSensorSettings, EnvironmentSensorType, Metric, RawEnvironmentBlock,
};
pub use home_assistant::{EntitySettings, HomeAssistantSettings};
pub use jellyfin::JellyfinSettings;
pub use light::RawLightBlock;
pub use location::LocationSettings;
pub use media_player::{MediaPlayerSettings, RawMediaPlayerBlock};
pub use notify::{NotifySource, NotifyTargets};
pub use plant::{PlantSensorSettings, RawPlantBlock};
pub use presence::{PresenceSensorType, PresenceSettings, RawPresenceBlock};
pub use roborock::{RawRoborockBlock, RoborockField, RoborockSettings};
pub use s3::S3Settings;
pub use solar::SolarSettings;
pub use sun::SunSettings;
pub use switch::{RawSmartSwitchBlock, SwitchRole};
pub use template::TemplateString;
pub use trigger::TriggerMatcher;
pub use trmnl::{RawTrmnlBlock, TrmnlDeviceSettings, TrmnlSettings};
pub use valetudo::{RawValetudoBlock, ValetudoSettings};
pub use watchdog::WatchdogSettings;
pub use woolworths::WoolworthsSettings;
pub use workflow::{Workflow, WorkflowSettings};
pub use zigbee_model::{RawZigbeeModelProfile, ZigbeeField, ZigbeeFieldType, ZigbeeModelProfile};

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
    pub fcm_project_id: String,
    pub fcm_service_account_json: String,
    pub mqtt_url: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub workflows: HashMap<String, Workflow>,
    pub workflow: WorkflowSettings,
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
    pub bom: BomSettings,
    pub eink_display: EinkGlobalSettings,
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
    zigbee_models: HashMap<String, RawZigbeeModelProfile>,
    #[serde(default)]
    workflows: Vec<Vec<Workflow>>,
    s3: S3Settings,
    watchdog: WatchdogSettings,
    workflow: WorkflowSettings,
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
    bom: BomSettings,
    #[serde(default)]
    eink_display: eink::RawEinkGlobal,
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
            zigbee_models,
            workflows,
            s3,
            watchdog,
            workflow,
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
            bom,
            eink_display,
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

        let registry = DeviceRegistry::build(devices, &notify_targets, zigbee_models)?;
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

        for workflow in resolved.values() {
            for target in workflow.run_workflow_targets() {
                if !resolved.contains_key(target) {
                    return Err(format!(
                        "workflow '{}': run_workflow references unknown workflow '{target}'",
                        workflow.name
                    ));
                }
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
                workflow,
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
                bom,
                eink_display: eink_display.resolve(),
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

    #[test]
    fn tmp_hallway_epd_registers_battery() {
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
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
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
    fn run_workflow_rejects_an_unknown_target() {
        let raw: RawSettings = serde_yaml::from_str(
            r#"
api_key: x
database_url: x
zigbee_models: {}
mqtt_url: x
mqtt_username: x
mqtt_password: x
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
workflows:
  - - name: Caller
      slug: caller
      run:
        - type: run_workflow
          workflow: does-not-exist
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
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
workflows:
  - - name: Callee
      slug: callee
      run: []
    - name: Caller
      slug: caller
      run:
        - type: run_workflow
          workflow: Callee
"#,
        )
        .unwrap();

        raw.resolve().expect("a known target resolves");
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
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
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
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
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
unifi_webhook_secret: x
android_app_webhook_secret: x
s3: { bucket: b, region: r }
watchdog: { enabled: false, timeout: 30m, check_interval: 5m, realert_after: 6h }
workflow: { workers: 12 }
location: { latitude: 0.0, longitude: 0.0 }
sun: { catch_up_within: 2h }
bom: { url: "http://bom" }
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
}
