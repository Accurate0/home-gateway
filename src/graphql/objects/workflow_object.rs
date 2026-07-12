use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct WorkflowStatus {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub config_enabled: bool,
    pub dry_run: bool,
    pub reusable: bool,
}
