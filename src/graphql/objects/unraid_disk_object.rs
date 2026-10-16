use async_graphql::Object;
use chrono::{DateTime, Utc};

use super::unraid_disk_point::UnraidDiskPoint;
use crate::integrations::unraid::types::DiskSnapshot;
use crate::repo::RepoRegistry;

pub struct UnraidDiskObject(pub DiskSnapshot);

#[Object(name = "UnraidDisk")]
impl UnraidDiskObject {
    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn kind(&self) -> String {
        self.0.kind.to_string()
    }

    async fn status(&self) -> Option<String> {
        self.0.status.map(|status| status.to_string())
    }

    async fn temperature(&self) -> Option<i32> {
        self.0.temperature
    }

    async fn usage_percent(&self) -> Option<f64> {
        self.0.usage_percent()
    }

    async fn size_kb(&self) -> Option<i64> {
        self.0.size_kb
    }

    async fn fs_size_kb(&self) -> Option<i64> {
        self.0.fs_size_kb
    }

    async fn fs_free_kb(&self) -> Option<i64> {
        self.0.fs_free_kb
    }

    async fn fs_used_kb(&self) -> Option<i64> {
        self.0.fs_used_kb
    }

    async fn errors(&self) -> Option<i64> {
        self.0.errors
    }

    async fn spinning(&self) -> Option<bool> {
        self.0.spinning
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<UnraidDiskPoint>> {
        let rows = ctx
            .data::<RepoRegistry>()?
            .unraid()
            .disk_history(&self.0.name, since)
            .await?;

        Ok(rows.into_iter().map(UnraidDiskPoint::from).collect())
    }
}
