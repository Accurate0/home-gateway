use home_gateway::device_registry::DeviceRegistry;
use home_gateway::integrations::mqtt::MqttProtocol;
use home_gateway::settings::SettingsContainer;
use pretty_assertions::assert_eq;
use std::path::Path;

const FIXTURE_CONFIG: &str = "tests/fixtures/config";

fn load() -> (home_gateway::settings::Settings, DeviceRegistry) {
    SettingsContainer::load_from_dir(Path::new(FIXTURE_CONFIG))
        .expect("failed to load the fixture config")
}

#[test]
fn fixture_config_resolves() {
    let (settings, devices) = load();

    assert_eq!(settings.version, "1");
    assert!(
        settings.workflows.contains_key("test-lamp-on"),
        "expected the fixture workflows to resolve, got {:?}",
        settings.workflows.keys().collect::<Vec<_>>()
    );

    for address in [
        "0x0000000000000001",
        "0x0000000000000002",
        "0x0000000000000003",
        "0x0000000000000004",
    ] {
        assert!(
            devices.mqtt_device(MqttProtocol::Zigbee, address).is_some(),
            "expected {address} to be registered by address"
        );
    }
}

#[test]
fn device_ids_alias_to_addresses() {
    let (_, devices) = load();

    assert_eq!(
        devices.address_or_self("test-lamp"),
        "0x0000000000000003",
        "the top-level device id should alias to its address"
    );
    assert_eq!(
        devices.address_or_self("0x0000000000000003"),
        "0x0000000000000003",
        "an address should resolve to itself"
    );
}

#[test]
fn an_esphome_native_api_node_registers_without_mqtt_topics() {
    let (_, devices) = load();

    let nodes: Vec<&String> = devices.esphome_native_api_nodes().collect();
    assert_eq!(nodes, vec!["test-speaker.iot"]);

    assert!(
        devices.media_player("test-speaker.iot").is_some(),
        "the node should carry its media player role"
    );
    assert!(
        devices.mqtt_topics_for("test-speaker.iot").is_empty(),
        "a native api node subscribes to no mqtt topics"
    );
}

#[test]
fn esphome_state_topics_are_subscribed() {
    let (_, devices) = load();

    let topics = devices.mqtt_subscriptions();

    assert!(
        topics
            .iter()
            .any(|topic| topic.contains("test-node") && topic.contains("test_node_temperature")),
        "expected a precomputed esphome temperature topic, got {topics:?}"
    );
    assert!(
        topics
            .iter()
            .any(|topic| topic.contains("test_node_humidity")),
        "expected a precomputed esphome humidity topic, got {topics:?}"
    );
}

#[test]
fn unknown_mqtt_model_is_rejected() {
    let error = build_with_devices(
        r#"
- id: broken
  state: enabled
  room: test
  transport:
    type: mqtt
    address: "0x000000000000dead"
  model: not_a_real_model
  roles:
    - type: door
      config: { name: Broken, id: broken, state: unarmed, notify: [android_app] }
"#,
    );

    assert!(
        error.contains("not_a_real_model"),
        "expected the unknown slug to be named in the error, got: {error}"
    );
}

#[test]
fn mqtt_device_without_a_model_names_the_transport() {
    let error = build_with_devices(
        r#"
- id: broken
  state: enabled
  room: test
  transport:
    type: mqtt
    address: broken-node
  roles:
    - type: environment
      config: { id: broken, name: Broken }
"#,
    );

    assert!(
        error.contains("mqtt transport requires a `model:`"),
        "expected a model-related error, got: {error}"
    );
}

/// Rebuilds the fixture config with `devices/test.yaml` swapped out, so the
/// startup validation errors can be asserted without a whole config tree.
fn build_with_devices(devices_yaml: &str) -> String {
    let dir = tempdir();
    std::fs::create_dir_all(dir.join("workflows")).unwrap();
    std::fs::create_dir_all(dir.join("devices")).unwrap();

    std::fs::copy(
        Path::new(FIXTURE_CONFIG).join("base.yaml"),
        dir.join("base.yaml"),
    )
    .unwrap();

    for models in ["lua/mqtt", "lua/esphome_native_api", "lua/home_assistant"] {
        std::fs::create_dir_all(dir.join(models)).unwrap();

        for entry in std::fs::read_dir(Path::new(FIXTURE_CONFIG).join(models)).unwrap() {
            let path = entry.unwrap().path();

            std::fs::copy(&path, dir.join(models).join(path.file_name().unwrap())).unwrap();
        }
    }
    std::fs::write(dir.join("workflows/index.yaml"), "[]\n").unwrap();
    std::fs::write(dir.join("devices/index.yaml"), "- !include test.yaml\n").unwrap();
    std::fs::write(dir.join("devices/test.yaml"), devices_yaml).unwrap();

    let error = SettingsContainer::load_from_dir(&dir)
        .err()
        .unwrap_or_else(|| panic!("expected the config to be rejected"));

    std::fs::remove_dir_all(&dir).ok();

    error.to_string()
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("home-gateway-config-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
