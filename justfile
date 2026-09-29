# Home gateway development tasks

set shell := ["bash", "-euo", "pipefail", "-c"]

esphome_ref := "dev"
esphome_api := "https://raw.githubusercontent.com/esphome/esphome/" + esphome_ref + "/esphome/components/api"
esphome_image := "ghcr.io/esphome/esphome:latest"
local_bucket := "home-gateway-bucket"
local_s3 := "AWS_ACCESS_KEY_ID=home-gateway AWS_SECRET_ACCESS_KEY=home-gateway-local AWS_REGION=ap-southeast-2 aws --endpoint-url http://localhost:9000"

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

# Build the eink firmware against the local gateway and publish it to the local s3 store
eink-firmware-local:
    #!/usr/bin/env bash
    set -euo pipefail
    : "${API_KEY:?set the api key the display uses against the local gateway}"
    if [ -f ~/export-esp.sh ]; then . ~/export-esp.sh; fi
    cd eink-display-firmware
    package_version=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')
    version="v${package_version}-local.$(date +%s)"
    image="firmware_${version}.bin"
    HOME_GATEWAY_API_KEY="$API_KEY" \
        WIFI_SSID="${WIFI_SSID:-test}" \
        WIFI_PASSWORD="${WIFI_PASSWORD:-test}" \
        FIRMWARE_VERSION="$version" \
        cargo espflash save-image --chip esp32s3 "$image"
    {{ local_s3 }} s3 cp "$image" "s3://{{ local_bucket }}/eink-display/firmware/$image"
    rm "$image"
    echo "published $version, run the gateway with EINK_DISPLAY__FIRMWARE_VERSION=$version"

# Build the eink dashboard against the local gateway and publish it to the local s3 store
eink-web-local:
    #!/usr/bin/env bash
    set -euo pipefail
    : "${API_KEY:?set the api key the dashboard uses against the local gateway}"
    NODE_ENV=development VITE_GRAPHQL_API_KEY="$API_KEY" pnpm --filter eink-display-web build
    {{ local_s3 }} s3 cp eink-display-web/dist/index.html "s3://{{ local_bucket }}/eink-display/web/index.html" --content-type text/html

# Regenerate the fish completions
completions:
    fish scripts/install-completions.fish --regenerate
