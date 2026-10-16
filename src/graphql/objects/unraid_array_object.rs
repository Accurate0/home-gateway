use async_graphql::Object;
use chrono::{DateTime, Utc};

use super::unraid_array_point::UnraidArrayPoint;
use crate::integrations::unraid::types::ArraySnapshot;
use crate::repo::RepoRegistry;

pub struct UnraidArrayObject(pub ArraySnapshot);

#[Object(name = "UnraidArray")]
impl UnraidArrayObject {
    async fn state(&self) -> String {
        self.0.state.to_string()
    }

    async fn usage_percent(&self) -> Option<f64> {
        self.0.usage_percent()
    }

    async fn kilobytes_free(&self) -> i64 {
        self.0.kilobytes_free
    }

    async fn kilobytes_used(&self) -> i64 {
        self.0.kilobytes_used
    }

    async fn kilobytes_total(&self) -> i64 {
        self.0.kilobytes_total
    }

    async fn parity_status(&self) -> String {
        self.0.parity_status.to_string()
    }

    async fn parity_progress(&self) -> Option<i64> {
        self.0.parity_progress
    }

    async fn parity_errors(&self) -> Option<i64> {
        self.0.parity_errors
    }

    async fn parity_running(&self) -> bool {
        self.0.parity_running
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        since: DateTime<Utc>,
    ) -> async_graphql::Result<Vec<UnraidArrayPoint>> {
        let rows = ctx
            .data::<RepoRegistry>()?
            .unraid()
            .array_history(since)
            .await?;

        Ok(rows.into_iter().map(UnraidArrayPoint::from).collect())
    }
}
