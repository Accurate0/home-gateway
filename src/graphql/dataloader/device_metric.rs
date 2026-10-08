use crate::repo::MetricRepo;
use crate::repo::metric::LatestMetricRow;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceMetricKey {
    pub address: String,
    pub metric: String,
}

pub struct DeviceMetricDataLoader {
    pub repo: MetricRepo,
}

impl Loader<DeviceMetricKey> for DeviceMetricDataLoader {
    type Value = Arc<LatestMetricRow>;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[DeviceMetricKey],
    ) -> Result<HashMap<DeviceMetricKey, Self::Value>, Self::Error> {
        let addresses: Vec<String> = keys.iter().map(|key| key.address.clone()).collect();
        let metrics: Vec<String> = keys.iter().map(|key| key.metric.clone()).collect();

        let rows = self.repo.latest_many(&addresses, &metrics).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let key = DeviceMetricKey {
                    address: row.address.clone(),
                    metric: row.metric.clone(),
                };

                (key, Arc::new(row))
            })
            .collect())
    }
}
