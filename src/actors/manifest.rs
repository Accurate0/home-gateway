use std::future::Future;
use std::pin::Pin;

use ractor::{ActorProcessingErr, ActorRef};

use crate::actors::alarm::AlarmActor;
use crate::actors::devices::door_events::DoorEventsSupervisor;
use crate::actors::devices::handler::{DeviceHandler, spawn_handler};
use crate::actors::devices::{
    control_switch::ControlSwitchHandler, door_sensor::DoorSensorHandler,
    environment_sensor::EnvironmentSensorHandler, garage_door::GarageDoorHandler,
    light::LightHandler, media_player::MediaPlayerHandler, plant_sensor::PlantSensorHandler,
    presence_sensor::PresenceSensorHandler, robot_vacuum::RobotVacuumHandler,
    smart_switch::SmartSwitchHandler,
};
use crate::actors::eink_display::EInkDisplayActor;
use crate::actors::integrations::{
    fuelwatch::FuelWatchActor, jellyfin::JellyfinActor, solar::SolarActor, synergy::SynergyActor,
    transperth::TransperthActor, trmnl::TrmnlActor, unifi::UnifiConnectedClientHandler,
    willyweather::WillyWeatherActor, woolworths::WoolworthsActor,
};
use crate::actors::root::RootMessage;
use crate::actors::sun::SunActor;
use crate::actors::system::{
    adhoc::AdhocTaskActor,
    battery::BatteryActor,
    cron::CronActor,
    esphome_native_api_ingest::EsphomeNativeApiIngest,
    home_assistant_ingest::HomeAssistantIngest,
    mqtt_ingest::MqttIngest,
    push::PushActor,
    reconciler::{ReconcilerSweeper, ReconcilerWorker},
    sampling::SamplingActor,
    tuya_ingest::TuyaIngest,
    watchdog::WatchdogActor,
};
use crate::actors::vacation::VacationActor;
use crate::actors::workflows::{WorkflowWorker, dispatcher::WorkflowDispatcher};
use crate::integrations::esphome_native_api::EsphomeNativeApi;
use crate::integrations::fuelwatch::FuelWatch;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::jellyfin::Jellyfin;
use crate::integrations::solar::{goodwe::GoodWeSemsAPI, weather::WeatherAPI};
use crate::integrations::transperth::Transperth;
use crate::integrations::trmnl::Trmnl;
use crate::integrations::tuya::Tuya;
use crate::integrations::willyweather::WillyWeather;
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
    pub depends_on: &'static [&'static str],
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

pub fn startup_order(specs: &[ActorSpec]) -> Result<Vec<&ActorSpec>, ActorProcessingErr> {
    for spec in specs {
        if let Some(missing) = spec
            .depends_on
            .iter()
            .find(|dependency| !specs.iter().any(|other| other.name == **dependency))
        {
            return Err(format!("{} depends on unknown actor `{missing}`", spec.name).into());
        }
    }

    let mut ordered: Vec<&ActorSpec> = Vec::with_capacity(specs.len());
    let mut pending: Vec<&ActorSpec> = specs.iter().collect();

    while !pending.is_empty() {
        let ready = pending.iter().position(|spec| {
            spec.depends_on
                .iter()
                .all(|dependency| ordered.iter().any(|started| started.name == *dependency))
        });

        let Some(index) = ready else {
            let stuck: Vec<&str> = pending.iter().map(|spec| spec.name).collect();

            return Err(format!("actor dependency cycle among: {}", stuck.join(", ")).into());
        };

        ordered.push(pending.remove(index));
    }

    Ok(ordered)
}

macro_rules! plain {
    ($actor:ident) => {
        plain!($actor, &[])
    };
    ($actor:ident, $deps:expr) => {
        ActorSpec {
            name: $actor::NAME,
            autostart: true,
            optional: false,
            requires: &[],
            depends_on: $deps,
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
        device!($handler, &[])
    };
    ($handler:path, $deps:expr) => {
        ActorSpec {
            name: <$handler as DeviceHandler>::NAME,
            autostart: true,
            optional: false,
            requires: &[],
            depends_on: $deps,
            spawn: |root, shared_actor_state| {
                Box::pin(async move {
                    spawn_handler::<$handler>(&root, shared_actor_state).await?;

                    Ok(Spawned::Started)
                })
            },
        }
    };
}

const DISPATCH_TARGETS: &[&str] = &[
    <LightHandler as DeviceHandler>::NAME,
    <DoorSensorHandler as DeviceHandler>::NAME,
    <PresenceSensorHandler as DeviceHandler>::NAME,
    <EnvironmentSensorHandler as DeviceHandler>::NAME,
    <PlantSensorHandler as DeviceHandler>::NAME,
    <SmartSwitchHandler as DeviceHandler>::NAME,
    <ControlSwitchHandler as DeviceHandler>::NAME,
    <MediaPlayerHandler as DeviceHandler>::NAME,
    <RobotVacuumHandler as DeviceHandler>::NAME,
    <GarageDoorHandler as DeviceHandler>::NAME,
    BatteryActor::NAME,
];

pub static ACTORS: &[ActorSpec] = &[
    device!(LightHandler),
    device!(DoorSensorHandler),
    device!(PresenceSensorHandler),
    device!(EnvironmentSensorHandler),
    device!(PlantSensorHandler),
    device!(SmartSwitchHandler, &[<LightHandler as DeviceHandler>::NAME]),
    device!(ControlSwitchHandler),
    device!(MediaPlayerHandler),
    device!(RobotVacuumHandler, &[BatteryActor::NAME]),
    device!(GarageDoorHandler, &[PushActor::NAME]),
    plain!(AlarmActor, &[WorkflowWorker::NAME]),
    plain!(BatteryActor),
    plain!(CronActor),
    plain!(DoorEventsSupervisor, &[PushActor::NAME]),
    plain!(EInkDisplayActor, &[BatteryActor::NAME]),
    ActorSpec {
        name: VacationActor::NAME,
        autostart: true,
        optional: true,
        requires: &[Requirement::Setting {
            label: "vacation.state",
            present: |settings| settings.vacation.state.is_enabled(),
        }],
        depends_on: &[<LightHandler as DeviceHandler>::NAME],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                root.spawn_linked(
                    Some(VacationActor::NAME.to_owned()),
                    VacationActor { shared_actor_state },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    plain!(SamplingActor),
    plain!(SynergyActor),
    plain!(UnifiConnectedClientHandler),
    plain!(WatchdogActor, &[PushActor::NAME]),
    plain!(WorkflowDispatcher, &[WorkflowWorker::NAME]),
    ActorSpec {
        name: AdhocTaskActor::NAME,
        autostart: true,
        optional: true,
        requires: &[],
        depends_on: &[],
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
        name: PushActor::NAME,
        autostart: true,
        optional: false,
        requires: &[],
        depends_on: &[],
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
        depends_on: &[
            <LightHandler as DeviceHandler>::NAME,
            <EnvironmentSensorHandler as DeviceHandler>::NAME,
            <PresenceSensorHandler as DeviceHandler>::NAME,
            <GarageDoorHandler as DeviceHandler>::NAME,
            DoorEventsSupervisor::NAME,
            SolarActor::NAME,
            PushActor::NAME,
        ],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::workflows::spawn::spawn_workflows(&root, shared_actor_state).await?;

                Ok(Spawned::Started)
            })
        },
    },
    plain!(SunActor, &[WorkflowDispatcher::NAME, WorkflowWorker::NAME]),
    ActorSpec {
        name: ReconcilerWorker::NAME,
        autostart: true,
        optional: true,
        requires: &[Requirement::Setting {
            label: "reconciler.state",
            present: |settings| settings.reconciler.state.is_enabled(),
        }],
        depends_on: &[<LightHandler as DeviceHandler>::NAME],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::reconciler::spawn_reconciler(&root, shared_actor_state)
                    .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: ReconcilerSweeper::NAME,
        autostart: true,
        optional: true,
        requires: &[Requirement::Setting {
            label: "reconciler.state",
            present: |settings| settings.reconciler.state.is_enabled(),
        }],
        depends_on: &[ReconcilerWorker::NAME],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                root.spawn_linked(
                    Some(ReconcilerSweeper::NAME.to_owned()),
                    ReconcilerSweeper { shared_actor_state },
                    (),
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: EsphomeNativeApiIngest::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "esphome native api",
            present: |handles| handles.contains::<EsphomeNativeApi>(),
        }],
        depends_on: DISPATCH_TARGETS,
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::esphome_native_api_ingest::spawn::spawn_esphome_native_api_ingest(
                    &root,
                    shared_actor_state,
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: TuyaIngest::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "tuya",
            present: |handles| handles.contains::<Tuya>(),
        }],
        depends_on: DISPATCH_TARGETS,
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::tuya_ingest::spawn::spawn_tuya_ingest(
                    &root,
                    shared_actor_state,
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: HomeAssistantIngest::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "home assistant",
            present: |handles| handles.contains::<HomeAssistant>(),
        }],
        depends_on: DISPATCH_TARGETS,
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                crate::actors::system::home_assistant_ingest::spawn::spawn_home_assistant_ingest(
                    &root,
                    shared_actor_state,
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
        depends_on: &[],
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
        depends_on: &[],
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
        requires: &[
            Requirement::Setting {
                label: "integrations.trmnl.state",
                present: |settings| settings.integrations.trmnl.state.is_enabled(),
            },
            Requirement::Setting {
                label: "integrations.trmnl.api_key",
                present: |settings| settings.integrations.trmnl.api_key.is_some(),
            },
        ],
        depends_on: &[BatteryActor::NAME],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let api_key = shared_actor_state
                    .settings
                    .integrations
                    .trmnl
                    .api_key
                    .clone()
                    .expect("integrations.trmnl.api_key was required");

                let trmnl = Trmnl::new(
                    api_key,
                    shared_actor_state
                        .settings
                        .integrations
                        .trmnl
                        .base_url
                        .clone(),
                    shared_actor_state
                        .settings
                        .http
                        .clients
                        .timeout_for(crate::settings::HttpClientKind::Trmnl),
                );

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
        name: FuelWatchActor::NAME,
        autostart: true,
        optional: false,
        requires: &[Requirement::Handle {
            label: "fuelwatch",
            present: |handles| handles.contains::<FuelWatch>(),
        }],
        depends_on: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let fuelwatch = shared_actor_state.handles.expect::<FuelWatch>().clone();

                let settings = shared_actor_state.settings.integrations.fuelwatch.clone();

                root.spawn_linked(
                    Some(FuelWatchActor::NAME.to_owned()),
                    FuelWatchActor {
                        shared_actor_state,
                        fuelwatch,
                    },
                    settings,
                )
                .await?;

                Ok(Spawned::Started)
            })
        },
    },
    ActorSpec {
        name: WillyWeatherActor::NAME,
        autostart: true,
        optional: false,
        requires: &[
            Requirement::Setting {
                label: "integrations.willyweather.state",
                present: |settings| settings.integrations.willyweather.state.is_enabled(),
            },
            Requirement::Handle {
                label: "willyweather",
                present: |handles| handles.contains::<WillyWeather>(),
            },
        ],
        depends_on: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let willyweather = shared_actor_state.handles.expect::<WillyWeather>().clone();
                let settings = shared_actor_state
                    .settings
                    .integrations
                    .willyweather
                    .clone();

                root.spawn_linked(
                    Some(WillyWeatherActor::NAME.to_owned()),
                    WillyWeatherActor {
                        shared_actor_state,
                        willyweather,
                    },
                    settings,
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
        requires: &[Requirement::Setting {
            label: "integrations.woolworths.state",
            present: |settings| settings.integrations.woolworths.state.is_enabled(),
        }],
        depends_on: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let woolworths = Woolworths::new(
                    shared_actor_state.db.clone(),
                    shared_actor_state
                        .settings
                        .http
                        .clients
                        .timeout_for(crate::settings::HttpClientKind::Woolworths),
                );

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
        depends_on: &[],
        spawn: |root, shared_actor_state| {
            Box::pin(async move {
                let goodwe = shared_actor_state.handles.expect::<GoodWeSemsAPI>().clone();

                let weather = WeatherAPI::new(
                    shared_actor_state
                        .settings
                        .http
                        .clients
                        .timeout_for(crate::settings::HttpClientKind::Bom),
                )?;

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
        depends_on: DISPATCH_TARGETS,
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
    use crate::decoding::DeviceRoleName;
    use std::collections::HashSet;
    use strum::IntoEnumIterator;

    #[test]
    fn every_role_with_a_handler_has_exactly_one_device_actor() {
        let handled = [
            LightHandler::ROLE,
            DoorSensorHandler::ROLE,
            PresenceSensorHandler::ROLE,
            EnvironmentSensorHandler::ROLE,
            PlantSensorHandler::ROLE,
            SmartSwitchHandler::ROLE,
            ControlSwitchHandler::ROLE,
            MediaPlayerHandler::ROLE,
            RobotVacuumHandler::ROLE,
            GarageDoorHandler::ROLE,
        ];

        let expected: HashSet<DeviceRoleName> = DeviceRoleName::iter()
            .filter(|role| role.has_handler())
            .collect();

        assert_eq!(handled.len(), expected.len());
        assert_eq!(handled.into_iter().collect::<HashSet<_>>(), expected);
    }

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
                VacationActor::NAME,
                ReconcilerWorker::NAME,
                ReconcilerSweeper::NAME,
                EsphomeNativeApiIngest::NAME,
                TuyaIngest::NAME,
                HomeAssistantIngest::NAME,
                JellyfinActor::NAME,
                TransperthActor::NAME,
                TrmnlActor::NAME,
                FuelWatchActor::NAME,
                WillyWeatherActor::NAME,
                WoolworthsActor::NAME,
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

    fn spec(name: &'static str, depends_on: &'static [&'static str]) -> ActorSpec {
        ActorSpec {
            name,
            autostart: true,
            optional: false,
            requires: &[],
            depends_on,
            spawn: |_, _| Box::pin(async { Ok(Spawned::Skipped) }),
        }
    }

    #[test]
    fn the_manifest_has_no_dependency_cycle() {
        if let Err(e) = startup_order(ACTORS) {
            panic!("manifest dependencies are invalid: {e}");
        }
    }

    #[test]
    fn every_dependency_starts_before_its_dependent() {
        let order: Vec<&str> = startup_order(ACTORS)
            .expect("a valid manifest")
            .into_iter()
            .map(|spec| spec.name)
            .collect();

        let position = |name: &str| order.iter().position(|started| *started == name);

        for spec in ACTORS {
            for dependency in spec.depends_on {
                assert!(
                    position(dependency) < position(spec.name),
                    "{dependency} should start before {}",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn startup_order_includes_every_actor_once() {
        let order = startup_order(ACTORS).expect("a valid manifest");

        let names: HashSet<&str> = order.iter().map(|spec| spec.name).collect();

        assert_eq!(order.len(), ACTORS.len());
        assert_eq!(names.len(), ACTORS.len());
    }

    #[test]
    fn startup_order_keeps_manifest_order_when_unconstrained() {
        let specs = [spec("b", &["c"]), spec("a", &[]), spec("c", &[])];

        let order: Vec<&str> = startup_order(&specs)
            .expect("no cycle")
            .into_iter()
            .map(|spec| spec.name)
            .collect();

        assert_eq!(order, vec!["a", "c", "b"]);
    }

    #[test]
    fn startup_order_rejects_a_cycle() {
        let specs = [spec("a", &["b"]), spec("b", &["a"])];

        assert!(startup_order(&specs).is_err());
    }

    #[test]
    fn startup_order_rejects_an_unknown_dependency() {
        let specs = [spec("a", &["missing"])];

        assert!(startup_order(&specs).is_err());
    }
}
