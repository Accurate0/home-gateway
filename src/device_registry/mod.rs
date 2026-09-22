mod capability;
mod device;
mod device_config;
pub mod last_seen;
pub mod lua;
mod raw_device;
mod raw_transport;
mod roles;
mod transport;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::decoding::{DecodedDevice, DeviceModels, ModelEntities, ModelProfile};
use crate::event_bus::SensorMetric;
use crate::integrations::mqtt::{MqttProtocol, TopicVars};
use crate::settings::notify::NotifyTargets;
use crate::settings::{
    BatterySettings, DeviceAliases, DeviceWatchdog, DoorSettings, EinkDisplaySettings,
    EnvironmentSensorSettings, IEEEAddress, MediaPlayerSettings, MqttProtocols,
    PlantSensorSettings, PresenceSettings, RobotVacuumSettings, TrmnlDeviceSettings,
};

pub use capability::Capability;
pub use device::Device;
pub use device_config::DeviceConfig;
pub use raw_device::RawDevice;
pub use raw_transport::RawTransport;
pub use roles::Roles;
pub use transport::Transport;

use roles::RoleContext;

#[derive(Debug)]
pub struct DeviceRegistryInner {
    devices: HashMap<String, Device>,
    aliases: DeviceAliases,
    mqtt_protocols: MqttProtocols,
    mqtt_topics: HashMap<String, Vec<String>>,
    home_assistant_entities: HashMap<String, String>,
    watchdog: HashMap<String, DeviceWatchdog>,
    disabled: HashSet<String>,
    known_devices: RwLock<HashMap<IEEEAddress, String>>,
}

#[derive(Debug, Clone)]
pub struct DeviceRegistry {
    inner: Arc<DeviceRegistryInner>,
}

impl std::ops::Deref for DeviceRegistry {
    type Target = DeviceRegistryInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DeviceRegistry {
    pub fn build(
        raw: Vec<RawDevice>,
        notify: &NotifyTargets,
        models: &DeviceModels,
        mqtt_protocols: &MqttProtocols,
    ) -> Result<Self, String> {
        let mut reg = DeviceRegistryInner {
            devices: HashMap::new(),
            aliases: DeviceAliases::default(),
            mqtt_protocols: mqtt_protocols.clone(),
            mqtt_topics: HashMap::new(),
            home_assistant_entities: HashMap::new(),
            watchdog: HashMap::new(),
            disabled: HashSet::new(),
            known_devices: RwLock::new(HashMap::new()),
        };

        for device in raw {
            let RawDevice {
                id,
                state,
                transport,
                model,
                roles,
                watchdog,
                room,
            } = device;

            if !state.is_enabled() {
                tracing::warn!("device {id} is {state}, not registering it");
                reg.disabled.insert(id);
                continue;
            }

            let transport_kind = transport.kind();
            let address = transport.into_address();

            if reg.aliases.insert(id.clone(), address.clone()).is_some() {
                return Err(format!("duplicate device id: {id}"));
            }

            let profile = resolve_model(&id, transport_kind, model, models)?;

            let roles = Roles::resolve(
                &RoleContext {
                    id: &id,
                    address: &address,
                    transport: transport_kind,
                    profile: profile.as_deref(),
                    notify,
                },
                roles,
            )?;

            if let Some(profile) = &profile {
                reg.register_entities(&id, &address, profile)?;
            }

            let source = profile
                .as_deref()
                .map_or_else(|| transport_kind.to_string(), ModelProfile::source);

            let watchdog_key = format!("{source}:{id}");

            let model_timeout = profile.as_deref().and_then(|profile| profile.watchdog);

            let watchdog = match (watchdog, model_timeout) {
                (Some(watchdog), _) => Some(watchdog.resolve(notify, model_timeout)?),
                (None, Some(timeout)) => Some(DeviceWatchdog {
                    timeout: Some(timeout),
                    notify: Vec::new(),
                }),
                (None, None) => None,
            };

            if let Some(watchdog) = watchdog {
                reg.watchdog.insert(watchdog_key.clone(), watchdog);
            }

            let device = Device {
                id: id.clone(),
                address: address.clone(),
                transport: transport_kind,
                profile,
                room,
                watchdog_key,
                roles,
            };

            if let Some(existing) = reg.devices.insert(address.clone(), device) {
                return Err(format!(
                    "device {id}: address `{address}` is already used by device {}",
                    existing.id
                ));
            }
        }

        Ok(Self {
            inner: Arc::new(reg),
        })
    }
}

fn resolve_model(
    id: &str,
    transport: Transport,
    model: Option<String>,
    models: &DeviceModels,
) -> Result<Option<Arc<ModelProfile>>, String> {
    let (profiles, slug) = match (models.for_transport(transport), model) {
        (None, None) => return Ok(None),
        (None, Some(_)) => {
            return Err(format!(
                "device {id}: the {transport} transport does not take a `model:`"
            ));
        }
        (Some(_), None) => {
            return Err(format!(
                "device {id}: {transport} transport requires a `model:`"
            ));
        }
        (Some(profiles), Some(slug)) => (profiles, slug),
    };

    let Some(profile) = profiles.get(&slug) else {
        let mut known: Vec<_> = profiles.keys().map(String::as_str).collect();
        known.sort_unstable();

        return Err(format!(
            "device {id}: unknown {transport} model `{slug}`; known models are {}",
            known.join(", ")
        ));
    };

    Ok(Some(profile.clone()))
}

impl DeviceRegistryInner {
    fn register_entities(
        &mut self,
        id: &str,
        address: &str,
        profile: &ModelProfile,
    ) -> Result<(), String> {
        if let Some(protocol) = profile.protocol {
            let topics = self.device_topics(protocol, address, &profile.entities);

            self.mqtt_topics.insert(address.to_owned(), topics);
        }

        match &profile.entities {
            ModelEntities::Payload | ModelEntities::Esphome(_) => {}
            ModelEntities::HomeAssistant(entities) => {
                let entity_ids =
                    std::iter::once(address.to_owned()).chain(entities.resolve(address));

                for entity_id in entity_ids {
                    if self
                        .home_assistant_entities
                        .insert(entity_id.clone(), address.to_owned())
                        .is_some()
                    {
                        return Err(format!(
                            "device {id}: home assistant entity `{entity_id}` is claimed by another device"
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn device_topics(
        &self,
        protocol: MqttProtocol,
        address: &str,
        entities: &ModelEntities,
    ) -> Vec<String> {
        let base = TopicVars::from([("address".to_owned(), address.to_owned())]);

        let entity_vars: Vec<TopicVars> = match entities {
            ModelEntities::Esphome(entities) => entities
                .targets(address)
                .map(|target| {
                    let mut vars = base.clone();
                    vars.insert("domain".to_owned(), target.domain.to_string());
                    vars.insert("object_id".to_owned(), target.object_id);
                    vars
                })
                .collect(),
            ModelEntities::Payload | ModelEntities::HomeAssistant(_) => vec![base],
        };

        let topics: BTreeSet<String> = self
            .mqtt_protocols
            .get(protocol)
            .topics
            .values()
            .flat_map(|template| entity_vars.iter().map(|vars| template.filter(vars)))
            .collect();

        topics.into_iter().collect()
    }

    fn each<'a, T: 'a>(
        &'a self,
        role: impl Fn(&'a Roles) -> Option<&'a T> + 'a,
    ) -> impl Iterator<Item = (&'a String, &'a T)> + 'a {
        self.devices
            .iter()
            .filter_map(move |(address, device)| Some((address, role(&device.roles)?)))
    }

    fn roles(&self, address: &str) -> Option<&Roles> {
        self.devices.get(address).map(|device| &device.roles)
    }

    pub fn device(&self, address: &str) -> Option<&Device> {
        self.devices.get(address)
    }

    pub fn aliases(&self) -> &DeviceAliases {
        &self.aliases
    }

    pub fn disabled(&self) -> &HashSet<String> {
        &self.disabled
    }

    pub fn address_or_self<'a>(&'a self, reference: &'a str) -> &'a str {
        self.aliases
            .get(reference)
            .map_or(reference, |a| a.as_str())
    }

    pub fn id_for_address(&self, address: &str) -> Option<&str> {
        self.devices.get(address).map(|device| device.id.as_str())
    }

    pub async fn record_friendly_name(&self, address: IEEEAddress, name: String) {
        self.known_devices.write().await.insert(address, name);
    }

    pub async fn friendly_name(&self, address: &str) -> Option<String> {
        self.known_devices.read().await.get(address).cloned()
    }

    pub async fn address_for_friendly_name(&self, name: &str) -> Option<String> {
        self.known_devices
            .read()
            .await
            .iter()
            .find(|(_, known)| known.as_str() == name)
            .map(|(address, _)| address.clone())
    }

    pub fn decoded(&self, address: &str) -> Option<DecodedDevice> {
        self.devices.get(address)?.decoded()
    }

    pub fn mqtt_device(&self, protocol: MqttProtocol, address: &str) -> Option<DecodedDevice> {
        let device = self.devices.get(address)?;

        match device.profile.as_ref()?.protocol {
            Some(declared) if declared == protocol => device.decoded(),
            Some(_) | None => None,
        }
    }

    pub fn mqtt_protocols(&self) -> &MqttProtocols {
        &self.mqtt_protocols
    }

    pub fn mqtt_subscriptions(&self) -> BTreeSet<String> {
        let none = TopicVars::new();

        let features = self.mqtt_protocols.iter().flat_map(|(_, settings)| {
            [&settings.directory, &settings.discovery]
                .into_iter()
                .flatten()
                .map(|template| template.filter(&none))
        });

        self.mqtt_topics
            .values()
            .flatten()
            .cloned()
            .chain(features)
            .collect()
    }

    pub fn mqtt_topics_for(&self, address: &str) -> &[String] {
        self.mqtt_topics.get(address).map_or(&[], Vec::as_slice)
    }

    pub async fn mqtt_command_topic(
        &self,
        address: &str,
        extra: TopicVars,
    ) -> Result<String, String> {
        let protocol = self
            .devices
            .get(address)
            .and_then(|device| device.profile.as_ref()?.protocol)
            .ok_or_else(|| format!("{address} is not an mqtt device"))?;

        let Some(command) = &self.mqtt_protocols.get(protocol).command else {
            return Err(format!("the {protocol} protocol has no `command` topic"));
        };

        let name = self
            .friendly_name(address)
            .await
            .unwrap_or_else(|| address.to_owned());

        let mut vars = extra;
        vars.insert("address".to_owned(), address.to_owned());
        vars.insert("name".to_owned(), name);

        command.render(&vars)
    }

    pub fn home_assistant_device(&self, entity_id: &str) -> Option<DecodedDevice> {
        self.decoded(self.home_assistant_entities.get(entity_id)?)
    }

    pub fn esphome_light(&self, address: &str) -> Option<&str> {
        self.devices.get(address)?.esphome_entities()?.light()
    }

    pub fn capabilities(&self, address: &str) -> &[Capability] {
        self.devices.get(address).map_or(&[], Device::capabilities)
    }

    pub fn room(&self, address: &str) -> Option<&str> {
        self.devices.get(address)?.room.as_deref()
    }

    pub fn sensor_metrics(&self, address: &str) -> Vec<SensorMetric> {
        let Some(device) = self.devices.get(address) else {
            return Vec::new();
        };

        let Some(profile) = &device.profile else {
            return Vec::new();
        };

        let environment = device
            .roles
            .environment
            .iter()
            .flat_map(|_| profile.environment.iter().copied().map(SensorMetric::from));

        let plant = device
            .roles
            .plant
            .iter()
            .flat_map(|_| profile.plant.iter().cloned().map(SensorMetric::from));

        environment.chain(plant).collect()
    }

    pub fn watchdog_devices(&self) -> impl Iterator<Item = (&String, &DeviceWatchdog)> {
        self.watchdog.iter()
    }

    pub fn watchdog_key(&self, address_or_id: &str) -> Option<&str> {
        let device = self
            .devices
            .get(address_or_id)
            .or_else(|| self.devices.get(self.aliases.get(address_or_id)?))?;

        Some(&device.watchdog_key)
    }

    pub fn door(&self, address: &str) -> Option<&DoorSettings> {
        self.roles(address)?.door.as_ref()
    }

    pub fn doors(&self) -> impl Iterator<Item = (&String, &DoorSettings)> {
        self.each(|roles| roles.door.as_ref())
    }

    pub fn control_switch(&self, address: &str) -> bool {
        self.roles(address)
            .is_some_and(|roles| roles.control_switch)
    }

    pub fn smart_switch(&self, address: &str) -> Option<&String> {
        self.roles(address)?.smart_switch.as_ref()
    }

    pub fn environment(&self, address: &str) -> Option<&EnvironmentSensorSettings> {
        self.roles(address)?.environment.as_ref()
    }

    pub fn environment_devices(
        &self,
    ) -> impl Iterator<Item = (&String, &EnvironmentSensorSettings)> {
        self.each(|roles| roles.environment.as_ref())
    }

    pub fn presence(&self, address: &str) -> Option<&PresenceSettings> {
        self.roles(address)?.presence.as_ref()
    }

    pub fn presence_devices(&self) -> impl Iterator<Item = (&String, &PresenceSettings)> {
        self.each(|roles| roles.presence.as_ref())
    }

    pub fn plant(&self, address: &str) -> Option<&PlantSensorSettings> {
        self.roles(address)?.plant.as_ref()
    }

    pub fn battery(&self, address: &str) -> Option<&BatterySettings> {
        self.roles(address)?.battery.as_ref()
    }

    pub fn light(&self, address: &str) -> Option<&String> {
        self.roles(address)?.light.as_ref()
    }

    pub fn lights(&self) -> impl Iterator<Item = (&String, &String)> {
        self.each(|roles| roles.light.as_ref())
    }

    pub fn eink_display(&self, address: &str) -> Option<&EinkDisplaySettings> {
        self.roles(address)?.eink_display.as_ref()
    }

    pub fn eink_displays(&self) -> impl Iterator<Item = (&String, &EinkDisplaySettings)> {
        self.each(|roles| roles.eink_display.as_ref())
    }

    pub fn trmnl(&self, address: &str) -> Option<&TrmnlDeviceSettings> {
        self.roles(address)?.trmnl.as_ref()
    }

    pub fn trmnl_devices(&self) -> impl Iterator<Item = (&String, &TrmnlDeviceSettings)> {
        self.each(|roles| roles.trmnl.as_ref())
    }

    pub fn robot_vacuum(&self, address: &str) -> Option<&RobotVacuumSettings> {
        self.roles(address)?.robot_vacuum.as_ref()
    }

    pub fn robot_vacuums(&self) -> impl Iterator<Item = (&String, &RobotVacuumSettings)> {
        self.each(|roles| roles.robot_vacuum.as_ref())
    }

    pub fn media_player(&self, address: &str) -> Option<&MediaPlayerSettings> {
        self.roles(address)?.media_player.as_ref()
    }

    pub fn media_players(&self) -> impl Iterator<Item = (&String, &MediaPlayerSettings)> {
        self.each(|roles| roles.media_player.as_ref())
    }
}
