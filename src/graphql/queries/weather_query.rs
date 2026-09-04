use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::weather_object::WeatherObject;
use async_graphql::{InputObject, Object};

#[derive(InputObject)]
pub struct WeatherInput {
    pub location: String,
}

#[derive(Default)]
pub struct WeatherQuery;

#[Object]
impl WeatherQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Weather, Action::Read)))]
    async fn weather(
        &self,
        _ctx: &async_graphql::Context<'_>,
        input: WeatherInput,
    ) -> async_graphql::Result<WeatherObject> {
        Ok(WeatherObject {
            location: input.location,
        })
    }
}
