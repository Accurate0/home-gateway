use async_graphql::Enum;
use serde::{Deserialize, Serialize};
use sqlx::{
    ConnectOptions, Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::time::Duration;

const SLOW_STATEMENT_THRESHOLD: Duration = Duration::from_secs(6);

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Postgres>> {
    let options = PgConnectOptions::from_url(&database_url.parse()?)?
        .log_slow_statements(log::LevelFilter::Warn, SLOW_STATEMENT_THRESHOLD);

    let pool = PgPoolOptions::new()
        .min_connections(0)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;

    Ok(())
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize, Enum, Eq, Copy,
)]
#[sqlx(type_name = "unifi_state", rename_all = "lowercase")]
pub enum UnifiState {
    Connected,
    Disconnected,
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Serialize, Deserialize, Enum, Eq, Copy,
)]
#[sqlx(type_name = "door_state", rename_all = "lowercase")]
pub enum DoorState {
    Open,
    Closed,
}
