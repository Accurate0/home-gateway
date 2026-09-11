use sqlx::{Pool, Postgres};

use crate::db;
use crate::device_registry::DeviceRegistry;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;

pub struct Storage {
    pub settings: SettingsContainer,
    pub devices: DeviceRegistry,
    pub pool: Pool<Postgres>,
    pub repos: RepoRegistry,
}

pub async fn init() -> anyhow::Result<Storage> {
    let (settings, devices) = SettingsContainer::new()?;

    let pool = db::connect(&settings.database_url, &settings.database).await?;
    db::migrate(&pool).await?;

    let repos = RepoRegistry::new(pool.clone());

    Ok(Storage {
        settings,
        devices,
        pool,
        repos,
    })
}
