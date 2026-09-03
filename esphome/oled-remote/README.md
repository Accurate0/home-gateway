# OLED Remote

A handheld ESP32 remote — SH1106 128x64 OLED, 6-button d-pad — that drives this
repository's gateway over its HTTP GraphQL API. No Home Assistant in the path.

Ported from [plugowski/esphome_remote](https://github.com/plugowski/esphome_remote)
(GPL-3.0). See `NOTICE` for provenance and the list of changes; this directory is
GPL-3.0 while the rest of home-gateway is MIT.

## Modes

- **Lights** — every light the gateway exposes, discovered at runtime. Up/down
  scrolls, OK toggles, left/right dim and brighten (via the gateway's *relative*
  `brightnessMove` mutation, so the remote never has to know the current level).
  A dot beside the name marks a light with the `BRIGHTNESS` capability.
- **Media** — one player at a time. OK play/pauses, up/down skips tracks,
  left/right cycles players, a short press of the stop button turns the player off.
- **Settings** — contrast, sleep timeout, battery-check interval, wifi info.

Long-press the stop button (or press the menu button) for the mode carousel.

## Display area

The bottom rows of this unit's panel are physically damaged, so the UI draws
nothing below y=52 and there is no bottom button-hint bar (upstream had one at
y=53..63). Keep new drawing above that line.

## Talking to the gateway

`packages/gateway.yaml` holds the whole client. State is one query, polled every
30s while awake and re-read on wake and 1s after any mutation:

```graphql
{entities{__typename ... on LightEntity{id name on capabilities} ... on MediaPlayerEntity{id name state}}}
```

Actions are one-shot mutations (`light(id:){toggle}`, `mediaPlayer(id:){playPause}`,
…). Requests carry `X-Api-Key`. `src/gateway_client.h` parses the response into
fixed-size arrays — no dynamic allocation on the hot path.

The gateway's `entities` query **silently omits** entity types the key lacks scope
for, so a missing scope looks exactly like "no lights exist". If a mode shows
`NO LIGHTS` / `NO PLAYERS`, check the key's scopes before the firmware.

## Setup

1. Add an `oled-remote` entry to `api_keys:` in `config/base.yaml` (already done)
   and issue the key with the `keys` CLI or `POST /v1/admin/keys`.
2. Fill in `secrets.yaml` (gitignored): `gateway_url` (full base URL with
   scheme, no trailing slash) and `gateway_api_key`. Wifi credentials come from
   the shared `../common/network.yaml`.
3. `esphome run remote.yaml` over USB the first time, OTA after.

Wifi credentials come from `../common/network.yaml` like the other devices here.
That package also points at the house MQTT broker; this remote publishes nothing
the gateway ingests, so `remote.yaml` drops it with `mqtt: !remove`.

## Power

Three tiers: awake with wifi associated (a press acts immediately) → screen
blanked after `sleep_timeout_mins` (default 2), wifi still up → deep sleep after
`deep_sleep_after_mins` (default 30), waking on GPIO0 or on the battery timer.
