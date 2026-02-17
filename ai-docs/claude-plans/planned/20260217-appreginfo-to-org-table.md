# Plan: Move AppRegInfo & AppStatus from secrets.yaml to Database

## Context

AppRegInfo (OAuth client credentials) and AppStatus (app lifecycle state) are currently stored in an AES-256-GCM encrypted file (`secrets.yaml`) managed by `SecretService`. As a step toward multi-tenancy (where multiple orgs will each have their own registration), we're moving this data into the SQLite database as an `organizations` table. This also lets us fully remove the `SecretService` abstraction, simplifying the service layer.

No backwards compatibility needed - no production release yet. Clean cut implementation.

## Migration SQL

**`crates/services/migrations/0010_organizations.up.sql`**
```sql
CREATE TABLE IF NOT EXISTS organizations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    client_secret_encrypted TEXT NOT NULL,
    client_secret_salt TEXT NOT NULL,
    client_secret_nonce TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    app_status TEXT NOT NULL DEFAULT 'setup',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**`crates/services/migrations/0010_organizations.down.sql`**
```sql
DROP TABLE IF EXISTS organizations;
```

## New Types

### Organization struct (`crates/services/src/objs.rs`)
Replaces `AppRegInfo`. Contains all DB columns with `client_secret` decrypted in memory:
- `id: i64`, `client_id: String`, `client_secret: String` (decrypted), `scope: String`, `app_status: AppStatus`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`

`AppStatus` enum stays as-is (Setup/Ready/ResourceAdmin).

### OrgRepository trait (`crates/services/src/db/org_repository.rs`)
Added to `DbService` super-trait. Raw DB CRUD:
- `get_organization() -> Result<Option<Organization>, DbError>` - returns the single row; **errors if more than 1 row exists** (invariant for single-tenant)
- `upsert_organization(client_id, client_secret, scope, status) -> Result<Organization, DbError>`
- `update_app_status(status) -> Result<(), DbError>`

Impl on `SqliteDbService` handles encryption/decryption using existing `encrypt_api_key`/`decrypt_api_key` from `crates/services/src/db/encryption.rs` and `self.encryption_key`.

### OrgService trait (`crates/services/src/org_service.rs`)
New standalone service trait with `#[mockall::automock]`:
- `get_organization() -> Result<Option<Organization>>`
- `get_app_status() -> Result<AppStatus>` (returns `AppStatus::Setup` if no row - matches current default behavior)
- `set_app_status(status) -> Result<()>`
- `create_organization(client_id, client_secret, scope) -> Result<Organization>`

`DefaultOrgService` holds `Arc<dyn DbService>`, delegates to `OrgRepository`.

### OrgServiceStub (`crates/services/src/test_utils/org.rs`)
In-memory test stub mirroring `SecretServiceStub` API:
- `with_app_status()`, `with_app_status_ready()`, `with_app_reg_info()`, `with_app_reg_info_default()`
- `Default` impl: status=Ready + default test reg info (matches current `SecretServiceStub::default()`)

## What Gets Removed

| Item | Location |
|------|----------|
| `SecretService` trait | `crates/services/src/secret_service.rs` |
| `DefaultSecretService` impl | same file |
| `SecretServiceExt` trait | `crates/services/src/service_ext.rs` |
| `SecretServiceStub` | `crates/services/src/test_utils/secret.rs` |
| `KeyringStoreStub` | same file |
| `AppRegInfo` struct | `crates/services/src/objs.rs` |
| `AppRegInfoBuilder` | same (derived) |
| `AppRegInfoMissingError` | `crates/objs/src/error/objs.rs` |
| `secret_service()` on `AppService` | `crates/services/src/app_service.rs` |
| `secret_service` field on `DefaultAppService` | same file |
| `secret_service` field on `AppServiceStub` | `crates/services/src/test_utils/app.rs` |
| SecretService construction in `AppServiceBuilder` | `crates/lib_bodhiserver/src/app_service_builder.rs` |
| `secrets_path()` on `SettingService` | `crates/services/src/setting_service/service.rs` |
| `KeyringStore` trait + `SystemKeyringStore` | `crates/services/src/keyring_service.rs` |

## Commit Plan

Single PR, commits follow crate dependency order. Each commit compiles; all tests pass after final commit.

### Commit 1: `objs` - Remove AppRegInfoMissingError
- Remove `AppRegInfoMissingError` from `crates/objs/src/error/objs.rs`
- Remove any re-exports in `crates/objs/src/lib.rs`

### Commit 2: `services` - Add Organization, OrgRepository, OrgService, migration
- Add migration files `0010_organizations.{up,down}.sql`
- Add `Organization` struct to `crates/services/src/objs.rs`
- Add `crates/services/src/db/org_repository.rs` (trait + SqliteDbService impl)
- Add `OrgRepository` to `DbService` super-trait + blanket impl in `crates/services/src/db/service.rs`
- Add `OrgRepository` to `MockDbService` mock! block in `crates/services/src/test_utils/db.rs`
- Add `crates/services/src/org_service.rs` (OrgService trait, OrgServiceError, DefaultOrgService)
- Add `crates/services/src/test_utils/org.rs` (OrgServiceStub)
- Add `org_service()` to `AppService` trait + `DefaultAppService` + `AppServiceStub`
- Wire `OrgServiceStub` default in `AppServiceStubBuilder`
- Add `with_org_service()` builder method
- Keep `secret_service()` temporarily (callers still reference it)

### Commit 3: `auth_middleware` - Migrate from SecretService to OrgService
- `token_service.rs`: Replace `secret_service: Arc<dyn SecretService>` with `org_service: Arc<dyn OrgService>` in `DefaultTokenService`
  - All `self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?` become `self.org_service.get_organization().await?.ok_or(OrgServiceError::NotFound)?`
  - Access `org.client_id`, `org.client_secret` instead of `app_reg_info.client_id`
- `auth_middleware.rs`: Replace `secret_service` with `org_service` in middleware functions
  - `DefaultTokenService::new()` call updated
  - `app_status_or_default(&secret_service)` becomes `org_service.get_app_status().await.unwrap_or_default()`
  - `AuthError` enum: replace `SecretService(#[from] SecretServiceError)` with `OrgService(#[from] OrgServiceError)`
- `utils.rs`: Update `app_status_or_default` signature to take `Arc<dyn OrgService>`, make async, or inline
- Update all auth_middleware tests

### Commit 4: `routes_app` - Migrate from SecretService to OrgService
- `routes_auth/login.rs`: Replace all `secret_service` calls with `org_service`
  - `auth_initiate_handler`: `org_service.get_organization().await?` for client_id
  - `auth_callback_handler`: `org_service.get_organization().await?` for token exchange, `org_service.set_app_status().await?`
- `routes_auth/error.rs`: Replace `SecretServiceError` with `OrgServiceError` in `LoginError`
- `routes_setup/mod.rs`: Setup handler writes via `org_service.create_organization()` and `org_service.set_app_status()`
  - `app_info_handler`: read status via `org_service.get_app_status()`
- `routes_dev.rs`: Replace SecretService usage in dev debug handler
- `test_utils/router.rs`: Replace SecretService setup with OrgService
- Update all routes_app test files

### Commit 5: `services` - Migrate internal callers, remove SecretService
- `access_request_service/service.rs`: Replace `secret_service: Arc<dyn SecretService>` with `org_service: Arc<dyn OrgService>` in `DefaultAccessRequestService`
- Delete `crates/services/src/secret_service.rs`
- Delete `crates/services/src/service_ext.rs`
- Delete `crates/services/src/test_utils/secret.rs`
- Remove `AppRegInfo` + `AppRegInfoBuilder` from `crates/services/src/objs.rs`
- Remove `secret_service()` from `AppService` trait, `DefaultAppService`, `AppServiceStub`
- Remove `with_secret_service()` from `AppServiceStubBuilder`
- Remove `mod secret_service`, `mod service_ext`, `pub use` from `crates/services/src/lib.rs`
- Remove `mod secret` from `crates/services/src/test_utils/mod.rs`
- Consider removing `KeyringStore` trait + `SystemKeyringStore` + `keyring_service.rs` if only used by SecretService
- Consider removing `secrets_path()` from `SettingService`

### Commit 6: `lib_bodhiserver` - Update AppServiceBuilder
- `app_service_builder.rs`:
  - Remove `get_or_build_secret_service()` method
  - Remove SecretService construction (encryption key from keyring, DefaultSecretService::new)
  - Add `DefaultOrgService::new(db_service.clone())` construction
  - Pass `org_service` to `DefaultAppService::new()` instead of `secret_service`
  - Pass `org_service` to `DefaultAccessRequestService::new()` instead of `secret_service`
  - Update `update_with_option()`: `secret_service.set_app_reg_info()` → `org_service.create_organization().await`, `secret_service.set_app_status()` → `org_service.set_app_status().await`
  - `update_with_option` becomes async (called from async `build()` context - straightforward)
- `app_options.rs`: Define local struct or inline fields for startup OAuth config (replaces `AppRegInfo` usage in options)
- Remove keyring-related deps if no longer needed

### Commit 7: `lib_bodhiserver_napi` + `bodhi/src-tauri` - Remove any SecretService references
- Compile check, remove any remaining SecretService imports/usages

### Commit 8: Final cleanup
- Remove any remaining imports, dead code
- Run `cargo check --all` and `make test.backend`

## Key Implementation Details

### Encryption reuse
- `encrypt_api_key(master_key, plaintext) -> (encrypted_b64, salt_b64, nonce_b64)` at `crates/services/src/db/encryption.rs:49`
- `decrypt_api_key(master_key, encrypted, salt, nonce) -> plaintext` at `crates/services/src/db/encryption.rs:68`
- `SqliteDbService.encryption_key` field already exists - same key used for `api_model_aliases` and `toolsets`

### Error code changes
- `AuthError::SecretService` → `AuthError::OrgService` changes error code from `auth_error-secret_service` to `auth_error-org_service`
- `LoginError` variants with `SecretServiceError` similarly change
- Search tests for old error codes and update

### Sync-to-async at `update_with_option`
- Currently sync (SecretService file ops are sync). Becomes async with DB calls.
- Only called from `AppServiceBuilder::build()` which is already `async fn` - clean propagation.

## Tech Debt Item (add to `ai-docs/claude-plans/20260210-access-request/tech-debt.md`)

Add after plan approval:

> **Org Resolution Middleware for Multi-Tenancy**: Currently `get_organization()` returns the single row from the `organizations` table (errors if >1 row). For multi-tenancy, implement an `OrgContext` middleware (similar to `AuthContext`) that:
> - Extracts org identifier from request subdomain (`<org>.getbodhi.app`) or `Host` header
> - Looks up the `organizations` table by slug/identifier
> - Injects `Extension<OrgContext>` into request extensions (containing `org_id`, `client_id`, decrypted `client_secret`, `scope`, `status`)
> - All downstream handlers and services extract `OrgContext` from extensions instead of calling `OrgService.get_organization()`
> - In single-tenant mode, middleware reads the single org row (no subdomain extraction)
> - Cache org lookup results via `CacheService` (Redis in multi-tenant, Moka in single-tenant)
> - The `organizations` table gains `slug TEXT NOT NULL UNIQUE` column for subdomain-based lookup
> - All other tables gain `org_id TEXT NOT NULL DEFAULT 'default'` column for row-level isolation

## Verification

1. `cargo check --all` after each commit
2. `make test.backend` passes after final commit
3. Verify setup flow end-to-end: `POST /bodhi/v1/app/setup` creates org row in DB
4. Verify login flow: auth handlers read client_id/client_secret from DB
5. Verify `client_secret` is encrypted in DB (not plaintext)
6. Verify app_status defaults to `setup` when no org row exists
