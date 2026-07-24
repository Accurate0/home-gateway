# API keys provisioned via config + admin regeneration

## Context

Today API keys are fully imperative: an admin runs the `keys` CLI (`src/bin/keys.rs`)
against the `/v1/admin/keys` REST API, passing free-form `name` + `scopes`. The scopes
live only in the DB (`api_keys.scopes TEXT[]`), with no declarative record of *which*
key should have *what* access. That makes access hard to review or reproduce, and there
is no rotation path — to change a leaked key you must revoke and recreate it, getting a
new id.

The k8s `ai-gateway` service (`~/Projects/k8s/platform-services/ai-gateway`) solves this
with a hybrid model we want to replicate here:

- **Config is the declarative source of truth for a key's name + scope**, joined to the
  DB row by a unique `name`.
- **The admin API/CLI owns the secret material** — it mints and *regenerates* the token,
  returning plaintext exactly once; only the SHA-256 hash is stored.
- **Startup reconciles** config scopes onto existing rows by name (`claim()`), warning
  (not creating) when a declared key hasn't been minted yet.

Per user decisions: scopes are **config + API override** (config seeds/reconciles declared
keys, but the admin API can still set scopes manually); keys **not** in config are **left
untouched** on reconcile (non-destructive); `create` **mints first** for any name and lets
config catch up (a config entry is not required to mint).

## Changes

### 1. DB migration — unique name for config join
New file `migrations/<ts>_api_keys_unique_name.sql`:
- Add `CREATE UNIQUE INDEX idx_api_keys_name ON api_keys (name) WHERE revoked_at IS NULL;`
  (partial, so a name can be reused after its old key is revoked — mirrors ai-gateway's
  `0002_unique_key_name.sql` intent while tolerating revoked history).

Regenerate the sqlx cache after schema/query changes: `cargo sqlx prepare` (needs live
`DATABASE_URL`), else the offline build in `.sqlx/` fails.

### 2. Config schema — declare keys
`src/settings/mod.rs`:
- Add a struct `ApiKeySettings { name: String, scopes: Vec<String>, #[serde(default)]
  expires_at: Option<DateTime<Utc>> }` with `Deserialize, JsonSchema, Clone, Debug`,
  next to `OAuthSettings`.
- Add `#[serde(default)] api_keys: Vec<ApiKeySettings>` to both `RawSettings` (line ~220)
  and `Settings` (line ~193), threaded through the `RawSettings::resolve` destructure/build
  (lines 256–338). In `resolve`, validate each entry's scope strings with
  `ScopePattern::parse` (from `src/auth/scope.rs`) and reject unparseable scopes at load
  time — same fail-loud philosophy as `validate_device` (`src/settings/mod.rs:180`).
  Optionally reject duplicate `name`s within the config list.

`config/base.yaml`:
- Add a top-level block, sibling to `oauth.group_scopes`:
  ```yaml
  api_keys:
    - name: some-service
      scopes: ["graphql:*:read"]
  ```
- Regenerate `config/config.schema.json` from the `JsonSchema` derives if that file is
  kept in sync (check how it's currently generated).

### 3. Reconcile at startup (the `claim` pattern)
`src/auth/manager.rs` — add:
```rust
pub async fn claim(&self, name: &str, scopes: &[String],
    expires_at: Option<DateTime<Utc>>) -> Result<bool, sqlx::Error>
```
- `UPDATE api_keys SET scopes = $2, expires_at = COALESCE($3, expires_at)
   WHERE name = $1 AND revoked_at IS NULL RETURNING key_hash`.
- On a hit, `cache.invalidate(hash)` so the new scopes take effect immediately; return
  `true`. On miss return `false` (caller logs "config key not found; mint it via the
  admin API"). Model directly on `AuthManager::update`/`revoke` which already do the
  hash-fetch + cache-invalidate dance.

`src/main.rs` — after `AuthManager::new` (line 200), loop over `settings.api_keys` and
call `api_state.auth.claim(&k.name, &k.scopes, k.expires_at)` for each, `tracing::warn!`
when it returns `false`. This mirrors ai-gateway `src/main.rs`'s claim loop. Undeclared
DB keys are never touched — reconcile only updates by declared name.

### 4. Admin regenerate endpoint + manager method
`src/auth/manager.rs` — add:
```rust
pub async fn regenerate(&self, id: Uuid) -> Result<Option<CreatedKey>, sqlx::Error>
```
- Generate a fresh token exactly as `create` does (`KEY_PREFIX` + 40 `Alphanumeric`,
  new `key_prefix`, `hash_key`).
- Fetch the old `key_hash` and `UPDATE api_keys SET key_prefix=$2, key_hash=$3
  WHERE id=$1 AND revoked_at IS NULL RETURNING name, scopes, expires_at`.
- Invalidate **both** the old and new hashes in the moka cache so the old token stops
  authenticating immediately (ai-gateway flushes on regenerate for exactly this reason).
- Return `CreatedKey` (plaintext shown once) or `None` if the id doesn't exist / is revoked.

`src/routes/admin/keys.rs` — add handler `regenerate_key(Auth(auth), State, Path(id))`
gated on `auth.require(&required::ADMIN_KEYS_WRITE)` (same guard as `create_key`),
delegating to `auth.regenerate(id)`, returning `201` + `CreatedKey` or `404`.

`src/main.rs` line ~218 — add route:
```rust
.route("/admin/keys/{id}/regenerate", post(regenerate_key))
```

### 5. CLI support
`src/bin/keys.rs` — add a `Regenerate { id: Uuid }` variant to `enum Action`, issuing
`POST {base}/v1/admin/keys/{id}/regenerate` with the `X-Api-Key` header, printing the
returned `CreatedKey` with the same "store this key now, it will not be shown again"
notice as `Create`. No new payload type needed (empty body → `CreatedKey`).

## Notes / decisions baked in
- **Config + API override:** `create`/`update` keep their free-form `scopes` args; config
  `claim` overwrites scopes for *declared* names on every startup, so config wins for those
  and the API remains authoritative for everything else.
- **Non-destructive:** no auto-revoke of undeclared keys.
- **Mint-first:** `create` still accepts any name; a config entry is optional and only
  reconciles scopes if/when present.
- Secrets stay out of config — config carries only name + scope + optional expiry, never
  token material, matching the existing "secrets via env, not YAML" convention.

## Verification
1. `cargo sqlx prepare` (live DB), then `cargo build` — offline sqlx cache must compile.
2. `cargo test` — add/extend a settings test (near `config_yaml_parses_and_resolves`)
   asserting `api_keys:` parses and that an invalid scope string is rejected by `resolve`.
3. Add a manager test (DB-backed, matching existing `auth` tests if present) for
   `regenerate`: old hash no longer authenticates, new plaintext does; and for `claim`:
   scopes on a matching-name row are updated, a missing name returns `false`.
4. End-to-end with `docker compose up`:
   - `keys create --name some-service --scopes graphql:solar:read` → note id + plaintext.
   - Add `some-service` with different scopes to `config/base.yaml`, restart → log shows
     the claim; `keys list` shows the config scopes.
   - `keys regenerate <id>` → new plaintext; confirm old token now `401`, new token works.
