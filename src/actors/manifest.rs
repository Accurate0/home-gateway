use std::future::Future;
use std::pin::Pin;

use ractor::{ActorProcessingErr, ActorRef};

use crate::actors::alarm::AlarmActor;
use crate::actors::devices::door_events::DoorEventsSupervisor;
use crate::actors::devices::handler::{DeviceHandler, spawn_handler};
use crate::actors::devices::{
    control_switch::ControlSwitchHandler, door_sensor::DoorSensorHandler,
    environment_sensor::EnvironmentSensorHandler, light::LightHandler,
    media_player::MediaPlayerHandler, plant_sensor::PlantSensorHandler,
    presence_sensor::PresenceSensorHandler, robot_vacuum::RobotVacuumHandler,
    smart_switch::SmartSwitchHandler,
};
use crate::actors::eink_display::EInkDisplayActor;
use crate::actors::integrations::{
    home_assistant::HomeAssistantActor, jellyfin::JellyfinActor, solar::SolarActor,
    synergy::SynergyActor, transperth::TransperthActor, trmnl::TrmnlActor,
    unifi::UnifiConnectedClientHandler, woolworths::WoolworthsActor,
};
use crate::actors::root::RootMessage;
use crate::actors::sun::SunActor;
use crate::actors::system::{
    adhoc::AdhocTaskActor, battery::BatteryActor, cron::CronActor, mqtt_ingest::MqttIngest,
    push::PushWorker, watchdog::WatchdogActor,
};
use crate::actors::workflows::{WorkflowWorker, dispatcher::WorkflowDispatcher};
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::jellyfin::Jellyfin;
use crate::integrations::solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI};
use crate::integrations::transperth::Transperth;
use crate::integrations::trmnl::Trmnl;
use crate::integrations::woolworths::Woolworths;
use crate::settings::SettingsContainer;
use crate::state::{AppState, HandleRegistry};

pub enum Spawned {
    Started,
    Skipped,
}

type SpawnFuture = Pin<Box<dyn Future<Output = Result<Spawned, ActorProcessingErr>> + Send>>;
type SpawnFn = fn(ActorRef<RootMessage>, AppState) -> SpawnFuture;

pub enum Requirement {
    Handle {
        label: &'static str,
        present: fn(&HandleRegistry) -> bool,
    },
    Setting {
        label: &'static str,
        present: fn(&SettingsContainer) -> bool,
    },
}

impl Requirement {
    pub fn label(&self) -> &'static str {
        match self {
            Requirement::Handle { label, .. } => label,
            Requirement::Setting { label, .. } => label,
        }
    }

    pub fn satisfied_by(&self, state: &AppState) -> bool {
        match self {
            Requirement::Handle { present, .. } => present(&state.handles),
            Requirement::Setting { present, .. } => present(&state.settings),
        }
    }
}

pub struct ActorSpec {
    pub name: &'static str,
    pub autostart: bool,
    pub optional: bool,
    pub requires: &'static [Requirement],
    pub spawn: SpawnFn,
}

impl ActorSpec {
    pub fn unmet_requirement(&self, state: &AppState) -> Option<&'static str> {
        self.requires
            .iter()
            .find(|requirement| !requirement.satisfied_by(state))
            .map(Requirement::label)
    }
}

pub fn find(name: &str) -> Option<&'static ActorSpec> {
    ACTORS.iter().find(|spec| spec.name == name)
}

macro_rules! plain {
    ($actor:ident) => {
        ActorSpec {
            name: $actor::NAME,
            autostart: true,
            optional: false,
            requires: &[],
            spawn: |root, shared_actor_state| {
                Box::pin(async move {
                    root.spawn_linked(
                        Some($actor::NAME.to_owned()),
                        $actor { shared_actor_state },
                        (),
                    )
                    .await?;

                    Ok(Spawned::Started)
                })
            },
        }
    };
}

macro_rules! device {
    ($handler:path) => {
        ActorSpec {
            name: <$handler as DeviceHandler>::NAME,
            autostart: true,
            optional: false,
            requires: &[],
            spawn: |root, shared_actor_state| {
                Box::pin(async move {
                    spawn_handler::<$handler>(&root, shared_actor_state).await?;

                    Ok(Spawned::Started)
                })
            },
        }
    };
}

pub static ACTORS: &[ActorSpec] = &[
    device!(LightHandler),
    device!(DoorSensorHandler),
    device!(PresenceSensorHandler),
    device!(EnvironmentSensorHandler),
    device!(PlantSensorHandler),
    device!(SmartSwitchHandler),
    device!(ControlSwitchHandler),
    device!(MediaPlayerHandler),
    device!(RobotVacuumHandler),
    plain!(AlarmActor),
    plain!(BatteryActor),
    plain!(CronActor),
    plain!(DoorEventsSupervisor),
    plain!(EInkDisplayActor),
    plain!(SunActor),
    plain!(SynergyActor),
    plain!(UnifiConnectedClientHandler),
    plain!(WatchdogActor),
    plain!(WorkflowDispatcher),
    ActorSpec {
        name: AdhocTaskActor::NAME,
        autostart: true,
        optional: true,
        requires: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                root.spawn_linked(
                    Some(AdhocTaskActor::NAME.to_owned()),
                    AdhocTaskActor { shared_actor_state },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: PushWorker::NAME,
        autostart: true,
        optional: false,
        requires: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::push::spawn::spawn_push(&root, shared_actor_state).await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: WorkflowWorker::NAME,
        autostart: true,
        optional: false,
        requires: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::workflows::spawn::spawn_workflows(&root, shared_actor_state).await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: HomeAssistantActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "home assistant",
            present: |handles| handles.contains::<HomeAssistant>(),
        }],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                root.spawn_linked(
                    Some(HomeAssistantActor::NAME.to_owned()),
                    HomeAssistantActor { shared_actor_state },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: JellyfinActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "jellyfin",
            present: |handles| handles.contains::<Jellyfin>(),
        }],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let jellyfin = shared_actor_state.handles.expect::<Jellyfin>().clone();

                root.spawn_linked(
                    Some(JellyfinActor::NAME.to_owned()),
                    JellyfinActor {
                        shared_actor_state,
                        jellyfin,
                    },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: TransperthActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "transperth",
            present: |handles| handles.contains::<Transperth>(),
        }],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let transperth = shared_actor_state.handles.expect::<Transperth>().clone();

                root.spawn_linked(
                    Some(TransperthActor::NAME.to_owned()),
                    TransperthActor {
                        shared_actor_state,
                        transperth,
                    },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: TrmnlActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Setting {
            label: "trmnl_api_key",
            present: |settings| settings.trmnl_api_key.is_some(),
        }],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let api_key = shared_actor_state
                    .settings
                    .trmnl_api_key
                    .clone()
                    .expect("trmnl_api_key was required");

                let trmnl = Trmnl::new(api_key, shared_actor_state.settings.trmnl.base_url.clone());

                root.spawn_linked(
                    Some(TrmnlActor::NAME.to_owned()),
                    TrmnlActor {
                        shared_actor_state,
                        trmnl,
                    },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: WoolworthsActor::NAME,
        autostart: true,
        optional: false,
        requires: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let woolworths = Woolworths::new(shared_actor_state.db.clone());

                root.spawn_linked(
                    Some(WoolworthsActor::NAME.to_owned()),
                    WoolworthsActor {
                        shared_actor_state,
                        woolworths,
                    },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: SolarActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "goodwe",
            present: |handles| handles.contains::<GoodWeSemsAPI>(),
        }],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let goodwe = shared_actor_state.handles.expect::<GoodWeSemsAPI>().clone();

                let weather = WeatherAPI::new()?;

                root.spawn_linked(
                    Some(SolarActor::NAME.to_owned()),
                    SolarActor {
                        shared_actor_state,
                        goodwe,
                        weather,
                    },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: MqttIngest::NAME,
        autostart: true,
        optional: false,
        requires: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::mqtt_ingest::spawn::spawn_mqtt_ingest(
                    &root,
                    shared_actor_state,
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn only_the_optional_integrations_declare_requirements() {
        let gated: Vec<&str> = ACTORS
            .iter()
            .filter(|spec| !spec.requires.is_empty())
            .map(|spec| spec.name)
            .collect();

        assert_eq!(
            gated,
            vec![
                HomeAssistantActor::NAME,
                JellyfinActor::NAME,
                TransperthActor::NAME,
                TrmnlActor::NAME,
                SolarActor::NAME,
            ]
        );
    }

    #[test]
    fn an_empty_registry_satisfies_only_the_ungated_actors() {
        let handles = HandleRegistry::default();

        for spec in ACTORS {
            let satisfied = spec.requires.iter().all(|requirement| match requirement {
                Requirement::Handle { present, .. } => present(&handles),
                Requirement::Setting { .. } => true,
            });

            let gated_by_a_handle = spec
                .requires
                .iter()
                .any(|requirement| matches!(requirement, Requirement::Handle { .. }));

            assert_eq!(
                satisfied, !gated_by_a_handle,
                "{} should be gated by its handle requirement",
                spec.name
            );
        }
    }

    #[test]
    fn actor_names_are_unique() {
        let mut seen = HashSet::new();
        for spec in ACTORS {
            assert!(
                seen.insert(spec.name),
                "duplicate actor name: {}",
                spec.name
            );
        }
    }

    #[test]
    fn every_actor_is_findable_by_name() {
        for spec in ACTORS {
            assert!(find(spec.name).is_some(), "{} not findable", spec.name);
        }
    }

    #[test]
    fn mqtt_ingest_starts_last_so_packets_arrive_after_the_handlers() {
        let last = ACTORS.last().expect("a non-empty manifest");

        assert_eq!(last.name, MqttIngest::NAME);
        assert!(last.autostart);
    }
}
