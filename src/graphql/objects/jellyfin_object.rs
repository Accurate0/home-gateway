use async_graphql::Object;

use crate::graphql::objects::jellyfin_session_object::JellyfinSessionObject;
use crate::repo::RepoRegistry;

pub struct JellyfinObject;

#[Object]
impl JellyfinObject {
    async fn now_playing(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<JellyfinSessionObject>> {
        let repos = ctx.data::<RepoRegistry>()?;

        let sessions = repos.jellyfin().sessions().await?;

        Ok(sessions
            .into_iter()
            .map(JellyfinSessionObject::from)
            .collect())
    }
}
