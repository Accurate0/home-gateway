# Home gateway development tasks

set shell := ["bash", "-euo", "pipefail", "-c"]

esphome_ref := "dev"
esphome_api := "https://raw.githubusercontent.com/esphome/esphome/" + esphome_ref + "/esphome/components/api"
esphome_image := "ghcr.io/esphome/esphome:latest"

# List available recipes
default:
    @just --list

# Compile, test, lint and format
check: build test clippy fmt

build:
    cargo build

test *args:
    cargo test {{ args }}

clippy:
    cargo clippy --all-targets

fmt:
    cargo fmt

# Refetch the esphome native api protobuf definitions (build.rs compiles them)
proto ref=esphome_ref:
    curl -fsSL "https://raw.githubusercontent.com/esphome/esphome/{{ ref }}/esphome/components/api/api.proto" -o proto/esphome/api.proto
    curl -fsSL "https://raw.githubusercontent.com/esphome/esphome/{{ ref }}/esphome/components/api/api_options.proto" -o proto/esphome/api_options.proto
    cargo build
    @git diff --stat proto/esphome

# Regenerate the json schemas the config files are validated against
schemas:
    SKIP_SCHEMA_VALIDATION=1 cargo run --bin gen_schema

# Regenerate the offline sqlx query cache (needs a live DATABASE_URL)
sqlx:
    cargo sqlx prepare

# Validate an esphome device config, e.g. `just esphome apollo-cast-1/livingroom.yaml`
esphome device:
    docker run --rm -v "$PWD/esphome":/config -w /config {{ esphome_image }} config {{ device }}

# Flash an esphome device over the network
esphome-run device:
    docker run --rm --network host -v "$PWD/esphome":/config -w /config {{ esphome_image }} run {{ device }}

# Run the gateway and TimescaleDB locally
up:
    docker compose up

# Regenerate the fish completions
completions:
    fish scripts/install-completions.fish --regenerate
