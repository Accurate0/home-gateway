use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::settings::DeviceAliases;

use super::raw_device::RawDevice;

static DEVICE_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)+-[0-9]+$").unwrap());

pub fn is_device_id(reference: &str) -> bool {
    DEVICE_ID.is_match(reference)
}

#[derive(Default)]
pub(super) struct DeviceNames {
    claimed: HashMap<String, String>,
    addresses: HashSet<String>,
    aliases: DeviceAliases,
}

impl DeviceNames {
    pub(super) fn check(raw: &[RawDevice]) -> Result<DeviceAliases, String> {
        let mut names = DeviceNames::default();

        for device in raw {
            names.claim_id(device)?;
        }

        for device in raw {
            names.claim_address(device)?;
        }

        for device in raw {
            for alias in &device.aliases {
                names.claim_alias(&device.id, alias)?;
            }
        }

        Ok(names.aliases)
    }

    fn claim_id(&mut self, device: &RawDevice) -> Result<(), String> {
        let id = &device.id;

        if !is_device_id(id) {
            return Err(format!(
                "device {id}: ids must be `<kind>-<product>-<n>`, e.g. `light-tradfri-1`"
            ));
        }

        self.claim(id, id, "id")
    }

    fn claim_address(&mut self, device: &RawDevice) -> Result<(), String> {
        let address = device.transport.address();

        if let Some(owner) = self.claimed.get(address) {
            return Err(format!(
                "device {}: address `{address}` is the id of device {owner}",
                device.id
            ));
        }

        self.addresses.insert(address.to_owned());

        Ok(())
    }

    fn claim_alias(&mut self, id: &str, alias: &str) -> Result<(), String> {
        if is_device_id(alias) {
            return Err(format!(
                "device {id}: alias `{alias}` looks like a device id; aliases must not end in `-<n>`"
            ));
        }

        if self.addresses.contains(alias) {
            return Err(format!("device {id}: alias `{alias}` is a device address"));
        }

        self.claim(id, alias, "alias")?;
        self.aliases.insert(alias.to_owned(), id.to_owned());

        Ok(())
    }

    fn claim(&mut self, id: &str, name: &str, kind: &str) -> Result<(), String> {
        if let Some(owner) = self.claimed.insert(name.to_owned(), id.to_owned()) {
            return Err(format!(
                "device {id}: {kind} `{name}` is already used by device {owner}"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn device(id: &str, address: &str, aliases: &[&str]) -> RawDevice {
        serde_yaml::from_str(&format!(
            "id: {id}\naliases: [{}]\nstate: enabled\ntransport: {{ type: mqtt, address: \"{address}\" }}\nroles: []\n",
            aliases.join(", ")
        ))
        .unwrap()
    }

    fn check(devices: &[RawDevice]) -> Result<DeviceAliases, String> {
        DeviceNames::check(devices)
    }

    #[test]
    fn ids_follow_the_kind_product_number_scheme() {
        for id in ["light-tradfri-1", "light-hue-e27-1", "env-wsdcgq11lm-12"] {
            assert!(is_device_id(id), "{id}");
        }

        for id in [
            "floor-lamp",
            "light-tradfri",
            "tradfri-1",
            "Light-Tradfri-1",
            "light_tradfri_1",
        ] {
            assert!(!is_device_id(id), "{id}");
        }
    }

    #[test]
    fn aliases_map_to_their_device_id() {
        let aliases = check(&[device("light-tradfri-1", "0x1", &["floor-lamp", "lamp"])]).unwrap();

        assert_eq!(
            aliases.get("floor-lamp").map(String::as_str),
            Some("light-tradfri-1")
        );
        assert_eq!(
            aliases.get("lamp").map(String::as_str),
            Some("light-tradfri-1")
        );
    }

    #[test]
    fn an_id_outside_the_scheme_is_rejected() {
        let err = check(&[device("floor-lamp", "0x1", &[])]).unwrap_err();

        assert!(err.contains("`<kind>-<product>-<n>`"), "{err}");
    }

    #[test]
    fn an_alias_used_twice_is_rejected() {
        let err = check(&[
            device("light-tradfri-1", "0x1", &["lamp"]),
            device("light-tradfri-2", "0x2", &["lamp"]),
        ])
        .unwrap_err();

        assert!(
            err.contains("alias `lamp` is already used by device light-tradfri-1"),
            "{err}"
        );
    }

    #[test]
    fn an_alias_matching_the_id_scheme_is_rejected() {
        let err = check(&[device("light-tradfri-1", "0x1", &["light-lamp-2"])]).unwrap_err();

        assert!(err.contains("looks like a device id"), "{err}");
    }

    #[test]
    fn an_alias_equal_to_an_address_is_rejected() {
        let err = check(&[
            device("light-tradfri-1", "0x1", &["rockrobo"]),
            device("vacuum-rockrobo-1", "rockrobo", &[]),
        ])
        .unwrap_err();

        assert!(
            err.contains("alias `rockrobo` is a device address"),
            "{err}"
        );
    }

    #[test]
    fn a_duplicate_id_is_rejected() {
        let err = check(&[
            device("light-tradfri-1", "0x1", &[]),
            device("light-tradfri-1", "0x2", &[]),
        ])
        .unwrap_err();

        assert!(
            err.contains("id `light-tradfri-1` is already used"),
            "{err}"
        );
    }

    #[test]
    fn a_disabled_device_still_claims_its_aliases() {
        let mut disabled = device("light-tradfri-2", "0x2", &["lamp"]);
        disabled.state = crate::settings::enabled_state::EnabledState::Disabled;

        let err = check(&[disabled, device("light-tradfri-1", "0x1", &["lamp"])]).unwrap_err();

        assert!(err.contains("alias `lamp` is already used"), "{err}");
    }
}
