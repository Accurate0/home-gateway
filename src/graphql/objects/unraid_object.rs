use async_graphql::Object;
use chrono::{DateTime, Utc};

use super::unraid_array_object::UnraidArrayObject;
use super::unraid_disk_object::UnraidDiskObject;
use crate::repo::unraid::StoredUnraidSnapshot;

pub struct UnraidObject(pub StoredUnraidSnapshot);

#[Object(name = "Unraid")]
impl UnraidObject {
    async fn recorded_at(&self) -> DateTime<Utc> {
        self.0.recorded_at
    }

    async fn array(&self) -> UnraidArrayObject {
        UnraidArrayObject(self.0.snapshot.array.clone())
    }

    async fn disks(&self) -> Vec<UnraidDiskObject> {
        self.0
            .snapshot
            .disks
            .iter()
            .cloned()
            .map(UnraidDiskObject)
            .collect()
    }

    async fn disk(&self, name: String) -> Option<UnraidDiskObject> {
        self.0
            .snapshot
            .disks
            .iter()
            .find(|disk| disk.name == name)
            .cloned()
            .map(UnraidDiskObject)
    }
}
