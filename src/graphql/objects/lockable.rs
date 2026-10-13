use async_graphql::{Context, Interface, Result, dataloader::DataLoader};

use crate::graphql::dataloader::device_lock::DeviceLockDataLoader;
use crate::graphql::objects::device_lock_object::DeviceLockObject;
use crate::graphql::objects::entity_object::AirPurifierEntity;

pub trait Lockable: Sync {
    fn device_id(&self) -> &str;

    fn lock_state(
        &self,
        ctx: &Context<'_>,
    ) -> impl Future<Output = Result<Option<DeviceLockObject>>> + Send {
        async move {
            let loader = ctx.data::<DataLoader<DeviceLockDataLoader>>()?;
            let lock = loader.load_one(self.device_id().to_owned()).await?;

            Ok(lock.map(DeviceLockObject::from))
        }
    }
}

#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "&str"),
    field(name = "lock", ty = "Option<DeviceLockObject>")
)]
pub enum LockableEntity {
    AirPurifier(AirPurifierEntity),
}
