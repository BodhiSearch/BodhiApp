# 23 — Codebase patterns: persistence, admin routes, and backend tests for TunnelService

**Date:** 2026-09-15
**Scope:** Named Cloudflare tunnels only (quick tunnels out of scope). Maps the exact BodhiApp patterns a `TunnelService` + `/bodhi/v1/tunnel*` routes should follow. All claims are `file_path:line_number` from the current `main` tree; verify against the live file before implementing (migrations especially — they are immutable, the next one is `m20250101_000029_*`).

This extends `01-bodhi-app-codebase-map.md` (§2, §4, §6, §8) and `bodhiapp-cloudflare-tunnel-feasibility.md` with concrete persistence/route/test wiring. It does not repeat the Cloudflare-external research already in `00-consolidated-research.md`.

---

## (a) Persistence: settings-DB vs a dedicated table

**Verdict: dedicated table, not the settings key-value store.** The generic `settings` table cannot hold an encrypted secret, and tunnel state is a structured row (7+ fields with different lifecycles), not a handful of independent scalars.

### Why the settings table doesn't fit

`crates/services/src/db/sea_migrations/m20250101_000013_settings.rs:7-14` — the table is `(key PK, value TEXT, value_type TEXT, created_at, updated_at)`. `crates/services/src/settings/settings_repository.rs:7-12` — `SettingsRepository` (`get_setting`/`upsert_setting`/`delete_setting`/`list_settings`) stores **plaintext** `serde_yaml::Value` per key. It backs `DefaultSettingService`'s System→CommandLine→Environment→**Database**→SettingsFile→Default precedence chain (`crates/services/src/settings/default_service.rs`, and see `01-bodhi-app-codebase-map.md` §2). It has:
- No salt/nonce columns — cannot store an encrypted Cloudflare API token without bolting encryption onto a generic table that every other scalar setting also reads unencrypted.
- No structured/JSON value type in practice (`value_type` is a discriminator for scalar parsing, not a JSON blob convention anywhere in the codebase — grep confirms no JSON-in-settings usage).
- A flat key namespace shared with every other setting; a 7-field tunnel row split across 7 keys loses row-level atomicity (partial-write risk) and needs 7 round trips for one status read.

**What *does* belong in `SETTING_VARS`** (`crates/services/src/settings/constants.rs:82-104`, whitelist consumed by `settings_update`/`settings_destroy` via `EDIT_SETTINGS_ALLOWED`/`is_valid_db_key`): the **feature gate**, e.g. `BODHI_TUNNEL_ENABLED` (bool), mirroring `BODHI_CANONICAL_REDIRECT`. This is a simple on/off toggle admins may want visible in the existing `GET/PUT /bodhi/v1/settings` UI, consistent with the desktop-vs-container default from `01-bodhi-app-codebase-map.md` §1 (`SettingService::is_native()`, `crates/services/src/settings/setting_service.rs:189-191`). Everything else — tunnel identity, credentials, last-known state — goes in the new table.

### Recommended table: `tunnels` (singleton per instance)

Model on the **`api_model_oauth_credentials`** sibling-table pattern (`crates/services/src/models/llm_liberty_credentials_entity.rs:1-35`, migration `m20250101_000021_api_model_oauth_credentials.rs`), not on `tenants.encrypted_client_secret` (that one has legacy-decrypt baggage you don't want to inherit — see §(b)). One row per app instance (tenant_id as PK is fine since the whole app has exactly one tunnel; no user_id — this is admin/instance-scoped, not per-user).

```
CREATE TABLE tunnels (
  tenant_id              TEXT PRIMARY KEY,   -- singleton row; NOT NULL, matches DefaultDbService tenant scoping
  provider               TEXT NOT NULL,      -- 'cloudflare' (leaves room for future providers; store even though only one exists today)
  auth_mode              TEXT NOT NULL,      -- 'cloudflared_cli' | 'oauth' | 'api_token' (tier 1/2/3 from product decisions)
  status                 TEXT NOT NULL,      -- 'disabled' | 'provisioning' | 'enabled' | 'error' — mirrors download_requests.status pattern
  hostname               TEXT NULL,          -- e.g. "bodhi.example.com" (user-chosen, on their zone)
  tunnel_id              TEXT NULL,          -- Cloudflare cfd_tunnel UUID once created
  account_id             TEXT NULL,          -- Cloudflare account id (inferred from token scope per 00-consolidated-research.md §1)
  zone_id                TEXT NULL,
  credentials_path       TEXT NULL,          -- path to cloudflared's tunnel credentials JSON (tier 1: cloudflared CLI owns this file; app doesn't re-encrypt it)
  encrypted_api_token    TEXT NULL,          -- tier 3: pasted scoped Cloudflare API token, v2: AES-GCM (see (b))
  api_token_salt         TEXT NULL,
  api_token_nonce        TEXT NULL,
  last_public_url        TEXT NULL,          -- "https://bodhi.example.com" once connected
  last_error             TEXT NULL,
  last_synced_at         TIMESTAMPTZ NULL,   -- last Keycloak redirect-URI sync (per product decision: on enable/disable + startup only)
  created_at             TIMESTAMPTZ NOT NULL,
  updated_at             TIMESTAMPTZ NOT NULL
);
```

Rationale for the field split:
- `credentials_path` vs `encrypted_api_token` are mutually exclusive by `auth_mode` — tier 1 (`cloudflared tunnel login` via the CLI) writes `~/.cloudflared/cert.pem` + a per-tunnel credentials JSON that `cloudflared` itself manages; BodhiApp just remembers the path, it does **not** need to decrypt or re-encrypt Cloudflare's own file. Tier 3 (pasted API token) is the one secret BodhiApp itself must encrypt at rest — see §(b).
- `status` + `last_error` follow the `download_requests` shape (`crates/services/src/db/sea_migrations/m20250101_000001_download_requests.rs:6-19`) so the same polling-status UI pattern (§(c)) applies.
- No `RLS`/multi-tenant isolation test needed beyond the standard `begin_tenant_txn` pattern (`crates/CLAUDE.md` "Multi-Tenant Transactions") **if** tunnel is per-instance in standalone deployments; confirm whether multi-tenant (Docker, `BODHI_DEPLOYMENT=multi_tenant`) is in scope at all — product decision says native/desktop only by default, so multi-tenant rows may never be created in practice, but the tenant_id column keeps the table consistent with every other table in the schema (`crates/CLAUDE.md` "Multi-Tenant Transactions": *"All mutating DbService operations use `begin_tenant_txn(tenant_id)`"*).

### Migration sketch

Follow `crates/services/src/db/sea_migrations/m20250101_000021_api_model_oauth_credentials.rs` as the direct template (single `create_table`, no separate index migration needed since lookups are always by `tenant_id` PK). Governance from `crates/services/src/db/CLAUDE.md` "Migration Governance": this file, once committed, is **immutable forever** — any later column needs a new migration (pattern: `m20250101_000024_download_archived_at.rs`, one `ALTER TABLE` per column, SQLite can't chain).

```rust
// crates/services/src/db/sea_migrations/m20250101_000029_tunnels.rs
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Tunnels {
  Table, TenantId, Provider, AuthMode, Status, Hostname, TunnelId, AccountId, ZoneId,
  CredentialsPath, EncryptedApiToken, ApiTokenSalt, ApiTokenNonce,
  LastPublicUrl, LastError, LastSyncedAt, CreatedAt, UpdatedAt,
}

impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.create_table(
      Table::create()
        .table(Tunnels::Table)
        .col(string(Tunnels::TenantId).primary_key())
        .col(string(Tunnels::Provider))
        .col(string(Tunnels::AuthMode))
        .col(string(Tunnels::Status))
        .col(string_null(Tunnels::Hostname))
        .col(string_null(Tunnels::TunnelId))
        .col(string_null(Tunnels::AccountId))
        .col(string_null(Tunnels::ZoneId))
        .col(string_null(Tunnels::CredentialsPath))
        .col(string_null(Tunnels::EncryptedApiToken))
        .col(string_null(Tunnels::ApiTokenSalt))
        .col(string_null(Tunnels::ApiTokenNonce))
        .col(string_null(Tunnels::LastPublicUrl))
        .col(string_null(Tunnels::LastError))
        .col(timestamp_with_time_zone_null(Tunnels::LastSyncedAt))
        .col(timestamp_with_time_zone(Tunnels::CreatedAt))
        .col(timestamp_with_time_zone(Tunnels::UpdatedAt))
        .to_owned(),
    ).await
  }
  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Tunnels::Table).to_owned()).await
  }
}
```

Register in `crates/services/src/db/sea_migrations/mod.rs` — add `mod m20250101_000029_tunnels;` and append `Box::new(m20250101_000029_tunnels::Migration)` as the **last** entry (`crates/services/src/db/sea_migrations/mod.rs:66` is currently the last line before `]`). No backfill needed (new table, no existing rows) — this is the easy migration case per `crates/services/src/db/CLAUDE.md` "Data migration and backfill".

Entity + repository: new sibling files `crates/services/src/tunnels/tunnel_entity.rs` (SeaORM `DeriveEntityModel`, `pub type TunnelEntity = Model;` — the alias convention from `crates/services/CLAUDE.md` "Entity Aliases") and `crates/services/src/tunnels/tunnel_repository.rs` (trait `TunnelRepository: Send + Sync`, `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]`, impl on `DefaultDbService`) — directly mirroring `crates/services/src/models/llm_liberty_credentials_entity.rs` + `llm_liberty_credentials_repository.rs`.

---

## (b) Encryption helper to reuse, and its gotchas

Reuse `crate::db::encryption::{encrypt_api_key, decrypt_api_key}` (`crates/services/src/db/encryption.rs:141-173`) exactly as `LlmLibertyCredentialsRepository` does for the OAuth access/refresh tokens (`crates/services/src/models/llm_liberty_credentials_repository.rs:75-80`) or as `api_alias_repository.rs:87-90,200-209` does for a single API key. Signature:

```rust
pub fn encrypt_api_key(keys: &EncryptionKeys, api_key: &str) -> Result<(String, String, String)>; // (ciphertext, salt, nonce), all base64, ciphertext prefixed "v2:"
pub fn decrypt_api_key(keys: &EncryptionKeys, encrypted: &str, salt: &str, nonce: &str) -> Result<String>;
```

Store the three return values in `encrypted_api_token`/`api_token_salt`/`api_token_nonce` (all three, or all `NULL` when `auth_mode != 'api_token'`) — same nullable-triplet convention `api_alias_repository.rs:126-127` uses.

**Gotchas (all documented in `crates/services/src/db/CLAUDE.md` "Encryption" and `feedback_encryption_key_derivation.md`):**
1. **Never call `EncryptionKeys::derive` per request** — it's a 600k-iteration PBKDF2 (~70ms), done once at boot (`crates/services/src/db/encryption.rs:69-74`, comment at line 19). `TunnelRepository` gets the already-derived `EncryptionKeys` the same way every other repository does — via `DefaultDbService`'s `self.encryption_key` field (see `crates/services/src/models/llm_liberty_credentials_repository.rs:76` using `&self.encryption_key`).
2. **Tests must use `EncryptionKeys::for_test`** (`crates/services/src/db/encryption.rs:78-83`), not `derive` — real derivation in every test adds ~70ms × N tests. `test_utils::test_encryption_keys` / `TEST_ENCRYPTION_MASTER_KEY` in `crates/services/src/test_utils/db.rs` is the fixture source.
3. **`decrypt_api_key` rejects pre-v2 rows** with `EncryptionError::LegacyCiphertextUnsupported` (`crates/services/src/db/encryption.rs:167-169`). Since `tunnels` is a **brand-new table**, every row it ever writes will already be v2 (`encrypt_api_key` always emits the `v2:` prefix, line 153) — **do not** add a legacy-decrypt path or a `reencrypt_legacy_*` migration step for this table; that machinery exists only because `tenants.encrypted_client_secret` predates the v2 scheme (`crates/services/src/db/encryption.rs:175-176`, `crates/services/src/db/CLAUDE.md` "`decrypt_api_key_legacy` has exactly one caller").
4. **Map crypto failures through `DbError::from_encryption(entity, err)`** (`crates/services/src/db/error.rs:83-91`), never `DbError::EncryptionError(e.to_string())` directly — the former preserves the "recreate this resource" (422 `UnprocessableEntity`) vs "wrong key" (500 `InternalServer`) distinction (`crates/services/src/db/CLAUDE.md` bullet 3). Example call site: `crates/services/src/models/llm_liberty_credentials_repository.rs:79-80` currently uses the raw form (`DbError::EncryptionError(e.to_string())`) — **follow `from_encryption` instead** for the new table, it's the documented-correct pattern even though one existing call site hasn't been migrated to it.
5. **`cloudflared`'s own credentials JSON (tier 1) is not run through this encryption at all.** It's a file on disk that `cloudflared` itself manages (its own private-key format); BodhiApp stores only the `credentials_path` string in plaintext, same trust boundary as any other local file path setting. Do not encrypt a copy of it into the DB — that would create a second copy of a secret to keep in sync.

---

## (c) Route module skeleton, endpoint registration, and status-reporting pattern

### Endpoint constants (`crates/routes_app/src/shared/openapi.rs`)

`make_ui_endpoint!` (`crates/routes_app/src/shared/openapi.rs:78-82`) generates `pub const NAME: &str = concat!("/bodhi/v1/", "path");`. Add next to the other `ENDPOINT_MODELS_*` block (`crates/routes_app/src/shared/openapi.rs:104-113`):

```rust
make_ui_endpoint!(ENDPOINT_TUNNEL, "tunnel");
make_ui_endpoint!(ENDPOINT_TUNNEL_DETECT, "tunnel/detect");   // GET  — is cloudflared on PATH?
make_ui_endpoint!(ENDPOINT_TUNNEL_LOGIN, "tunnel/login");     // POST — tier-1 `cloudflared tunnel login` (opens browser)
make_ui_endpoint!(ENDPOINT_TUNNEL_PROVISION, "tunnel/provision"); // POST — create cfd_tunnel + DNS route + write config
make_ui_endpoint!(ENDPOINT_TUNNEL_ENABLE, "tunnel/enable");   // POST — start the tunnel process + sync Keycloak redirect URI
make_ui_endpoint!(ENDPOINT_TUNNEL_DISABLE, "tunnel/disable"); // POST — stop process + remove Keycloak redirect URI
```

(Six endpoints for six verbs is consistent with how the settings module gets one `ENDPOINT_SETTINGS` reused across GET/PUT/DELETE (`routes_settings.rs:9,54,131`) vs. how downloads gets `ENDPOINT_MODELS_FILES_PULL` reused across GET/POST plus `/{id}`, `/{id}/archive`, `/{id}/retry` suffixes (`routes_files_pull.rs:22,68,223,270,300`) — either style fits; the suffix style (`ENDPOINT_TUNNEL` + `/detect`, `/login`, etc. as string-formatted suffixes in `routes.rs`, matching `&format!("{ENDPOINT_MODELS_FILES_PULL}/{{id}}/retry")`) needs fewer new constants and is the more common pattern in this codebase — prefer it over six separate constants above.)

### Route module (`crates/routes_app/src/tunnel/routes_tunnel.rs`)

Skeleton mirrors `routes_settings.rs` (simple CRUD-ish, admin session only) crossed with `routes_files_pull.rs` (background job + status polling):

```rust
use crate::shared::AuthScope;
use crate::{BodhiErrorResponse, ValidatedJson};
use crate::{API_TAG_TUNNEL, ENDPOINT_TUNNEL};
use axum::Json;
use services::{TunnelStatusResponse, EnableTunnelRequest, ProvisionTunnelRequest};

#[utoipa::path(get, path = ENDPOINT_TUNNEL, tag = API_TAG_TUNNEL, operation_id = "getTunnelStatus",
  security(("session_auth" = ["resource_admin"])))]
pub async fn tunnel_show(auth_scope: AuthScope) -> Result<Json<TunnelStatusResponse>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().get_status().await?))
}

#[utoipa::path(get, path = ENDPOINT_TUNNEL.to_owned() + "/detect", ...)]
pub async fn tunnel_detect(auth_scope: AuthScope) -> Result<Json<CloudflaredDetection>, BodhiErrorResponse> { ... }

#[utoipa::path(post, path = ENDPOINT_TUNNEL.to_owned() + "/login", ...)]
pub async fn tunnel_login(auth_scope: AuthScope) -> Result<Json<TunnelStatusResponse>, BodhiErrorResponse> { ... } // spawns `cloudflared tunnel login`, returns immediately with status=provisioning; UI polls tunnel_show

#[utoipa::path(post, path = ENDPOINT_TUNNEL.to_owned() + "/provision", ...)]
pub async fn tunnel_provision(auth_scope: AuthScope, ValidatedJson(req): ValidatedJson<ProvisionTunnelRequest>) -> Result<Json<TunnelStatusResponse>, BodhiErrorResponse> { ... }

#[utoipa::path(post, path = ENDPOINT_TUNNEL.to_owned() + "/enable", ...)]
pub async fn tunnel_enable(auth_scope: AuthScope) -> Result<Json<TunnelStatusResponse>, BodhiErrorResponse> { ... } // starts cloudflared process + syncs Keycloak redirect URI (product decision: sync only on enable/disable + startup)

#[utoipa::path(post, path = ENDPOINT_TUNNEL.to_owned() + "/disable", ...)]
pub async fn tunnel_disable(auth_scope: AuthScope) -> Result<Json<TunnelStatusResponse>, BodhiErrorResponse> { ... }
```

`ValidatedJson<T>` extraction, `Result<_, BodhiErrorResponse>` return, blanket `From<T: AppError>` conversion — see `crates/routes_app/CLAUDE.md` "Error Handling Chain". `AuthScope` gives you `auth_scope.tunnels()` if you add a `tunnels()` accessor on `AuthScopedAppService` next to `mcps()`/`downloads()`/`settings()` (`crates/services/src/app_service/auth_scoped.rs:60,76,90`) — auth-scoped services inject `tenant_id`/`user_id` from `AuthContext` with no extra validation (`crates/services/CLAUDE.md` "AuthScoped services").

### Registration in `build_routes()` (`crates/routes_app/src/routes.rs`)

Tunnel enable/disable/provision/login are instance-wide, destructive-ish admin actions — same tier as settings. Add to the existing `admin_session_apis` router (`crates/routes_app/src/routes.rs:478-495`), which is already `route_layer`'d with `api_auth_middleware(ResourceRole::Admin, None, None, ...)` (line ~488) and merged into `session_protected` (restrictive CORS, `crates/routes_app/src/routes.rs:540`):

```rust
let admin_session_apis = Router::new()
  .route(ENDPOINT_SETTINGS, get(settings_index))
  // ... existing settings/mcp_servers routes ...
  .route(ENDPOINT_TUNNEL, get(tunnel_show))
  .route(&format!("{ENDPOINT_TUNNEL}/detect"), get(tunnel_detect))
  .route(&format!("{ENDPOINT_TUNNEL}/login"), post(tunnel_login))
  .route(&format!("{ENDPOINT_TUNNEL}/provision"), post(tunnel_provision))
  .route(&format!("{ENDPOINT_TUNNEL}/enable"), post(tunnel_enable))
  .route(&format!("{ENDPOINT_TUNNEL}/disable"), post(tunnel_disable))
  .route_layer(from_fn_with_state(state.clone(), move |state, req, next| {
    api_auth_middleware(ResourceRole::Admin, None, None, state, req, next)
  }));
```

This matches `01-bodhi-app-codebase-map.md` §4's recommendation and confirms `GET /bodhi/v1/tunnel` also belongs here (Admin session), not in a public/optional-auth group — tunnel state includes secret-adjacent fields (`credentials_path`, whether a token is configured) an unauthenticated caller shouldn't see. (`/bodhi/v1/info`, which lists public origins including the tunnel URl per product decision, is a **separate**, already-existing, lower-privilege endpoint — `setup_show`/`ENDPOINT_APP_INFO`, `crates/routes_app/src/routes.rs:118` — that just needs a new field, not new auth.)

### OpenAPI schema/path registration (`crates/routes_app/src/shared/openapi.rs`)

Checklist from `crates/routes_app/CLAUDE.md` "OpenAPI Registration Checklist" (lines under that heading):
1. `#[utoipa::path(...)]` on each handler → generates `__path_tunnel_show` etc.
2. New tag: add `API_TAG_TUNNEL` to `src/shared/constants.rs` next to `API_TAG_SETTINGS`, then a `tags((name = API_TAG_TUNNEL, description = "..."))` entry (`crates/routes_app/src/shared/openapi.rs:263` area).
3. Import the new `__path_*` symbols into the giant `use crate::{...}` block at the top (`crates/routes_app/src/shared/openapi.rs:1-45`, same grouping style as the "Settings and setup" import block at line 42).
4. Add response DTOs (`TunnelStatusResponse`, `ProvisionTunnelRequest`, `CloudflaredDetection`, etc.) to the `components(schemas(...))` list (`crates/routes_app/src/shared/openapi.rs:266` onward).
5. Add handler fns to the `paths(...)` list (`crates/routes_app/src/shared/openapi.rs:421` onward, grouped like the existing `// API Models endpoints` / `// Model Router endpoints` comment blocks).
6. `cargo run --package xtask openapi` → `cd ts-client && npm run generate` (or `make build.ts-client`, which wraps both per `ts-client/package.json`'s `generate` script chain: `generate:openapi` → `cargo run --package xtask openapi`, then `generate:types`/`generate:msw-types` etc.) → frontend imports `TunnelStatusResponse` etc. from `@bodhiapp/ts-client`.

### Status reporting: polling, not SSE — and why

**There is exactly one existing long-running-job pattern in this codebase: model downloads, and it is pure polling, no SSE.** Confirmed by reading the actual handler code, not assumption:

- `POST /bodhi/v1/models/files/pull` (`crates/routes_app/src/models/files/routes_files_pull.rs:119-141`, handler `models_pull_create`) creates a `download_requests` row (status `Pending`) and `tokio::spawn`s the real work (`spawn_pull`, lines ~185-218) — **returns immediately** with `201 Created` + the row, before the download runs.
- `GET /bodhi/v1/models/files/pull/{id}` (`routes_files_pull.rs:220-262`, handler `models_pull_show`) is a plain `auth_scope.downloads().get(&id).await?` — the frontend polls this.
- Progress is written into the same DB row via `Progress::Database(DatabaseProgress::new(db_service, tenant_id, request_id))` (`routes_files_pull.rs:206-212`; `DatabaseProgress` type from `services::`), which the hf-hub download callback drives with `.init()/.update()/.finish()` (see `.claude/skills/test-services/advanced.md` "DatabaseProgress Integration Test" for the exact call shape).
- No `text/event-stream` anywhere in this flow — `crates/server_core/src/fwd_sse.rs` (`fwd_sse()`) exists only for LLM chat-completion streaming, an unrelated subsystem. Grepping the whole `routes_app` crate for a status-streaming SSE endpoint of any kind (jobs, downloads, tunnels-equivalent) found none.

**Recommendation: reuse this exact pattern for tunnel provisioning.** `POST /bodhi/v1/tunnel/login` (and `/provision`, `/enable`) create-or-update the singleton `tunnels` row to `status = 'provisioning'`, spawn the `cloudflared` subprocess work in `tokio::spawn`, and return immediately; the UI polls `GET /bodhi/v1/tunnel` (cheap, single-row lookup) at a short interval until `status` settles to `enabled`/`error`. This is strictly simpler than SSE for a job that transitions through 2-4 states over tens of seconds (cloudflared login is a human-in-the-loop browser flow, so "poll every 1-2s" is more than adequate and matches what the UI already does for downloads). Do not introduce SSE for this — it would be the first status-streaming SSE endpoint in the codebase and there's no architectural precedent or clear win (no byte-level progress to stream, just a handful of discrete states).

---

## (d) Test plan skeletons

### `services` crate — `TunnelRepository` + `TunnelService` (per `.claude/skills/test-services/SKILL.md`)

```rust
// crates/services/src/tunnels/test_tunnel_repository.rs
use crate::test_utils::{test_db_service, TestDbService};
use crate::tunnels::TunnelRepository;
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_repository_upsert_and_get(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // upsert row with auth_mode=api_token, encrypted via db_service.encryption_key()
  // assert get_status roundtrips hostname/status/last_public_url
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_repository_api_token_roundtrip_through_encryption(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  // write via encrypt_api_key(db_service.encryption_key(), "cf-token"), read back via decrypt_api_key
  // assert decrypted == original; assert stored ciphertext starts with "v2:"
  Ok(())
}
```

Use `TestDbService` (real SQLite, `crates/services/src/test_utils/db.rs`) not `MockDbService` for the repository-layer tests — same as `llm_liberty_credentials_repository`'s own tests. Use `MockTunnelService` (from `#[cfg_attr(test, mockall::automock)]` on the service trait, `.claude/skills/test-services/mock-patterns.md` "Automock Traits") for anything one layer up (e.g. a Keycloak-sync coordinator) that only needs `TunnelService`'s behavior, not real persistence.

For the subprocess-spawning part of `TunnelService` (detect/login/provision/enable driving `cloudflared`), there is no existing "spawn a subprocess and parse its output" service in `services` to copy from — `llama_server_proc` is the only subprocess spawner in the codebase (`feedback_llama_server_proc_std_process.md`: uses `std::process` intentionally, not `tokio::process`, to avoid orphaned processes) but it lives in its own crate below `services` in the dependency chain and isn't reusable directly. Structure `TunnelService`'s binary-runner as an injectable trait (e.g. `CloudflaredRunner: Send + Sync` with `spawn_login`, `spawn_run`, `detect_binary`) so tests can substitute a fake — see §(e).

### `routes_app` crate (per `.claude/skills/test-routes-app/SKILL.md`)

**Auth tier tests** — `build_test_router()` (`crates/routes_app/src/test_utils`), following the `SKILL.md` "Auth Tier + Integration Tests" shape:

```rust
#[rstest]
#[case::status("GET", "/bodhi/v1/tunnel")]
#[case::detect("GET", "/bodhi/v1/tunnel/detect")]
#[case::enable("POST", "/bodhi/v1/tunnel/enable")]
#[case::disable("POST", "/bodhi/v1/tunnel/disable")]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) -> anyhow::Result<()> {
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_endpoints_reject_non_admin(
  #[values("resource_user", "resource_power_user", "resource_manager")] role: &str,
) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router.oneshot(session_request("GET", "/bodhi/v1/tunnel", &cookie)).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}
```

**Handler/business-logic tests** — `AppServiceStubBuilder` + a mocked `TunnelService` (SKILL.md "Isolated Handler Tests"):

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_enable_starts_provisioning_and_returns_status(...) -> anyhow::Result<()> {
  let mut mock_tunnels = MockTunnelService::new();
  mock_tunnels.expect_enable().times(1).returning(|_| Ok(TunnelStatus { status: "provisioning".into(), ..Default::default() }));
  let app_service = AppServiceStubBuilder::default().tunnel_service(Arc::new(mock_tunnels)).build()?;
  // build router with tunnel_enable handler + AuthScope over app_service, POST /bodhi/v1/tunnel/enable, assert 200 + body
  Ok(())
}

#[rstest]
#[case::cloudflared_not_found(TunnelServiceError::CloudflaredNotFound, StatusCode::NOT_FOUND)] // or whatever ErrorType maps to
#[case::already_enabled(TunnelServiceError::AlreadyEnabled, StatusCode::CONFLICT)]
#[tokio::test]
#[anyhow_trace]
async fn test_tunnel_enable_error_paths(...) -> anyhow::Result<()> { ... }
```

Error-path assertions go through `.code()` (`body["error"]["code"]`), never message text (`test-routes-app/SKILL.md` "Core Rules" #5, `.claude/skills/test-services/SKILL.md` "Error Code Convention").

### `server_app` crate — real HTTP, multi-turn

`server_app` tests hit a real HTTP server and, per the existing tests (`crates/server_app/tests/test_live_multi_tenant.rs:1-9`, `test_live_mcp.rs`, `test_oauth_external_token.rs`), a **real Keycloak** driven entirely by env vars (`INTEG_TEST_AUTH_URL`, `INTEG_TEST_AUTH_REALM`, `crates/server_app/tests/utils/live_server_utils.rs:78-83`) — never a mocked `AuthService` at this layer (contrast with `routes_app`'s `MockAuthService`, which `build_test_router()` wires and which **panics if called**, `.claude/skills/test-routes-app/fixtures.md` "Services Wired" table). This matches `feedback_no_skip_for_missing_env.md`/`feedback_e2e_external_keycloak_flakiness.md`: these tests throw in setup (`.map_err(|_| anyhow::anyhow!("INTEG_TEST_AUTH_URL not set"))`, line 79) rather than skip, and are annotated `#[serial_test::serial(live)]` per `crates/CLAUDE.md` "Test boundaries".

For a tunnel `server_app` test:

```rust
// crates/server_app/tests/test_live_tunnel.rs
mod utils;
use utils::{live_server, TestServerHandle};

#[anyhow_trace]
#[rstest]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_tunnel_enable_disable_full_flow(
  #[future] live_server: TestServerHandle,
) -> anyhow::Result<()> {
  // 1. PATH is pointed at the fake cloudflared stub (see (e)) via BODHI_TUNNEL_CLOUDFLARED_PATH or plain PATH prepend
  // 2. admin session login against real Keycloak (same helper live_server_utils.rs uses)
  // 3. POST /bodhi/v1/tunnel/detect -> 200, cloudflared_found: true
  // 4. POST /bodhi/v1/tunnel/enable -> spawns fake cloudflared, poll GET /bodhi/v1/tunnel until status == "enabled" (bounded loop + timeout, no fixed sleep)
  // 5. assert GET /bodhi/v1/info now lists the tunnel origin (flagged as tunnel) per product decision
  // 6. assert Keycloak redirect URI now includes the tunnel hostname (requires either the SPI test client from auth-server-client.mjs equivalent in Rust, or a check against the app's own AuthService call)
  // 7. POST /bodhi/v1/tunnel/disable -> status == "disabled", redirect URI removed
  Ok(())
}
```

Whether this needs a **real** Keycloak SPI call to verify the redirect-URI sync depends on whether the redirect-URI-update method is implemented as (1) a new Bodhi SPI endpoint (per `01-bodhi-app-codebase-map.md` §6 option 1, the recommended one) — in which case the sibling `keycloak-bodhi-ext` repo needs the endpoint deployed in whatever Keycloak the `INTEG_TEST_AUTH_URL` points at before this test can pass — or (2) mocked at the `AuthService` boundary for this specific assertion. Given `server_app` tests never mock `AuthService`, prefer (1) and treat this as a cross-repo test dependency to flag explicitly in the implementation plan.

---

## (e) Faking `cloudflared` in tests

**Cross-reference:** the exact `cloudflared tunnel run` argv, the `/ready`-polling vs log-line-scraping decision, and the precise stub contract this fake binary must implement are now pinned in `10-cloudflared-cli-named-tunnel-lifecycle.md`, "Follow-up: Pin the exact cloudflared invocation..." section (§A–D at the end of that doc) — read that before implementing the sketch below; §D supersedes the `tunnel run` case in the sketch (it never binds `/ready`, which §B makes the primary signal).

**The repo does not currently fake any external binary.** Grepping `crates/services/src` and `crates/llama_server_proc/src` for `std::process::Command`/`tokio::process::Command` found no results outside `llama_server_proc`'s own `llama-server` launcher, and `llama_server_proc` tests use the **real** `llama-server` binary checked into `crates/llama_server_proc/bin/<target-triple>/` (referenced via `BODHI_EXEC_LOOKUP_PATH`, see `crates/server_app/tests/utils/live_server_utils.rs:63-70` setting that env var to the real `bin/` dir). There is no precedent for a shell/Python stub binary anywhere in the codebase — this would be new infrastructure.

### Recommended approach

1. **Design `TunnelService`'s `cloudflared` interaction behind a small trait** (as noted in §(d)) so `routes_app`/`services` unit tests inject a mock and never touch a real process at all. This covers the bulk of test coverage cheaply.
2. **For `server_app` (real-process) tests, write a fake `cloudflared` executable** and prepend its directory to `PATH` (or point a `BODHI_TUNNEL_CLOUDFLARED_PATH`-style setting at it, mirroring `BODHI_EXEC_LOOKUP_PATH`'s override-the-search-path pattern) for the test process only. Concretely:
   - A checked-in shell script (portable enough for CI Linux/macOS; Windows CI, if any, would need a `.cmd` — check whether `server_app` live tests run on Windows CI at all before over-investing here) at e.g. `crates/server_app/tests/resources/fake-cloudflared`:
     ```sh
     #!/usr/bin/env bash
     case "$1 $2" in
       "tunnel login")
         echo "A browser window should have opened. If it hasn't, please open this URL:"
         echo "https://dash.cloudflare.com/argotunnel?..."
         echo "You have successfully logged in."
         exit 0 ;;
       "tunnel create")
         echo "Created tunnel fake-tunnel-id with id fake-tunnel-id"
         exit 0 ;;
       "tunnel run")
         # emit the log lines TunnelService's parser watches for, then block until killed
         echo "INF Connection registered connIndex=0"
         echo "INF Registered tunnel connection"
         trap 'exit 0' TERM
         sleep 3600 & wait ;;
       *) exit 1 ;;
     esac
     ```
   - `TunnelService` must already be built to parse `cloudflared`'s real stdout log lines (per `00-consolidated-research.md`'s note that the URL/connection state comes from stdout parsing, same as the quick-tunnel case) — so the fake script's job is to emit **exactly** those substrings, keeping the parser identical between fake and real runs.
   - Make the script executable at test-fixture-build time (`chmod +x` in a `build.rs` or a `#[fixture]` setup step) rather than relying on git preserving the executable bit across checkouts/CI runners — safer to `std::fs::Permissions` it at test startup.
3. **A `/ready`-style health check is unnecessary for `cloudflared` specifically** — real `cloudflared` doesn't expose one either; the codebase's existing signal for "the subprocess is up" is log-line matching (this is exactly how quick-tunnel URL discovery works per `00-consolidated-research.md`), so the fake should match that contract rather than invent an HTTP probe the real binary doesn't have.
4. Keep the fake **out of `services`/`routes_app` crate tests** entirely — those should never spawn a process, real or fake (per the trait-injection approach in point 1). Confine the fake-binary integration test to `server_app`, where "real HTTP, real subprocess, real (or faked) external dependency" is already the established boundary (`crates/CLAUDE.md` "Test boundaries": `server_app` = multi-turn real HTTP).

---

## Summary table: file:line quick reference

| Pattern | File:line |
|---|---|
| Settings KV table schema | `crates/services/src/db/sea_migrations/m20250101_000013_settings.rs:7-14` |
| `SettingsRepository` trait | `crates/services/src/settings/settings_repository.rs:7-12` |
| `SETTING_VARS` whitelist | `crates/services/src/settings/constants.rs:82-104` |
| Sibling encrypted-credentials table (model to copy) | `crates/services/src/models/llm_liberty_credentials_entity.rs:1-35`, `llm_liberty_credentials_repository.rs:63-131` |
| Single-secret nullable-triplet pattern | `crates/services/src/models/api_alias_repository.rs:87-90,126-127,200-209` |
| `encrypt_api_key`/`decrypt_api_key` | `crates/services/src/db/encryption.rs:141-173` |
| `EncryptionKeys::for_test` | `crates/services/src/db/encryption.rs:78-83` |
| `DbError::from_encryption` | `crates/services/src/db/error.rs:83-91` |
| Migration governance | `crates/services/src/db/CLAUDE.md` "Migration Governance", "Registering a migration" |
| `is_native()` gate | `crates/services/src/settings/setting_service.rs:189-191` |
| `admin_session_apis` router | `crates/routes_app/src/routes.rs:478-495` |
| `make_ui_endpoint!` + `ENDPOINT_*` | `crates/routes_app/src/shared/openapi.rs:78-113` |
| OpenAPI `components(schemas(...))` / `paths(...)` | `crates/routes_app/src/shared/openapi.rs:266`, `:421` |
| Settings routes (simple admin CRUD model) | `crates/routes_app/src/settings/routes_settings.rs:9-52` |
| Download create+spawn+poll (job-status model) | `crates/routes_app/src/models/files/routes_files_pull.rs:68-141` (create), `:220-262` (get status) |
| `AuthScope` sub-service accessors | `crates/services/src/app_service/auth_scoped.rs:60,76,90` |
| `AppService` trait | `crates/services/src/app_service/app_service.rs:10-40` |
| `test-routes-app` skill | `.claude/skills/test-routes-app/SKILL.md` |
| `test-services` skill | `.claude/skills/test-services/SKILL.md`, `advanced.md` (`wait_for_event!`, `DatabaseProgress` test) |
| `server_app` live-Keycloak fixture | `crates/server_app/tests/utils/live_server_utils.rs:78-83` |
| ts-client regen chain | `ts-client/package.json` `generate` script; `cargo run --package xtask openapi` |

---

## Open questions for the implementation plan (not answered here)

- Whether `tunnels` needs true multi-tenant RLS/isolation tests (`crates/services/src/mcps/test_mcp_repository_isolation.rs` pattern) or whether "native/desktop only, one tenant" makes that moot — depends on whether Docker/multi-tenant mode is ever allowed to flip the feature flag, even experimentally.
- Whether the Keycloak redirect-URI sync is implemented as SPI extension (needs `keycloak-bodhi-ext` changes landed and deployed to whatever Keycloak `INTEG_TEST_AUTH_URL` points at before `server_app` live tests can assert it) or Admin REST API (`01-bodhi-app-codebase-map.md` §6 option 2) — this gates whether the `server_app` test in §(d) can be written before the sibling-repo work lands.
- ~~Exact wire shape of `TunnelStatusResponse`/`ProvisionTunnelRequest` DTOs (fields, `has_api_token: bool` vs full secret-masking convention used elsewhere, e.g. `ApiAliasResponse`'s `has_api_key: bool` per `crates/services/CLAUDE.md` "Response types").~~ **Resolved: schema finalized in `21-codebase-settings-network-and-info.md` §6, use that verbatim** (also drops the unused `EnableTunnelRequest` import from the route skeleton above).

## Sources

All findings above are direct reads of the BodhiApp repository at the commit checked out during this research (`main`, see git status in the task context — HEAD around `d4268ddb`). No external sources were consulted for this document; see `00-consolidated-research.md` and `bodhiapp-cloudflare-tunnel-feasibility.md` for the external Cloudflare/`cloudflared` research this extends.

---

## Follow-up: Confirm whether server_app / E2E live tests run on Windows CI, to scope Windows test infra

**Verdict: No. As of 2026-09-15, `server_app` / E2E live tests do not run on Windows CI — and, beyond the Windows-specific question, no backend/E2E workflow currently runs automatically on push/PR at all.** The three workflows capable of running them are all `disabled_manually` in GitHub Actions (confirmed via `gh workflow list --all`, not visible from the YAML alone). This converts doc 10/23's open question into a stated fact and changes the scoping recommendation.

### The one workflow with a Windows test matrix entry

- `.github/workflows/build-multiplatform.yml:14-24` — the `build` job's matrix includes `windows-latest` / `x86_64-pc-windows-msvc` alongside `macos-latest` and `ubuntu-latest-4-cores`.
- That job's `build-and-test` step (`build-multiplatform.yml:69-76`) invokes `.github/actions/build-and-test/action.yml:20-22`, which runs plain `make ci.coverage` (no `-f Makefile.win.mk`, no platform branch) on every matrix OS including Windows.
- `make ci.coverage` resolves via `Makefile.ci.mk:14-15` → `$(MAKE) test.coverage` → `Makefile:127-130`: `cargo llvm-cov test --no-fail-fast --all-features $PACKAGES --lcov --output-path lcov.info`, where `$PACKAGES` is every workspace member name from `cargo metadata` — confirmed `server_app` is a workspace member (`cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'` includes `server_app`) and there is **no** package filter excluding it and **no** `cfg(windows)`/`cfg(not(windows))`/`cfg(target_os = "windows")` gate anywhere under `crates/server_app/` (grep found zero hits) that would skip its live tests on that OS.
- Real Keycloak credentials are wired into this exact job for all platforms (`build-multiplatform.yml:69-76`: `INTEG_TEST_AUTH_URL`, `INTEG_TEST_AUTH_REALM`, `INTEG_TEST_USERNAME`, `INTEG_TEST_PASSWORD`, `INTEG_TEST_DEV_CONSOLE_CLIENT_ID/SECRET`), matching what `crates/server_app/tests/utils/live_server_utils.rs:78-83` reads — so **if this workflow ran to completion**, `server_app`'s live tests (`test_live_multi_tenant.rs`, `test_live_mcp.rs`, `test_oauth_external_token.rs`, and any future `test_live_tunnel.rs`) would execute for real against Keycloak on `windows-latest`, not be skipped.

### But the workflow is disabled and has never completed

```
$ gh workflow list --all
...
Fast Linux Build and Test                     disabled_manually  96778628
Mac/Linux/Windows Multiplatform Build Flow    disabled_manually  177144133
Playwright Tests                              disabled_manually  191323655
```

- `build-multiplatform.yml` (workflow id `177144133`) has exactly **one run in its entire history**: a manual `workflow_dispatch` on 2025-08-11 (run `16882936920`) that was **cancelled**, not completed. Despite defining `on.push.branches: [main]` (`build-multiplatform.yml:4-6`), it has **never had a push-triggered run** — `gh run list --workflow=build-multiplatform.yml` returns only that one row.
- The two Linux-only workflows that *do* actively run backend/E2E tests are also now disabled: `build.yml` ("Fast Linux Build and Test") last ran 2026-03-13 (`gh run list --workflow=build.yml`), `playwright.yml` ("Playwright Tests") last ran 2025-10-05. Both show `disabled_manually`.
- Net effect: **no workflow in this repo currently runs `server_app` or Playwright E2E tests automatically on any OS**, Windows included. `gh workflow list` (active-only, no `--all`) confirms only release/publish/deploy workflows are live (`App Release Flow`, `Publish TypeScript Client`, `Publish Docker Image (Multi-Variant/Platform)`, `Publish app-bindings`, `Deploy Website to GitHub Pages`, `Publish Docker MT CPU Image`, `Dependabot Updates`) — none of these run `cargo test`.

### Windows-specific Makefile is dead code from CI's perspective

- `Makefile.win.mk` exists with its own `ci.coverage: pwsh -NoProfile -File scripts/coverage.win.ps1` target (`Makefile.win.mk:16-17`), a separate PowerShell-driven coverage path distinct from the default `Makefile`'s.
- No workflow (active or disabled) invokes it — `grep -rn "Makefile.win.mk" .github/` finds exactly one use, `.github/actions/setup-node/action.yml:38`, and it's scoped to `ci.app-npm` (npm install) only. The `build-and-test`/`build-only` composite actions call bare `make ci.coverage` / `make ci.clean` with no `-f` flag on every platform (`.github/actions/build-and-test/action.yml:16,20`), so on Windows this resolves to the default `Makefile`'s `ci.coverage` (via `Makefile.ci.mk`), **not** `Makefile.win.mk`'s. `.github/actions/setup-win/action.yml` (the action that `choco install`s `make` itself) is likewise not referenced by any current workflow YAML — only mentioned in `.github/PACKAGE.md`.
- Practical implication: if `build-multiplatform.yml` were re-enabled today, GNU Make on `windows-latest` would need to already be present in the hosted runner image (GitHub's `windows-latest` images ship GNU Make — UNVERIFIED against the exact current image manifest, but consistent with `setup-win` no longer being wired in) for `make ci.coverage` to run at all; `scripts/coverage.win.ps1` would not be exercised.

### Tauri desktop release workflow on Windows: build only, zero tests

- `.github/workflows/release.yml` ("App Release Flow") is the one **active** workflow with a `windows-latest` matrix entry (`release.yml:32-34`), and it is marked `optional: true` — a Windows build failure does not fail the release.
- Its steps for that platform: `make ci.clean` → `make ci.update-version` → `cargo build --release -p lib_bodhiserver --locked` (`release.yml:104-109`) → `tauri-apps/tauri-action@v0` to produce the `.msi` bundle. **No `cargo test`, `make test.backend`, or `make ci.coverage` step appears anywhere in this job** — confirms the desktop release pipeline builds the Windows installer only and never runs Rust tests on Windows.

### Consequence for the tunnel plan

- Doc 23 §(e)'s `.cmd` fake-`cloudflared` stub question is answered: **do not build it now.** No currently-running CI job would execute it. Ship the POSIX shell fake (`crates/server_app/tests/resources/fake-cloudflared`) only, scoped to whatever CI *does* run (currently: nothing automated — see below).
- Doc 10's Windows shutdown gotchas (no POSIX SIGTERM, `CTRL_BREAK_EVENT` + `CREATE_NEW_PROCESS_GROUP`) are **exercised by neither automated CI nor a currently-passing manual workflow run** — only by ad hoc manual QA before a release, and even that QA path (`release.yml`'s Windows job) never runs the app's own test suite, only the Tauri build. Flag this as a real gap in the implementation plan's risk section, not just a scoping question: a tunnel-triggered graceful-shutdown regression on Windows has no CI signal today, disabled or not.
- Broader, out-of-scope-for-this-gap finding worth surfacing to the plan's stakeholders: `make test.backend`/`make test.e2e` (the commands this repo's own `CLAUDE.md` documents as the way to validate changes) are **not what CI runs** even when the workflows are enabled — CI uses `make ci.coverage` (`cargo llvm-cov test --all-features` over all packages) and `npm run test:playwright:ci`, parallel but distinct commands. And right now none of it runs automatically regardless of platform; re-enabling `build.yml` (Linux) and/or `build-multiplatform.yml` (Linux+macOS+Windows) is a prerequisite for any Windows tunnel-shutdown test coverage claim, independent of anything this tunnel feature needs to add.

### Sources (this follow-up)

- `.github/workflows/build-multiplatform.yml`, `.github/workflows/build.yml`, `.github/workflows/playwright.yml`, `.github/workflows/release.yml` (read directly, current `main`)
- `.github/actions/build-and-test/action.yml`, `.github/actions/build-only/action.yml`, `.github/actions/setup-node/action.yml`, `.github/actions/setup-win/action.yml`, `.github/actions/setup-rust/action.yml`, `.github/actions/setup-environment/action.yml`
- `Makefile:127-130`, `Makefile.ci.mk:10-22`, `Makefile.win.mk:13-17`
- `gh workflow list --all` (GitHub CLI, live query against `BodhiSearch/BodhiApp` — workflow IDs `96778628`, `177144133`, `191323655` all `disabled_manually` at query time)
- `gh run list --workflow=build-multiplatform.yml/build.yml/playwright.yml --json ...` (GitHub CLI, live query — run history and conclusions cited above)
- `cargo metadata --no-deps --format-version 1` (local, confirms `server_app` workspace membership)
- `grep -rn "cfg(windows)\|cfg(not(windows))\|cfg(target_os" crates/server_app/` (local, zero hits)
