# Move AppRegInfo & AppStatus from secrets.yaml to Database

## Context

AppRegInfo (OAuth client credentials: `client_id`, `client_secret`, `scope`) and AppStatus (app lifecycle: Setup/Ready/ResourceAdmin) are currently stored in an AES-256-GCM encrypted file (`secrets.yaml`) managed by `SecretService`. This file-based approach was designed for single-tenant deployments.

As a step toward multi-tenancy (where multiple orgs will each have their own OAuth registration), we're moving this data into the SQLite database as an `organizations` table. This also lets us fully remove the `SecretService` abstraction (trait + impl + test stubs), simplifying the service layer.

**Migration goals**:
- Move OAuth credentials from encrypted file to encrypted database columns
- Move app status from encrypted file to database column
- Remove `SecretService` completely (file handling, keyring integration, all dependencies)
- Enable future multi-tenant org resolution (tracked separately as tech debt)

**No backwards compatibility**: No production release yet, so clean cut implementation — no data migration from existing secrets.yaml, no fallback reads.

---

## Current Implementation (to be replaced)

### Storage (`SecretService`)
**File**: `crates/services/src/secret_service.rs`
- `SecretService` trait: low-level key-value storage (`get_secret_string`, `set_secret_string`, `delete_secret`)
- `DefaultSecretService`: AES-256-GCM encrypted file at `{BODHI_HOME}/secrets.yaml`
  - Encryption key derived from OS keyring via `SystemKeyringStore` or `BODHI_ENCRYPTION_KEY` env var
  - PBKDF2 key derivation (1000 iterations), file locking (fs2)
  - EncryptedData struct: `{ salt, nonce, data }` serialized as YAML

**File**: `crates/services/src/service_ext.rs`
- `SecretServiceExt` trait: domain-specific accessors wrapping `SecretService`
- `app_reg_info() -> Result<Option<AppRegInfo>>` — reads `"app_reg_info"` key, deserializes YAML
- `set_app_reg_info(app_reg_info: &AppRegInfo)` — serializes to YAML, writes to `"app_reg_info"` key
- `app_status() -> Result<AppStatus>` — reads `"app_status"` key, returns `AppStatus::default()` (Setup) if missing
- `set_app_status(app_status: &AppStatus)`

**File**: `crates/services/src/test_utils/secret.rs`
- `SecretServiceStub`: In-memory HashMap for tests
- Builder methods: `with_app_status()`, `with_app_status_ready()`, `with_app_reg_info()`, `with_app_reg_info_default()`
- `Default` impl: status=Ready + default test reg info

**File**: `crates/services/src/keyring_service.rs`
- `KeyringStore` trait: OS credential storage abstraction
- `SystemKeyringStore`: Uses keyring crate (macOS Keychain, Linux Secret Service, Windows Credential Manager)
- Used only to get/set encryption key for `DefaultSecretService`

### Domain Types (`services::objs`)
**File**: `crates/services/src/objs.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder)]
pub struct AppRegInfo {
  pub client_id: String,
  pub client_secret: String,
  #[serde(default)]
  pub scope: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, strum::Display, Clone, Default, strum::EnumString, ToSchema)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AppStatus {
  #[default]
  Setup,
  Ready,
  ResourceAdmin,
}
```

### Error Types
**File**: `crates/objs/src/error/objs.rs`
```rust
#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("Application registration information is missing.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct AppRegInfoMissingError;
```

### Service Integration (`AppService`)
**File**: `crates/services/src/app_service.rs`
- `AppService` trait has `fn secret_service(&self) -> Arc<dyn SecretService>`
- `DefaultAppService` holds `secret_service: Arc<dyn SecretService>` field
- `AppServiceStub` (test utils) has `secret_service` field with `default_secret_service()` builder

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`
- Constructs `DefaultSecretService` with:
  - `encryption_key` from `get_or_build_encryption_key()` (reads `BODHI_ENCRYPTION_KEY` env var or OS keyring)
  - `secrets_path` from `setting_service.secrets_path()` (= `{BODHI_HOME}/secrets.yaml`)
- Passes `Arc<dyn SecretService>` to `DefaultAppService::new()` and `DefaultAccessRequestService::new()`
- `update_with_option()` function (sync) writes `AppRegInfo` + `AppStatus` via `SecretServiceExt`

---

## Callers of SecretService (to migrate to OrgService)

| Crate | File | Usage |
|-------|------|-------|
| auth_middleware | `token_service.rs` | `secret_service.app_reg_info()` — 5 call sites for token exchange, validation, refresh |
| auth_middleware | `auth_middleware.rs` | `app_status_or_default(&secret_service)` for middleware gating |
| routes_app | `routes_auth/login.rs` | `secret_service.app_reg_info()` (2 call sites), `secret_service.app_status()` (1), `secret_service.set_app_status()` (2) |
| routes_app | `routes_setup/mod.rs` | `secret_service.app_reg_info()`, `secret_service.set_app_reg_info()`, `secret_service.app_status()`, `secret_service.set_app_status()` |
| routes_app | `routes_dev.rs` | `secret_service.app_reg_info()`, `secret_service.app_status()`, `secret_service.dump()` (dev debug) |
| routes_app | `routes_auth/error.rs` | `LoginError::SecretServiceError(#[from] SecretServiceError)` variant |
| services | `access_request_service/service.rs` | `secret_service.app_reg_info()` for auto-approve flow |

**Total**: ~15 call sites across 3 crates, all `.app_reg_info()` or `.app_status()` calls. **All become async DB calls.**

---

## New Implementation Design

### Migration SQL
**File**: `crates/services/migrations/0010_organizations.up.sql`
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

**File**: `crates/services/migrations/0010_organizations.down.sql`
```sql
DROP TABLE IF EXISTS organizations;
```

**Design notes**:
- Single row in practice for single-tenant. `get_organization()` **errors if >1 row** (enforces single-tenant invariant).
- `client_secret` encrypted per-column (separate salt+nonce) using existing `encrypt_api_key` / `decrypt_api_key` from `crates/services/src/db/encryption.rs`.
- Suffix naming: `client_secret_encrypted`, `client_secret_salt`, `client_secret_nonce` (not generic `encrypted_data`/`salt`/`nonce` — supports future encrypted fields in same row).
- `app_status` stores `"setup"` / `"ready"` / `"resource-admin"` (kebab-case via `strum::Display`).
- Timestamps use Unix epoch integers (consistent with all other tables).

### New Domain Type (`Organization`)
**File**: `crates/services/src/objs.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Organization {
    pub id: i64,
    pub client_id: String,
    pub client_secret: String,  // Decrypted in memory
    pub scope: String,
    pub app_status: AppStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Replaces `AppRegInfo`. `AppStatus` enum stays as-is.

### OrgRepository Trait (raw DB CRUD)
**File**: `crates/services/src/db/org_repository.rs` (NEW)
```rust
use crate::db::DbError;
use crate::Organization;

#[async_trait::async_trait]
pub trait OrgRepository: Send + Sync {
    async fn get_organization(&self) -> Result<Option<Organization>, DbError>;
    async fn upsert_organization(&self, org: &Organization) -> Result<Organization, DbError>;
    async fn update_app_status(&self, status: &AppStatus) -> Result<(), DbError>;
}
```

Added to `DbService` super-trait in `crates/services/src/db/service.rs`:
```rust
pub trait DbService:
  ModelRepository
  + AccessRepository
  + AccessRequestRepository
  + TokenRepository
  + ToolsetRepository
  + UserAliasRepository
  + OrgRepository  // <-- NEW
  + DbCore
  + Send + Sync + Debug {}
```

**SqliteDbService implementation**:
- `get_organization()`: SELECT single row, decrypt `client_secret`, parse `app_status`, return `None` if empty. **Error if COUNT(*) > 1** (single-tenant invariant).
- `upsert_organization()`: Check if row exists (SELECT COUNT). If 0, INSERT. If 1, UPDATE. If >1, error. Encrypt `client_secret` via `encrypt_api_key(self.encryption_key, client_secret)` before write.
- `update_app_status()`: UPDATE single row. If no rows affected, INSERT placeholder row with status (or error).

Encryption reuses existing helpers:
- `encrypt_api_key(master_key: &[u8], plaintext: &str) -> Result<(String, String, String)>` at `crates/services/src/db/encryption.rs:49`
- `decrypt_api_key(master_key: &[u8], encrypted: &str, salt: &str, nonce: &str) -> Result<String>` at `crates/services/src/db/encryption.rs:68`
- `SqliteDbService.encryption_key: Vec<u8>` field already exists (same key used for `api_model_aliases`, `toolsets`).

**MockDbService**: Add `OrgRepository` to `mock!` block in `crates/services/src/test_utils/db.rs`.

### OrgService Trait (business logic layer)
**File**: `crates/services/src/org_service.rs` (NEW)
```rust
use crate::{AppStatus, Organization};
use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum OrgServiceError {
    #[error("Organization not found.")]
    #[error_meta(error_type = ErrorType::InvalidAppState)]
    NotFound,
    #[error(transparent)]
    DbError(#[from] crate::db::DbError),
    #[error(transparent)]
    EncryptionError(#[from] crate::db::encryption::EncryptionError),
}

type Result<T> = std::result::Result<T, OrgServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait OrgService: Send + Sync + std::fmt::Debug {
    async fn get_organization(&self) -> Result<Option<Organization>>;
    async fn get_app_status(&self) -> Result<AppStatus>;
    async fn set_app_status(&self, status: &AppStatus) -> Result<()>;
    async fn create_organization(&self, client_id: &str, client_secret: &str, scope: &str) -> Result<Organization>;
}

#[derive(Debug)]
pub struct DefaultOrgService {
    db_service: Arc<dyn DbService>,
}

impl DefaultOrgService {
    pub fn new(db_service: Arc<dyn DbService>) -> Self {
        Self { db_service }
    }
}

#[async_trait::async_trait]
impl OrgService for DefaultOrgService {
    async fn get_organization(&self) -> Result<Option<Organization>> {
        Ok(self.db_service.get_organization().await?)
    }

    async fn get_app_status(&self) -> Result<AppStatus> {
        // Matches current behavior: returns Setup if no org exists
        match self.db_service.get_organization().await? {
            Some(org) => Ok(org.app_status),
            None => Ok(AppStatus::default()), // AppStatus::Setup
        }
    }

    async fn set_app_status(&self, status: &AppStatus) -> Result<()> {
        self.db_service.update_app_status(status).await?;
        Ok(())
    }

    async fn create_organization(&self, client_id: &str, client_secret: &str, scope: &str) -> Result<Organization> {
        let now = Utc::now();
        let org = Organization {
            id: 0, // DB auto-generates
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            scope: scope.to_string(),
            app_status: AppStatus::Setup,
            created_at: now,
            updated_at: now,
        };
        Ok(self.db_service.upsert_organization(&org).await?)
    }
}
```

### Test Utility: OrgServiceStub
**File**: `crates/services/src/test_utils/org.rs` (NEW)
Mirrors `SecretServiceStub` API for easy test migration:
```rust
#[derive(Debug)]
pub struct OrgServiceStub {
    organization: Mutex<Option<Organization>>,
}

impl OrgServiceStub {
    pub fn new() -> Self { ... }
    pub fn with_app_status(self, status: AppStatus) -> Self { ... }
    pub fn with_app_status_ready(self) -> Self { self.with_app_status(AppStatus::Ready) }
    pub fn with_app_reg_info(self, client_id: &str, client_secret: &str, scope: &str) -> Self { ... }
    pub fn with_app_reg_info_default(self) -> Self {
        // Uses TEST_CLIENT_ID, TEST_CLIENT_SECRET, computed scope
        self.with_app_reg_info("test-client", "test-secret", "scope_test-client")
    }
}

impl Default for OrgServiceStub {
    fn default() -> Self {
        Self::new().with_app_status_ready().with_app_reg_info_default()
    }
}

#[async_trait::async_trait]
impl OrgService for OrgServiceStub {
    async fn get_organization(&self) -> Result<Option<Organization>> { ... }
    async fn get_app_status(&self) -> Result<AppStatus> { ... }
    async fn set_app_status(&self, status: &AppStatus) -> Result<()> { ... }
    async fn create_organization(&self, client_id: &str, client_secret: &str, scope: &str) -> Result<Organization> { ... }
}
```

### Integration into AppService
**File**: `crates/services/src/app_service.rs`
- Add `fn org_service(&self) -> Arc<dyn OrgService>` to `AppService` trait
- Add `org_service: Arc<dyn OrgService>` field to `DefaultAppService`
- Remove `fn secret_service()` and `secret_service` field (done in later commit)

**File**: `crates/services/src/test_utils/app.rs`
- Add `org_service: Option<Arc<dyn OrgService>>` field to `AppServiceStub` / `AppServiceStubBuilder`
- Add `default_org_service()` builder: returns `Arc::new(OrgServiceStub::default())`
- Add `with_org_service()` builder method
- Remove `secret_service` field (done in later commit)

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`
- Construct `DefaultOrgService::new(db_service.clone())`
- Pass to `DefaultAppService::new()` (replaces `secret_service` arg)
- Pass to `DefaultAccessRequestService::new()` (replaces `secret_service` arg)
- `update_with_option()`: Replace `service.secret_service().set_app_reg_info()` with `service.org_service().create_organization().await`
  - **Make `update_with_option` async** (straightforward — called from async `build()` context)
- Remove SecretService construction (later commit)

---

## Error Code Changes

| Old Enum Variant | Old Code | New Enum Variant | New Code |
|------------------|----------|------------------|----------|
| `AuthError::SecretService(SecretServiceError)` | `auth_error-secret_service` | `AuthError::OrgService(OrgServiceError)` | `auth_error-org_service` |
| `LoginError::SecretServiceError(SecretServiceError)` | `login_error-secret_service_error` | `LoginError::OrgServiceError(OrgServiceError)` | `login_error-org_service_error` |
| `AppRegInfoMissingError` | `app_reg_info_missing_error` | `OrgServiceError::NotFound` | `org_service_error-not_found` |

**Test migration impact**: All error code assertions in tests need updating (search for old codes via grep).

---

## Sync-to-Async Migration Impact

| Service | Current | New | Call Sites |
|---------|---------|-----|------------|
| `DefaultTokenService` | `secret_service.app_reg_info()` (sync) | `org_service.get_organization().await` (async) | Already async methods — add `.await` |
| `auth_middleware` | `app_status_or_default(&secret_service)` (sync) | `org_service.get_app_status().await` (async) | Already async middleware — add `.await` |
| `DefaultAccessRequestService` | `secret_service.app_reg_info()` (sync) | `org_service.get_organization().await` (async) | Already async methods — add `.await` |
| `setup_handler` | `secret_service.set_app_reg_info(...)` (sync) | `org_service.create_organization(...).await` (async) | Already async handler — add `.await` |
| `update_with_option` | Sync function | **Make async** | Called from async `AppServiceBuilder::build()` — clean propagation |

**No blocking issues**: All callers are already in async contexts (async route handlers, async middleware, async builder methods).

---

## Phased Commit Plan

Single PR, commits follow crate dependency order. Each commit compiles; all tests pass after final commit.

### Commit 1: `objs` — Remove AppRegInfoMissingError
- Remove `AppRegInfoMissingError` from `crates/objs/src/error/objs.rs`
- Remove re-exports in `crates/objs/src/lib.rs`

### Commit 2: `services` — Add Organization, OrgRepository, OrgService
- Add migration files `migrations/0010_organizations.{up,down}.sql`
- Add `Organization` struct to `crates/services/src/objs.rs` (keep `AppRegInfo` temporarily)
- Add `crates/services/src/db/org_repository.rs` (trait + SqliteDbService impl)
- Add `OrgRepository` to `DbService` super-trait + blanket impl in `crates/services/src/db/service.rs`
- Add `OrgRepository` to `MockDbService` mock! block in `crates/services/src/test_utils/db.rs`
- Add `crates/services/src/org_service.rs` (OrgService trait, OrgServiceError, DefaultOrgService)
- Add `crates/services/src/test_utils/org.rs` (OrgServiceStub)
- Add `org_service()` to `AppService` trait + `DefaultAppService` + `AppServiceStub`
- Wire `OrgServiceStub` default in `AppServiceStubBuilder::default_org_service()`
- Add `with_org_service()` builder method
- **Keep `secret_service()` temporarily** (callers still reference it)

### Commit 3: `auth_middleware` — Migrate from SecretService to OrgService
- `token_service.rs`: Replace `secret_service: Arc<dyn SecretService>` with `org_service: Arc<dyn OrgService>` in `DefaultTokenService`
  - All `self.secret_service.app_reg_info()?.ok_or(AppRegInfoMissingError)?` become `self.org_service.get_organization().await?.ok_or(OrgServiceError::NotFound)?`
  - Access `org.client_id`, `org.client_secret` instead of `app_reg_info.client_id`
- `auth_middleware.rs`: Replace `secret_service` with `org_service` in middleware functions
  - `DefaultTokenService::new()` call updated
  - `app_status_or_default(&secret_service)` becomes `org_service.get_app_status().await.unwrap_or_default()`
  - `AuthError` enum: replace `SecretService(#[from] SecretServiceError)` with `OrgService(#[from] OrgServiceError)`
- `utils.rs`: Update `app_status_or_default` signature to take `Arc<dyn OrgService>`, make async
- Update all auth_middleware tests (replace `SecretServiceStub` with `OrgServiceStub` or `MockOrgService`)

### Commit 4: `routes_app` — Migrate from SecretService to OrgService
- `routes_auth/login.rs`: Replace all `secret_service` calls with `org_service`
  - `auth_initiate_handler`: `org_service.get_organization().await?` for client_id
  - `auth_callback_handler`: `org_service.get_organization().await?` for token exchange, `org_service.set_app_status().await?`
- `routes_auth/error.rs`: Replace `SecretServiceError` with `OrgServiceError` in `LoginError`
- `routes_setup/mod.rs`: Setup handler writes via `org_service.create_organization()` and `org_service.set_app_status()`
  - `app_info_handler`: read status via `org_service.get_app_status()`
- `routes_dev.rs`: Replace SecretService usage in dev debug handler
- `test_utils/router.rs`: Replace SecretService setup with OrgService
- Update all routes_app test files (replace `SecretServiceStub` with `OrgServiceStub`)

### Commit 5: `services` — Migrate internal callers, remove SecretService
- `access_request_service/service.rs`: Replace `secret_service: Arc<dyn SecretService>` with `org_service: Arc<dyn OrgService>` in `DefaultAccessRequestService`
- **DELETE** `crates/services/src/secret_service.rs`
- **DELETE** `crates/services/src/service_ext.rs`
- **DELETE** `crates/services/src/test_utils/secret.rs`
- **DELETE** `crates/services/src/keyring_service.rs` (only used by SecretService)
- Remove `AppRegInfo` + `AppRegInfoBuilder` from `crates/services/src/objs.rs`
- Remove `secret_service()` from `AppService` trait, `DefaultAppService`, `AppServiceStub`
- Remove `with_secret_service()` from `AppServiceStubBuilder`
- Remove `mod secret_service`, `mod service_ext`, `mod keyring_service`, `pub use` from `crates/services/src/lib.rs`
- Remove `mod secret` from `crates/services/src/test_utils/mod.rs`

### Commit 6: `lib_bodhiserver` — Update AppServiceBuilder
- `app_service_builder.rs`:
  - Remove `get_or_build_secret_service()` method
  - Remove `get_or_build_encryption_key()` method (keyring logic no longer needed — encryption key is for DB only, not file-based secrets)
  - Remove SecretService construction
  - Add `DefaultOrgService::new(db_service.clone())` construction
  - Pass `org_service` to `DefaultAppService::new()` instead of `secret_service`
  - Pass `org_service` to `DefaultAccessRequestService::new()` instead of `secret_service`
  - Update `update_with_option()`: `secret_service.set_app_reg_info()` → `org_service.create_organization().await`, `secret_service.set_app_status()` → `org_service.set_app_status().await`
  - **Make `update_with_option` async** (called from async `build()` context — straightforward)
- `app_options.rs`: Define local struct or inline fields for startup OAuth config (replaces `AppRegInfo` usage in options)
- Remove keyring-related deps from Cargo.toml if no longer needed

### Commit 7: `lib_bodhiserver_napi` + `bodhi/src-tauri` — Remove any SecretService references
- Compile check, remove any remaining SecretService imports/usages

### Commit 8: Final cleanup
- Remove any remaining imports, dead code
- Remove `secrets_path()` from `SettingService` trait if only used for SecretService
- Run `cargo check --all` and `make test.backend`

---

## Tech Debt Item (for multi-tenancy)

After this migration, `get_organization()` returns the single row from the `organizations` table (errors if >1 row). For multi-tenancy, implement an **OrgContext middleware** (similar to `AuthContext`):

**Org Resolution Middleware**:
- Extracts org identifier from request subdomain (`<org>.getbodhi.app`) or `Host` header
- Looks up the `organizations` table by slug/identifier
- Injects `Extension<OrgContext>` into request extensions (containing `org_id`, `client_id`, decrypted `client_secret`, `scope`, `status`)
- All downstream handlers and services extract `OrgContext` from extensions instead of calling `OrgService.get_organization()`
- In single-tenant mode, middleware reads the single org row (no subdomain extraction)
- Cache org lookup results via `CacheService` (Redis in multi-tenant, Moka in single-tenant)

**Schema changes for multi-tenancy** (not in this PR):
- `organizations` table gains `slug TEXT NOT NULL UNIQUE` column for subdomain-based lookup
- All other tables gain `org_id TEXT NOT NULL DEFAULT 'default'` column for row-level isolation
- Unique constraints become org-scoped (e.g., `UNIQUE(prefix)` → `UNIQUE(org_id, prefix)`)

---

## Key Files Summary

| Phase | File | Action |
|-------|------|--------|
| 1 | `crates/objs/src/error/objs.rs` | Remove `AppRegInfoMissingError` |
| 2 | `crates/services/migrations/0010_organizations.{up,down}.sql` | **NEW** — Migration SQL |
| 2 | `crates/services/src/objs.rs` | Add `Organization` struct, keep `AppRegInfo` temporarily |
| 2 | `crates/services/src/db/org_repository.rs` | **NEW** — OrgRepository trait + SqliteDbService impl |
| 2 | `crates/services/src/db/service.rs` | Add `OrgRepository` to `DbService` super-trait |
| 2 | `crates/services/src/org_service.rs` | **NEW** — OrgService trait, OrgServiceError, DefaultOrgService |
| 2 | `crates/services/src/test_utils/org.rs` | **NEW** — OrgServiceStub |
| 2 | `crates/services/src/app_service.rs` | Add `org_service()` to `AppService` trait + `DefaultAppService` |
| 2 | `crates/services/src/test_utils/app.rs` | Add `org_service` to `AppServiceStub` / `AppServiceStubBuilder` |
| 3 | `crates/auth_middleware/src/token_service.rs` | Replace `secret_service` with `org_service` |
| 3 | `crates/auth_middleware/src/auth_middleware.rs` | Replace `secret_service` with `org_service`, update `AuthError` |
| 3 | `crates/auth_middleware/src/utils.rs` | Update `app_status_or_default` signature |
| 4 | `crates/routes_app/src/routes_auth/login.rs` | Replace `secret_service` with `org_service` |
| 4 | `crates/routes_app/src/routes_auth/error.rs` | Replace `SecretServiceError` with `OrgServiceError` |
| 4 | `crates/routes_app/src/routes_setup/mod.rs` | Replace `secret_service` with `org_service` |
| 4 | `crates/routes_app/src/routes_dev.rs` | Replace SecretService usage |
| 4 | `crates/routes_app/src/test_utils/router.rs` | Replace SecretService setup |
| 5 | `crates/services/src/access_request_service/service.rs` | Replace `secret_service` with `org_service` |
| 5 | `crates/services/src/secret_service.rs` | **DELETE** |
| 5 | `crates/services/src/service_ext.rs` | **DELETE** |
| 5 | `crates/services/src/test_utils/secret.rs` | **DELETE** |
| 5 | `crates/services/src/keyring_service.rs` | **DELETE** |
| 5 | `crates/services/src/objs.rs` | Remove `AppRegInfo` + `AppRegInfoBuilder` |
| 5 | `crates/services/src/app_service.rs` | Remove `secret_service()` method + field |
| 6 | `crates/lib_bodhiserver/src/app_service_builder.rs` | Remove SecretService construction, add OrgService, make `update_with_option` async |
| 6 | `crates/lib_bodhiserver/src/app_options.rs` | Replace `AppRegInfo` usage |

---

## Verification Checklist

1. **Compilation**: `cargo check --all` after each commit
2. **Tests**: `make test.backend` passes after final commit
3. **Setup flow**: `POST /bodhi/v1/app/setup` creates org row in DB with encrypted `client_secret`
4. **Login flow**: auth handlers read `client_id`/`client_secret` from DB
5. **Encryption**: `client_secret` is encrypted in DB (verify via SQLite browser or `SELECT` query)
6. **Status default**: `get_app_status()` returns `AppStatus::Setup` when no org row exists
7. **Single-tenant invariant**: `get_organization()` errors if >1 row in table
