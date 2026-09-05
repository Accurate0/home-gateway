use crate::repo::{
    AdhocRepo, BatteryRepo, DeviceRepo, DoorRepo, EinkRepo, EnergyRepo, EnvironmentRepo,
    HomeAssistantRepo, JellyfinRepo, LightRepo, MediaPlayerRepo, MetricRepo, PushRepo,
    RobotVacuumRepo, SmartSwitchRepo, SolarRepo, SunRepo, UnifiRepo, WatchdogRepo, WoolworthsRepo,
    WorkflowRepo,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

struct Repos {
    adhoc: AdhocRepo,
    battery: BatteryRepo,
    device: DeviceRepo,
    door: DoorRepo,
    eink: EinkRepo,
    energy: EnergyRepo,
    home_assistant: HomeAssistantRepo,
    jellyfin: JellyfinRepo,
    solar: SolarRepo,
    unifi: UnifiRepo,
    woolworths: WoolworthsRepo,
    environment: EnvironmentRepo,
    light: LightRepo,
    media_player: MediaPlayerRepo,
    metric: MetricRepo,
    robot_vacuum: RobotVacuumRepo,
    smart_switch: SmartSwitchRepo,
    push: PushRepo,
    sun: SunRepo,
    watchdog: WatchdogRepo,
    workflow: WorkflowRepo,
}

#[derive(Clone)]
pub struct RepoRegistry {
    inner: Arc<Repos>,
}

impl RepoRegistry {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self {
            inner: Arc::new(Repos {
                adhoc: AdhocRepo::new(db.clone()),
                battery: BatteryRepo::new(db.clone()),
                device: DeviceRepo::new(db.clone()),
                door: DoorRepo::new(db.clone()),
                eink: EinkRepo::new(db.clone()),
                energy: EnergyRepo::new(db.clone()),
                home_assistant: HomeAssistantRepo::new(db.clone()),
                jellyfin: JellyfinRepo::new(db.clone()),
                solar: SolarRepo::new(db.clone()),
                unifi: UnifiRepo::new(db.clone()),
                woolworths: WoolworthsRepo::new(db.clone()),
                environment: EnvironmentRepo::new(db.clone()),
                light: LightRepo::new(db.clone()),
                media_player: MediaPlayerRepo::new(db.clone()),
                metric: MetricRepo::new(db.clone()),
                robot_vacuum: RobotVacuumRepo::new(db.clone()),
                smart_switch: SmartSwitchRepo::new(db.clone()),
                push: PushRepo::new(db.clone()),
                sun: SunRepo::new(db.clone()),
                watchdog: WatchdogRepo::new(db.clone()),
                workflow: WorkflowRepo::new(db),
            }),
        }
    }

    pub fn adhoc(&self) -> &AdhocRepo {
        &self.inner.adhoc
    }

    pub fn battery(&self) -> &BatteryRepo {
        &self.inner.battery
    }

    pub fn device(&self) -> &DeviceRepo {
        &self.inner.device
    }

    pub fn door(&self) -> &DoorRepo {
        &self.inner.door
    }

    pub fn eink(&self) -> &EinkRepo {
        &self.inner.eink
    }

    pub fn energy(&self) -> &EnergyRepo {
        &self.inner.energy
    }

    pub fn home_assistant(&self) -> &HomeAssistantRepo {
        &self.inner.home_assistant
    }

    pub fn jellyfin(&self) -> &JellyfinRepo {
        &self.inner.jellyfin
    }

    pub fn solar(&self) -> &SolarRepo {
        &self.inner.solar
    }

    pub fn unifi(&self) -> &UnifiRepo {
        &self.inner.unifi
    }

    pub fn woolworths(&self) -> &WoolworthsRepo {
        &self.inner.woolworths
    }

    pub fn environment(&self) -> &EnvironmentRepo {
        &self.inner.environment
    }

    pub fn light(&self) -> &LightRepo {
        &self.inner.light
    }

    pub fn media_player(&self) -> &MediaPlayerRepo {
        &self.inner.media_player
    }

    pub fn metric(&self) -> &MetricRepo {
        &self.inner.metric
    }

    pub fn robot_vacuum(&self) -> &RobotVacuumRepo {
        &self.inner.robot_vacuum
    }

    pub fn smart_switch(&self) -> &SmartSwitchRepo {
        &self.inner.smart_switch
    }

    pub fn push(&self) -> &PushRepo {
        &self.inner.push
    }

    pub fn sun(&self) -> &SunRepo {
        &self.inner.sun
    }

    pub fn watchdog(&self) -> &WatchdogRepo {
        &self.inner.watchdog
    }

    pub fn workflow(&self) -> &WorkflowRepo {
        &self.inner.workflow
    }
}
