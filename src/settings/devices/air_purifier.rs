use schemars::JsonSchema;
use serde::Deserialize;

use crate::device_lock::{LockSettings, RawLockSettings};
use crate::repo::air_purifier::AirPurifierField;

#[derive(Debug, Clone)]
pub struct AirPurifierSettings {
    pub name: String,
    pub id: String,
    pub address: String,
    pub lock: LockSettings,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawAirPurifierBlock {
    name: String,
    lock: RawLockSettings<AirPurifierField>,
}

impl RawAirPurifierBlock {
    pub(crate) fn resolve(self, id: &str, address: &str) -> AirPurifierSettings {
        AirPurifierSettings {
            name: self.name,
            id: id.to_owned(),
            address: address.to_owned(),
            lock: self.lock.resolve(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn a_lock_block_is_parsed_and_resolved() {
        let block = serde_yaml::from_str::<RawAirPurifierBlock>(
            "name: Purifier\nlock: { physical: 6h, excluded: [display] }",
        )
        .unwrap();

        let settings = block.resolve("purifier-core300s-1", "purifier.iot");

        assert_eq!(settings.lock.physical, TimeDelta::hours(6));
        assert_eq!(settings.lock.excluded, vec!["display"]);
    }

    #[test]
    fn a_missing_lock_block_is_rejected() {
        assert!(serde_yaml::from_str::<RawAirPurifierBlock>("name: Purifier").is_err());
    }

    #[test]
    fn an_unknown_excluded_field_is_rejected() {
        let block = serde_yaml::from_str::<RawAirPurifierBlock>(
            "name: Purifier\nlock: { physical: 6h, excluded: [colour] }",
        );

        assert!(block.is_err());
    }
}
