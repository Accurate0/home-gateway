use async_graphql::{Interface, Object, SimpleObject};
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Australia::Perth;

use crate::auth::scope::{Action, Resource, Scope};
use crate::away::{build_plan, coverage, target_at};
use crate::device_registry::DeviceRegistry;
use crate::graphql::guard::ScopeGuard;
use crate::mode::Mode;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;
use crate::timedelta_format::humanize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
pub enum LightTarget {
    On,
    Off,
    LeaveAlone,
}

#[derive(SimpleObject)]
pub struct AwayAction {
    pub at: DateTime<Utc>,
    pub on: bool,
    pub slot: i32,
    pub on_fraction: f64,
}

#[derive(SimpleObject)]
pub struct AwayLight {
    pub address: String,
    pub device_id: Option<String>,
    pub name: String,
    pub coverage: i32,
    pub current_target: LightTarget,
    pub actions: Vec<AwayAction>,
}

pub struct AwayMode {
    pub mode: Mode,
    pub active: bool,
}

#[Object]
impl AwayMode {
    async fn mode(&self) -> Mode {
        self.mode
    }

    async fn active(&self) -> bool {
        self.active
    }

    async fn enabled(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        Ok(ctx.data::<SettingsContainer>()?.away.enabled)
    }

    async fn window(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<String> {
        Ok(humanize(ctx.data::<SettingsContainer>()?.away.window))
    }

    async fn jitter(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<String> {
        Ok(humanize(ctx.data::<SettingsContainer>()?.away.jitter))
    }

    async fn min_observations(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<i64> {
        Ok(ctx.data::<SettingsContainer>()?.away.min_observations)
    }

    /// The replay plan for every configured light: what it should be doing now,
    /// and every switch still to come in the next 24 hours.
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Read)))]
    async fn lights(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<AwayLight>> {
        let settings = &ctx.data::<SettingsContainer>()?.away;
        let devices = ctx.data::<DeviceRegistry>()?;
        let repos = ctx.data::<RepoRegistry>()?;

        let buckets = repos.light().profile_all(settings.window).await?;

        let now = Utc::now();
        let today = now.with_timezone(&Perth).date_naive();
        let tomorrow = today.succ_opt().unwrap_or(today);

        let mut plan = build_plan(&buckets, today, settings);
        plan.extend(build_plan(&buckets, tomorrow, settings));

        let horizon = now + Duration::hours(24);
        let mut lights = Vec::new();

        for (address, name) in devices.lights() {
            let actions = plan
                .iter()
                .filter(|action| {
                    &action.address == address && action.at >= now && action.at <= horizon
                })
                .map(|action| AwayAction {
                    at: action.at,
                    on: action.on,
                    slot: i32::from(action.slot),
                    on_fraction: action.on_fraction,
                })
                .collect();

            let current_target = match target_at(&plan, address, now) {
                Some(true) => LightTarget::On,
                Some(false) => LightTarget::Off,
                None => LightTarget::LeaveAlone,
            };

            lights.push(AwayLight {
                address: address.clone(),
                device_id: devices.id_for_address(address).map(str::to_owned),
                name: name.clone(),
                coverage: coverage(&buckets, address, today, settings.min_observations) as i32,
                current_target,
                actions,
            });
        }

        lights.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(lights)
    }
}

pub struct PlainMode {
    pub mode: Mode,
    pub active: bool,
}

#[Object]
impl PlainMode {
    async fn mode(&self) -> Mode {
        self.mode
    }

    async fn active(&self) -> bool {
        self.active
    }
}

#[derive(Interface)]
#[graphql(
    name = "ModeState",
    field(name = "mode", ty = "Mode"),
    field(name = "active", ty = "bool")
)]
pub enum ModeObject {
    Away(AwayMode),
    Plain(PlainMode),
}

impl ModeObject {
    pub fn new(mode: Mode, active: bool) -> Self {
        match mode {
            Mode::Away => ModeObject::Away(AwayMode { mode, active }),
            _ => ModeObject::Plain(PlainMode { mode, active }),
        }
    }
}
