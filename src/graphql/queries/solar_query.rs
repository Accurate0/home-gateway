use crate::graphql::objects::solar_object::SolarObject;
use async_graphql::Object;

#[derive(Default)]
pub struct SolarQuery;

#[Object]
impl SolarQuery {
    async fn solar(&self, _ctx: &async_graphql::Context<'_>) -> async_graphql::Result<SolarObject> {
        Ok(SolarObject {})
    }
}
