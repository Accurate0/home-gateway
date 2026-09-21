# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build                                    # compile (uses offline sqlx query cache in .sqlx/)
cargo test                                      # run all tests
cargo test settings::tests::config_yaml_parses_and_resolves   # single test by path
cargo test <substring>                          # run tests matching a name

docker compose up                               # run the gateway + TimescaleDB (Postgres) locally
fish scripts/install-completions.fish --regenerate   # regenerate completions/home.fish and install fish completions
```

- SQL is checked at compile time by `sqlx` against the cache in `.sqlx/`. After changing any `sqlx::query!`/`query_as!`, regenerate with `cargo sqlx prepare` (needs a live `DATABASE_URL`) or the build will fail offline.
- Migrations live in `migrations/` and require the TimescaleDB extension (`Dockerfile.postgres`); the DB is not plain Postgres.
- The `keys` binary (`src/bin/keys.rs`) is a separate CLI for managing API keys.

## Architecture

This is a home-automation gateway: it ingests device events (Zigbee via zigbee2mqtt, ESPHome, plus HTTP webhooks from UniFi/solar/etc.), runs rule-based **workflows**, and exposes a GraphQL + REST API. It is built on the **`ractor` actor framework** — almost all runtime logic lives in supervised actors, not in request handlers.

### Actor tree
Actors are grouped by role under `src/actors/`: `devices/` (per-device-kind handlers), `integrations/` (unifi, solar, synergy, woolworths, trmnl), `system/` (watchdog, cron, battery, push, mqtt_ingest, home_assistant_ingest, rpc), `home/` (alarm, sun, eink_display), plus `workflows/`, `events/` and `root/`. Modules that talk to an external service live under `src/integrations/`.

`main.rs` builds a `SharedActorState` (DB pool, MQTT client, event bus, device registry, settings) and starts `RootSupervisor` (`src/actors/root/`), which spawns and links every long-lived actor (device handlers, workflow dispatcher, watchdog, cron, unifi, solar, eink display, woolworths, etc.). Actors are looked up by name string via `ractor::registry::where_is`, so an actor's `NAME` const is effectively its address. Many device actors are `ractor` **factories** (worker pools) that take `FactoryMessage::Dispatch(Job { .. })`.

### Inbound event flow (the important path)
1. `Mqtt` (`src/integrations/mqtt/`) holds the broker connection, subscribes to `DeviceRegistry::mqtt_subscriptions`, and forwards every packet to the `MqttIngest` actor (`src/actors/system/mqtt_ingest/`).
2. `MqttTopic::classify` matches the topic against the `mqtt.protocols` topic templates (`config/sections/mqtt.yaml`): a protocol's `directory` (zigbee2mqtt's device list → friendly names), its `discovery` (esphome → subscribe to that node's topics), or one of its named `topics` (a report). A report resolves its device by `{address}`, or by `{name}` through the payload's `device.ieee_addr` / the friendly-name directory, and must belong to a model of that protocol.
3. MQTT and Home Assistant messages all take the same path: the device's **model** `decode` turns the input into a `DeviceReading`, and `decoding::dispatch` hands each role block to the owning **device actor** under `src/actors/devices/` (presence, environment, door, light, smart_switch, control_switch, plant, …). Each device actor persists to the DB and, on a meaningful state change, publishes an `EventBusMessage` onto the in-memory **event bus** (`src/event_bus/`).
4. The **workflow dispatcher** (`src/actors/workflows/dispatcher.rs`) subscribes to the event bus, matches messages against configured `triggers`, and dispatches to the parallel workflow factory (`src/actors/workflows/`), which executes `run:` steps (light control, notify, run another workflow, …) recursively.

The event bus does no matching — it is a thin `tokio::broadcast` fan-out. Producers publish without knowing which workflows (if any) consume.

### Device registry & config (`config/` + `src/settings/` + `src/device_registry/`)
Runtime config is a directory of YAML loaded at startup: `config/base.yaml` is the entry point and pulls in `config/devices/*.yaml` (one file per device type, listed in `devices/index.yaml`) and `config/workflows/*.yaml` via `!include` (the `yaml-include` crate). Secrets/overrides come from environment variables (`__`-separated, e.g. `MQTT__URL`).

`RawSettings::resolve` turns the raw YAML into `Settings` + a `DeviceRegistry`. Each device declares `transport: { type, address }` (`mqtt` — a zigbee IEEE address, esphome node name or valetudo identifier — `home_assistant` entity id, or `eink_display_firmware`/`trmnl`), a `model:` for mqtt/home_assistant, and `roles:` — pure behaviour config (names, door arming), never entity ids, capabilities or commands. The registry (`src/device_registry/`) holds one `Device { id, address, transport, profile, room, watchdog_key, roles: Roles }` per address plus lookup indexes (id alias, address → MQTT subscription topics, HA entity id → address) and the `MqttProtocols`. Which roles a device can carry is `Transport::supports` (`src/device_registry/transport.rs`) narrowed by `MqttProtocol::roles` (`src/integrations/mqtt/protocol.rs`) for mqtt models; eink/trmnl also require their own role. Watchdog keys are `<protocol or transport>:<id>`. The top-level device `id` is a stable alias referenced from workflows.

**MQTT protocols (`mqtt.protocols` in `config/sections/mqtt.yaml`):** each of `zigbee`/`esphome`/`valetudo` declares `topics` (named `TopicTemplate`s like `"valetudo/{address}/state"`; each needs `{address}` or `{name}`), an optional `command` template (filled with `address`, `name`, `domain`, `object_id`), and optionally `directory`/`discovery`. Subscriptions are the templates rendered per device (unknown vars become `+`; esphome renders one topic per model entity). Only payload parsing (`MqttProtocol::parse_payload`) and the directory/discovery payload formats stay in code.

**Device models (`config/lua/{mqtt,home_assistant}/<slug>.lua`, dirs set by `mqtt.models` / top-level `home_assistant_models:`):** a model describes a *product* and returns `{ roles = {...}, capabilities = {...}, decode = function(input) ... end }`, plus:
- `protocol = "zigbee" | "esphome" | "valetudo"` on every mqtt model (and never on a home_assistant one);
- `plant = { "soil_moisture" }` with the plant role;
- `commands = { robot_vacuum = { start, stop, dock } }` with the robot_vacuum role — MQTT payloads published to the protocol's `command` topic, or HA `domain.service` names;
- `entities` for per-entity protocols — esphome: `{ sensor = {...}, binary_sensor = {...}, light = {...} }` (the state topics to subscribe to; the light role needs exactly one light), home_assistant: entity-id templates like `"sensor.{name}_status"` where `{name}` is the address's object id. Zigbee and valetudo models take no `entities`.

`capabilities` (`Capability`: `brightness`/`colour_temp`/`rgb` + the environment `Metric`s) feed GraphQL and workflow validation; the environment role requires `temperature`. An mqtt `decode` receives `{ topic, vars, payload }` — the matched `topics` name, the template's extracted vars, and the payload (JSON, or for esphome already parsed per domain: number / bool / light table); a HA `decode` receives `{ entity_id, state, attributes }`. Either returns role blocks (`door = { contact }`, `environment = { temperature = … }`, `presence = { presence, sensor }`, `plant = { … }`, …) deserialized into `DeviceReading` (`src/decoding/reading.rs`; unknown blocks are an error, nil fields mean "not in this input"). Decoders are stateless: the environment actor merges partial readings per address, and the presence actor ORs readings keyed by `sensor`. The free-form `metrics = { name = value }` block lands in the generic `device_metric` sink. Everything is validated at startup (missing/unknown model, a role the transport or model can't carry, a role declared twice, a duplicate address/HA entity, a malformed protocol topic). LuaLS annotations live in `config/lua/types/{device,mqtt,zigbee,esphome,valetudo,home_assistant}.lua`.

**When adding a new device:** add it to the matching `config/devices/<type>.yaml` (a new type file also goes in `devices/index.yaml` and `config/kustomization.yaml`). If its model isn't there yet, add `config/lua/{mqtt,home_assistant}/<slug>.lua` plus its `config/kustomization.yaml` entry. No code change.

### API layer
Axum server in `main.rs`. Primary surface is **GraphQL** (`src/graphql/`: `QueryRoot`, `MutationRoot`, `SubscriptionRoot`, dataloaders, guards). REST routes (`src/routes/`) cover webhook ingests (unifi, solar, home/alarm, synergy), light control, admin key management, health, and metrics. Auth (`src/auth/`) supports both API keys and OAuth/OIDC JWTs, with scopes derived from token group memberships (`auth.oauth.group_scopes` in config; declarative keys live under `auth.api_keys`).

### Observability
OpenTelemetry (traces/metrics/logs) is wired throughout (`src/tracing_setup.rs`, `src/metrics.rs`); Prometheus metrics are exposed via a route.

## Notes
- Rust 2024 edition.
- Do not write comments — including doc comments — in new or edited code.
