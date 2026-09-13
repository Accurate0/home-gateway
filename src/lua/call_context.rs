use std::fmt::Display;
use std::future::Future;
use std::time::{Duration, Instant};

use mlua::ExternalResult;
use uuid::Uuid;

use crate::auth::scope::{Action, Resource};
use crate::state::AppState;

use super::{LuaAuthority, LuaFunction};

#[derive(Clone)]
pub struct LuaCallContext {
    pub state: AppState,
    pub event_id: Uuid,
    pub depth: u8,
    pub dry_run: bool,
    pub origin: String,
    pub authority: LuaAuthority,
}

impl LuaCallContext {
    pub fn new(state: AppState, event_id: Uuid, origin: impl Into<String>) -> Self {
        LuaCallContext {
            state,
            event_id,
            depth: 0,
            dry_run: false,
            origin: origin.into(),
            authority: LuaAuthority::Trusted,
        }
    }

    pub fn with_authority(mut self, authority: LuaAuthority) -> Self {
        self.authority = authority;

        self
    }

    pub fn allows(&self, resource: Resource, action: Action) -> bool {
        self.authority.allows(resource, action)
    }

    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth;

        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;

        self
    }

    pub fn expose(
        &self,
        table: &mlua::Table,
        function: &LuaFunction,
        build: impl FnOnce() -> mlua::Result<mlua::Function>,
    ) -> mlua::Result<()> {
        if let Some(scope) = function.scope
            && !self.allows(scope.resource, scope.action)
        {
            tracing::debug!(
                "withholding lua `{}` from {}: needs {scope}",
                function.name,
                self.authority.describe()
            );

            return Ok(());
        }

        table.set(function.name, build()?)
    }

    pub fn timeout(&self) -> Duration {
        self.state.settings.workflow.condition_timeout()
    }

    pub async fn command<F, Fut, T, E>(
        &self,
        name: &'static str,
        detail: impl Display,
        run: F,
    ) -> mlua::Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        if self.dry_run {
            tracing::info!(
                "[{}] dry-run would call lua {name}: {detail}",
                self.event_id
            );
            crate::metrics::record_step(name, true, Duration::ZERO);

            return Ok(());
        }

        tracing::debug!("[{}] lua {name}: {detail}", self.event_id);

        let start = Instant::now();
        let result = run().await;
        crate::metrics::record_step(name, result.is_ok(), start.elapsed());

        result.map(|_| ()).into_lua_err()
    }

    pub async fn query<F, Fut, T, E>(&self, name: &'static str, run: F) -> mlua::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        tracing::debug!("[{}] lua {name}", self.event_id);

        run().await.into_lua_err()
    }
}
