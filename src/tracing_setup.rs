use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_target(
                    "opentelemetry_sdk",
                    if cfg!(debug_assertions) {
                        LevelFilter::OFF
                    } else {
                        LevelFilter::ERROR
                    },
                )
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    Ok(())
}
