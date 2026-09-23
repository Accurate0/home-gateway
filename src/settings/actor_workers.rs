use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::decoding::DeviceRoleName;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ActorWorkerSettings {
    pub mqtt_ingest: usize,
    pub home_assistant_ingest: usize,
    pub esphome_native_api_ingest: usize,
    devices: BTreeMap<DeviceRoleName, usize>,
}

impl ActorWorkerSettings {
    pub fn validate(&self) -> Result<(), String> {
        for role in DeviceRoleName::ALL {
            match (role.has_handler(), self.devices.contains_key(&role)) {
                (true, false) => {
                    return Err(format!("actors.workers.devices is missing `{role}`"));
                }
                (false, true) => {
                    return Err(format!(
                        "actors.workers.devices: the `{role}` role has no device actor"
                    ));
                }
                (true, true) | (false, false) => {}
            }
        }

        Ok(())
    }

    pub fn device(&self, role: DeviceRoleName) -> usize {
        self.devices[&role]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workers(devices: &str) -> ActorWorkerSettings {
        serde_yaml::from_str(&format!(
            "{{ mqtt_ingest: 1, home_assistant_ingest: 1, esphome_native_api_ingest: 1, devices: {devices} }}"
        ))
        .expect("workers")
    }

    const ALL_HANDLERS: &str = "{ door: 1, environment: 1, plant: 1, light: 2, smart_switch: 1, presence: 1, control_switch: 1, robot_vacuum: 1, media_player: 1 }";

    #[test]
    fn every_handled_role_resolves_its_worker_count() {
        let workers = workers(ALL_HANDLERS);

        workers.validate().expect("valid");
        assert_eq!(workers.device(DeviceRoleName::Light), 2);
    }

    #[test]
    fn a_missing_handled_role_is_rejected() {
        let error = workers("{ door: 1 }")
            .validate()
            .expect_err("missing roles");

        assert!(error.contains("is missing `environment`"), "{error}");
    }

    #[test]
    fn a_role_without_a_device_actor_is_rejected() {
        let error = workers(&ALL_HANDLERS.replace(" }", ", battery: 1 }"))
            .validate()
            .expect_err("battery has no actor");

        assert!(
            error.contains("`battery` role has no device actor"),
            "{error}"
        );
    }
}
