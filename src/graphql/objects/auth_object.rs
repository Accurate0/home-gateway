use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct AuthObject {
    pub id: Option<String>,
    pub scopes: Vec<String>,
}
