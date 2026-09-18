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
1. `Mqtt` (`src/integrations/mqtt/`) holds the broker connection and forwards every packet to the `MqttIngest` actor (`src/actors/system/mqtt_ingest/`).
2. `MqttIngest::classify` routes each topic: `zigbee2mqtt/bridge/devices` (device list), `zigbee2mqtt/<name>` (a device state report — looked up in the registry by the payload's `device.ieee_addr`, then read field-by-field through its **model profile**), `esphome/discover/<node>` (discovery → subscribe to that node's registered state topics), or `Other` (an esphome state topic resolved by exact lookup in the device registry).
3. The ingest actor forwards a typed event to the owning **device actor** under `src/actors/devices/` (presence, environment, door, light, smart_switch, control_switch, plant). Each device actor persists to the DB and, on a meaningful state change, publishes an `EventBusMessage` onto the in-memory **event bus** (`src/event_bus/`).
4. The **workflow dispatcher** (`src/actors/workflows/dispatcher.rs`) subscribes to the event bus, matches messages against configured `triggers`, and dispatches to the parallel workflow factory (`src/actors/workflows/`), which executes `run:` steps (light control, notify, run another workflow, …) recursively.

The event bus does no matching — it is a thin `tokio::broadcast` fan-out. Producers publish without knowing which workflows (if any) consume.

### Device registry & config (`config/` + `src/settings/` + `src/device_registry/`)
Runtime config is a directory of YAML loaded at startup: `config/base.yaml` is the entry point and pulls in `config/devices/*.yaml` (one file per device type, listed in `devices/index.yaml`) and `config/workflows/*.yaml` via `!include` (the `yaml-include` crate). Secrets/overrides come from environment variables (`__`-separated, e.g. `MQTT__URL`).

`RawSettings::resolve` turns the raw YAML into `Settings` + a `DeviceRegistry`. Each device declares a `transport` (`zigbee` | `esphome`), an `address` (zigbee IEEE address, or esphome node name which is the MQTT topic prefix), and one or more `kinds` (door/appliance/presence/environment/plant/light/control_switch/smart_switch). The registry keys settings by address and, for esphome, precomputes the exact set of state topics to subscribe to (`esphome_topics`, `EsphomeTarget`) so incoming topics route by lookup rather than by re-parsing shape. The top-level device `id` is a stable alias referenced from workflows.

**Zigbee model decoders (`config/lua/zigbee/<slug>.lua`, dir set by top-level `zigbee_models:`):** zigbee payloads are *not* parsed by per-model Rust structs — a device declares `model: <slug>` in its `config/devices/<type>.yaml` file and the slug's Lua module decodes the payload. A module returns `{ roles = {...}, environment = {...}, decode = function(payload) ... end }`: `roles` lists what the model feeds (`battery`/`door`/`environment`/`light`/`smart_switch`/`presence`/`control_switch`), `environment` lists its `Metric`s (required with the environment role, must include `temperature`), and `decode` returns a table of role blocks (`door = { contact }`, `smart_switch = { state, voltage, power, current, energy }`, …) deserialized into `ZigbeeReading` (`src/settings/devices/zigbee_reading.rs`; unknown blocks are an error, nil fields mean "not in this payload"). The free-form `metrics = { name = value }` block lands in the generic `device_metric` sink (numbers → `value`, strings/bools → `text_value`). All modules share one long-lived sandboxed `LuaDecoder` (`src/lua/decoder.rs`, no `gw` API). Everything is validated at startup: a zigbee device without a `model:`, an unknown slug, a module missing `roles`/`decode`, or a device role the model doesn't declare is a hard error, and an unregistered address warns at runtime. LuaLS annotations live in `config/lua/types/zigbee.lua`.

**When adding a new zigbee device:** add its address + `model:` to the matching `config/devices/<type>.yaml` (a new type file also goes in `devices/index.yaml` and `config/kustomization.yaml`), and add `config/lua/zigbee/<slug>.lua` (plus its `config/kustomization.yaml` entry) if that model isn't already there. No code change.

**When adding a new ESPHome environment sensor/device:** map metrics to object_ids directly in `config/devices/environment.yaml` — the environment kind's `entities:` is a required `metric: object_id` map (metrics: `temperature`/`humidity`/`pressure`/`lux`/`uv_index`/`pm25`/`voc_index`, the `Metric` enum in `src/settings/devices/environment.rs`, shared with the zigbee decoders' `environment` list). No code change is needed for environment sensors. An esphome environment device with an empty `entities:` map is rejected at load. Non-environment esphome device kinds (presence, plant) still route by config entity lists.

### API layer
Axum server in `main.rs`. Primary surface is **GraphQL** (`src/graphql/`: `QueryRoot`, `MutationRoot`, `SubscriptionRoot`, dataloaders, guards). REST routes (`src/routes/`) cover webhook ingests (unifi, solar, home/alarm, synergy), light control, admin key management, health, and metrics. Auth (`src/auth/`) supports both API keys and OAuth/OIDC JWTs, with scopes derived from token group memberships (`auth.oauth.group_scopes` in config; declarative keys live under `auth.api_keys`).

### Observability
OpenTelemetry (traces/metrics/logs) is wired throughout (`src/tracing_setup.rs`, `src/metrics.rs`); Prometheus metrics are exposed via a route.

## Notes
- Rust 2024 edition.
- Do not write comments — including doc comments — in new or edited code.
