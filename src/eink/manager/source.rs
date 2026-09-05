use super::resolve::ResolvedDisplay;
use crate::error::AppError;
use crate::integrations::reddit::Reddit;
use crate::integrations::s3::S3;
use crate::settings::EinkMode;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceImage {
    pub image_key: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    Ready(SourceImage),
    NeedsRefresh,
    Skip,
}

#[async_trait::async_trait]
pub trait ScreenshotBackend: Send + Sync {
    async fn screenshot(
        &self,
        url: &str,
        dims: (u32, u32),
        settle: Duration,
    ) -> Result<Option<Vec<u8>>, AppError>;
}

pub struct SourceContext<'a> {
    pub display: &'a ResolvedDisplay,
    pub db: &'a sqlx::Pool<sqlx::Postgres>,
    pub eink: &'a crate::repo::EinkRepo,
    pub s3: &'a S3,
    pub reddit: &'a Reddit,
    pub screenshots: &'a dyn ScreenshotBackend,
}

#[async_trait::async_trait]
pub trait ImageSource: Send + Sync {
    fn name(&self) -> &'static str;

    fn mode(&self) -> EinkMode;

    async fn acquire(&self, ctx: &SourceContext<'_>) -> Result<Option<SourceImage>, AppError>;

    async fn prepare(&self, ctx: &SourceContext<'_>) -> Result<Prepared, AppError> {
        let prepared = match self.acquire(ctx).await? {
            Some(image) => Prepared::Ready(image),
            None => Prepared::Skip,
        };

        Ok(prepared)
    }
}

pub struct SourceRegistry {
    sources: Vec<Box<dyn ImageSource>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn register(mut self, source: impl ImageSource + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    pub fn select(&self, mode: EinkMode) -> Option<&dyn ImageSource> {
        self.sources
            .iter()
            .find(|source| source.mode() == mode)
            .map(|source| source.as_ref())
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::manager::sources::{AlbumSource, DashboardSource, RedditSource};

    fn registry() -> SourceRegistry {
        SourceRegistry::new()
            .register(DashboardSource::default())
            .register(AlbumSource)
            .register(RedditSource)
    }

    #[test]
    fn every_mode_has_a_registered_source() {
        let registry = registry();

        for mode in [EinkMode::Dashboard, EinkMode::Album, EinkMode::Reddit] {
            assert_eq!(
                registry.select(mode).map(|source| source.mode()),
                Some(mode)
            );
        }
    }

    #[test]
    fn an_empty_registry_selects_nothing() {
        assert!(SourceRegistry::new().select(EinkMode::Dashboard).is_none());
    }

    #[test]
    fn the_first_registration_for_a_mode_wins() {
        let registry = SourceRegistry::new()
            .register(AlbumSource)
            .register(AlbumSource);

        assert_eq!(
            registry.select(EinkMode::Album).map(|source| source.name()),
            Some("album")
        );
    }
}
