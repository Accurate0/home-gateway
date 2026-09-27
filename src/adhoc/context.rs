use sqlx::{Pool, Postgres, Transaction};

use crate::device_registry::DeviceRegistry;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::s3::S3;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;
use crate::state::AppState;

pub struct AdhocTaskContext<'a> {
    pub tx: &'a mut Transaction<'static, Postgres>,
    pub db: &'a Pool<Postgres>,
    pub devices: &'a DeviceRegistry,
    pub repos: &'a RepoRegistry,
    pub settings: &'a SettingsContainer,
    pub s3: &'a S3,
    pub home_assistant: Option<&'a HomeAssistant>,
}

impl<'a> AdhocTaskContext<'a> {
    pub fn new(state: &'a AppState, tx: &'a mut Transaction<'static, Postgres>) -> Self {
        Self {
            tx,
            db: &state.db,
            devices: &state.devices,
            repos: &state.repos,
            settings: &state.settings,
            s3: state.handles.expect::<S3>(),
            home_assistant: state.handles.get::<HomeAssistant>(),
        }
    }

    pub async fn commit_batch(&mut self) -> Result<(), sqlx::Error> {
        let next = self.db.begin().await?;
        let done = std::mem::replace(self.tx, next);

        done.commit().await
    }
}
