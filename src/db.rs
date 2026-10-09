use crate::settings::DatabaseSettings;
use sqlx::{
    ConnectOptions, Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub async fn connect(
    database_url: &str,
    settings: &DatabaseSettings,
) -> anyhow::Result<Pool<Postgres>> {
    let options = PgConnectOptions::from_url(&database_url.parse()?)?
        .log_statements(log::LevelFilter::Debug)
        .log_slow_statements(log::LevelFilter::Warn, settings.slow_statement_threshold());

    let pool = PgPoolOptions::new()
        .min_connections(settings.min_connections)
        .max_connections(settings.max_connections)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;

    Ok(())
}
