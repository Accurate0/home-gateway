use pgmq::PGMQueueExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::{fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug)]
struct DelayQueueInner<T>
where
    T: Send + Debug,
{
    _phantom: PhantomData<T>,
    pub(crate) queue: PGMQueueExt,
    pub(crate) queue_name: String,
}

#[derive(Debug)]
pub struct DelayQueue<T>
where
    T: Send + Debug,
{
    inner: Arc<DelayQueueInner<T>>,
}

#[derive(Error, Debug)]
pub enum DelayQueueError {
    #[error("serde parse error has occurred: `{0}`")]
    Serde(#[from] serde_json::Error),
    #[error("sqlx error has occurred: `{0}`")]
    Sqlx(#[from] sqlx::Error),
    #[error("pgmq error has occurred: `{0}`")]
    Pgmq(#[from] pgmq::PgmqError),
}

impl<T> DelayQueue<T>
where
    T: Send + Debug + Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub async fn new(pool: Pool<Postgres>, queue_name: String) -> Result<Self, DelayQueueError> {
        let queue: PGMQueueExt = PGMQueueExt::new_with_pool(pool).await;
        queue.create(&queue_name).await?;

        Ok(Self {
            inner: DelayQueueInner {
                queue,
                queue_name,
                _phantom: Default::default(),
            }
            .into(),
        })
    }

    #[instrument(skip(self))]
    pub async fn push(&self, item: T, delay: Duration) -> Result<(), DelayQueueError> {
        self.inner
            .queue
            .send_delay(&self.inner.queue_name, &item, delay.as_secs() as u32)
            .await?;

        Ok(())
    }

    pub async fn read(
        &self,
        visibility_timeout: Duration,
    ) -> Result<Option<pgmq::Message<T>>, DelayQueueError> {
        let message = self
            .inner
            .queue
            .read_batch_with_poll(
                &self.inner.queue_name,
                visibility_timeout.as_secs() as i32,
                1,
                None,
                None,
            )
            .await?
            .and_then(|mut r| r.pop());

        Ok(message)
    }

    #[instrument(skip(self))]
    pub async fn archive(&self, message_id: i64) -> Result<bool, DelayQueueError> {
        Ok(self
            .inner
            .queue
            .archive(&self.inner.queue_name, message_id)
            .await?)
    }
}

impl<T> Clone for DelayQueue<T>
where
    T: Send + Debug,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
