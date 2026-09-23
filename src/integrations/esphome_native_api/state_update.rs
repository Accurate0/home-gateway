use serde_json::Value;

use super::entity_domain::EntityDomain;

pub struct StateUpdate {
    pub address: String,
    pub domain: EntityDomain,
    pub object_id: String,
    pub payload: Value,
}
