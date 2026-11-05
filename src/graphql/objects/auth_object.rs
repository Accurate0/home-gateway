use async_graphql::SimpleObject;

#[derive(SimpleObject)]
#[graphql(name = "HomeAuthObject")]
pub struct AuthObject {
    pub id: Option<String>,
    pub name: Option<String>,
    pub scopes: Vec<String>,
}
