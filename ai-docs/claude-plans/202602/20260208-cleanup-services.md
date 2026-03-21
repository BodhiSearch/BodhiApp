# Commands & Services Layer Consolidation Plan

## Context

The routes layer was reorganized into domain-coherent folders. This plan applies the same rigor to `services` and `commands`: split oversized files, consolidate error types (one enum per service, fold standalone structs), eliminate the legacy `commands` crate, and decompose the 3300-line DbService into domain-specific repository traits.

## Summary of Changes

| Phase | What | Risk | Scope |
|-------|------|------|-------|
| 1 | Split ToolService + SettingService into sub-modules, delete empty obj_exts | Low | services internal |
| 2 | Error consolidation: fold standalone error structs into parent enums, merge HubApiError | Medium | services + all consumers |
| 3 | Delete commands crate, inline logic into routes_app handlers | Medium | commands, routes_app, workspace |
| 4 | Split DbService into 5 repository sub-traits | High | services, all consumers |

---

## Phase 1: Internal Services File Reorganization

### 1A: Split `tool_service.rs` (2061 lines) → `tool_service/` directory

Rust forbids splitting `impl Trait for Type` across files. The split separates error types, interface, implementation, and tests.

**New structure:**
```
crates/services/src/tool_service/
  mod.rs       — re-exports, sub-module declarations
  error.rs     — ToolsetError enum
  service.rs   — ToolService trait, DefaultToolService struct + impl
  tests.rs     — #[cfg(test)] module (~1060 lines)
```

**File contents:**

`mod.rs`:
- `mod error; pub use error::*;`
- `mod service; pub use service::*;`
- `#[cfg(test)] mod tests;`

`error.rs`:
- Lines 1-14: imports
- Lines 16-64: `ToolsetError` enum with errmeta_derive

`service.rs`:
- Lines 66-186: `ToolService` trait definition with `#[mockall::automock]`
- Lines 188-999: `DefaultToolService` struct, `new()`, helpers, builtin definitions, full `impl ToolService for DefaultToolService` block

`tests.rs`:
- Lines ~1001-2061: all `#[cfg(test)]` code

### 1B: Split `setting_service.rs` (2061 lines) → `setting_service/` directory

**New structure:**
```
crates/services/src/setting_service/
  mod.rs       — constants (BODHI_*, DEFAULT_*), SettingsChangeListener trait, re-exports
  error.rs     — SettingServiceError enum + impl_error_from! macros
  service.rs   — SettingService trait + DefaultSettingService impl
  tests.rs     — #[cfg(test)] module (~1136 lines)
```

**File contents:**

`mod.rs`:
- Lines 15-99: all `pub const` declarations (~35 constants)
- Lines 101-126: `SettingsChangeListener` trait + Arc impl
- `mod error; pub use error::*;`
- `mod service; pub use service::*;`
- `#[cfg(test)] mod tests;`

`error.rs`:
- Lines 128-150: `SettingServiceError` enum + `impl_error_from!` + `type Result<T>`

`service.rs`:
- Lines 152-531: `SettingService` trait definition (with all default methods)
- Lines 533-923: `DefaultSettingService` struct + all impl blocks + `asref_impl!`

`tests.rs`:
- Lines ~925-2061: all test code

### 1C: Delete empty `obj_exts` module

- Delete `crates/services/src/obj_exts/mod.rs`
- Remove `mod obj_exts;` and its re-export comment from `crates/services/src/lib.rs`

### Phase 1 Verification

```bash
cargo check -p services && cargo test -p services
```

### Phase 1 Risks

- Test files need `use super::*` or explicit imports — verify all test imports resolve
- `asref_impl!` in SettingService may need to be co-located with the struct — check if it compiles from `service.rs`

---

## Phase 2: Services Error Consolidation

Goal: Each service has exactly ONE error enum. Fold standalone error structs into their parent enum. Merge multi-level error types into single enums.

### Current Error Landscape (problems)

| Problem | Current | Target |
|---------|---------|--------|
| **4 error types in hub_service.rs** | `HubApiError` (6 variants) + `HubServiceError` (4 variants wrapping HubApiError) + `HubFileNotFoundError` (struct) + `RemoteModelNotFoundError` (struct) | Single `HubServiceError` with ~12 variants |
| **4 error types in data_service.rs** | `DataServiceError` (7 variants) + `AliasExistsError` (struct) + `AliasNotFoundError` (struct) + `DataFileNotFoundError` (struct) | Single `DataServiceError` with ~9 variants |
| **2 error types in token.rs** | `TokenError` (6 variants) + `JsonWebTokenError` (struct) | Single `TokenError` with ~7 variants |
| **ItemNotFound in db/error.rs** | Standalone struct used by 1 consumer | Fold into `DbError::ItemNotFound` variant |

### 2A: Merge hub_service.rs errors → single `HubServiceError`

**Before (4 types):**
```
HubFileNotFoundError (struct)         → consumed by routes_app/pull.rs (never constructed by routes)
RemoteModelNotFoundError (struct)     → consumed by routes_app/pull.rs (constructed directly)
HubApiError (enum, 6 variants)       → internal only, not imported by consumers
HubServiceError (enum, 4 variants)   → wraps the above three + ObjValidation + IoError
```

**After (1 type):**
```rust
pub enum HubServiceError {
  // From HubApiError (flattened)
  GatedAccess { repo: String, error: String },
  MayNotExist { repo: String, error: String },
  RepoDisabled { repo: String, error: String },
  Transport { repo: String, error: String },
  UnknownApi { repo: String, error: String },
  Request { repo: String, error: String },
  // From HubFileNotFoundError (folded)
  FileNotFound { filename: String, repo: String, snapshot: String },
  // From RemoteModelNotFoundError (folded)
  RemoteModelNotFound(String),
  // Existing
  ObjValidation(#[from] ObjValidationError),
  IoError(#[from] IoError),
}
```

**Consumer updates:**
- `routes_app/src/routes_models/pull.rs:248`: `RemoteModelNotFoundError::new(alias)` → `HubServiceError::RemoteModelNotFound(alias)`
- Internal hub_service.rs: Replace all `HubApiError::*` constructions with `HubServiceError::*`
- Tests in hub_service.rs: Update pattern matches from `HubServiceError::HubApiError(HubApiError::MayNotExist{..})` → `HubServiceError::MayNotExist{..}`

**Error code changes:**
- `hub_file_not_found_error` → `hub_service_error-file_not_found`
- `remote_model_not_found_error` → `hub_service_error-remote_model_not_found`
- `hub_api_error-gated_access` → `hub_service_error-gated_access`
- Search tests for old codes and update

### 2B: Fold data_service.rs standalone structs → `DataServiceError`

**Before (4 types):**
```
AliasExistsError(String)              → consumed by routes_app (types.rs via commands, being removed in Phase 3)
AliasNotFoundError(String)            → consumed by routes_app (aliases.rs, types.rs), routes_oai (routes_oai_models.rs)
DataFileNotFoundError { filename, dirname } → not consumed by routes (internal to services)
DataServiceError (enum, 7 variants)   → wraps the above three
```

**After (1 type):**
```rust
pub enum DataServiceError {
  AliasExists(String),                    // was AliasExistsError
  AliasNotFound(String),                  // was AliasNotFoundError
  FileNotFound { filename: String, dirname: String },  // was DataFileNotFoundError
  Io(#[from] IoError),
  SerdeYaml(#[from] SerdeYamlError),
  HubService(#[from] HubServiceError),
  Db(#[from] DbError),
}
```

**Consumer updates:**
- `routes_app/src/routes_models/aliases.rs:361,417,461`: `AliasNotFoundError(alias)` → `DataServiceError::AliasNotFound(alias)`
- `routes_app/src/routes_models/error.rs:3,17`: `AliasNotFoundError` import + `#[from]` → `DataServiceError`
- `routes_oai/src/routes_oai_models.rs:165`: `AliasNotFoundError(id)` → `DataServiceError::AliasNotFound(id)`

**Error code changes:**
- `alias_exists_error` → `data_service_error-alias_exists`
- `alias_not_found_error` → `data_service_error-alias_not_found`
- `data_file_not_found_error` → `data_service_error-file_not_found`

### 2C: Fold token.rs `JsonWebTokenError` → `TokenError`

**Before (2 types):**
```
JsonWebTokenError (struct)  → consumed by routes_app/shared/error.rs as #[from]
TokenError (enum, 6 variants) → consumed by auth_middleware
```

**After (1 type):**
```rust
pub enum TokenError {
  JsonWebToken(jsonwebtoken::errors::Error),  // was JsonWebTokenError
  InvalidToken(String),
  SerdeJson(#[from] SerdeJsonError),
  InvalidIssuer(String),
  ScopeEmpty,
  Expired,
  InvalidAudience(String),
}
```

Note: The `JsonWebTokenError` struct had a custom `code()` method that matched on `ErrorKind` variants. This logic moves to the `errmeta_derive` annotation or a manual `AppError` impl on the `JsonWebToken` variant.

**Consumer updates:**
- `routes_app/src/routes_auth/error.rs:31`: `JsonWebToken(#[from] JsonWebTokenError)` → `TokenError(#[from] TokenError)` (check for variant name conflicts in the host error enum)

**Error code changes:**
- `json_web_token_error` (struct code) → `token_error-json_web_token` (variant code)
- The custom `code()` method on `JsonWebTokenError` that matched `ErrorKind` variants needs special handling — may require manual `AppError` impl on `TokenError` or `args_delegate=false`

### 2D: Fold db `ItemNotFound` → `DbError`

**Before:** `ItemNotFound` standalone struct in `db/error.rs`

**After:** `DbError::ItemNotFound { id: String, item_type: String }` variant

**Consumer updates:**
- `routes_app/src/routes_models/pull.rs:356`: `ItemNotFound::new(id, "download_requests")` → `DbError::ItemNotFound { id, item_type: "download_requests".to_string() }`

**Error code changes:**
- `item_not_found` → `db_error-item_not_found`

### 2E: Keep intermediate wrapper structs

`SqlxError` and `SqlxMigrateError` in `db/error.rs` are NOT folded — they're intermediate types required by the `impl_error_from!` pattern to bridge the orphan rule. They stay as implementation details.

`EncryptionError` in `db/encryption.rs` stays as-is — both `SecretServiceError` and `DbError` convert it to String, neither wraps it as `#[from]`.

### 2F: Update lib.rs re-exports

Remove re-exports of deleted standalone types:
- Remove `pub use data_service::AliasExistsError;` (if explicitly re-exported)
- Remove `pub use data_service::AliasNotFoundError;`
- Remove `pub use data_service::DataFileNotFoundError;`
- Remove `pub use hub_service::HubFileNotFoundError;`
- Remove `pub use hub_service::RemoteModelNotFoundError;`
- Remove `pub use hub_service::HubApiError;`
- Remove `pub use token::JsonWebTokenError;`
- Remove `pub use db::ItemNotFound;`

Note: Since lib.rs uses `pub use module::*;`, the types are automatically un-exported when they're removed from the source. No explicit removal needed — just verify they don't appear in the `*` glob anymore.

### Phase 2 Verification

```bash
cargo check -p services
cargo check -p routes_app
cargo check -p routes_oai
cargo check -p auth_middleware
make test.backend
```

### Phase 2 Risks

- **Error code changes**: Folding changes auto-generated codes (e.g., `alias_not_found_error` → `data_service_error-alias_not_found`). Search ALL test files for old codes.
- **PartialEq loss**: Standalone structs like `AliasExistsError` had `PartialEq`. Parent enums (DataServiceError) can't derive `PartialEq` because they wrap non-PartialEq types (IoError, HubServiceError). Tests using `assert_eq!` on these errors must change to pattern matching.
- **JsonWebTokenError custom code()**: This struct has a custom `code()` method matching `jsonwebtoken::ErrorKind`. After folding into `TokenError::JsonWebToken`, this custom logic needs manual `AppError` impl or `args_delegate=false` on the variant.
- **Routes_oai also imports AliasNotFoundError**: `routes_oai/src/routes_oai_models.rs:165` constructs it directly. Must update.

---

## Phase 3: Commands Crate Deletion

The CLI is gone. `commands` crate is legacy (608 lines, 2 commands). Only consumed by `routes_app/src/routes_models/` (3 files). `server_app` and `lib_bodhiserver` have phantom Cargo.toml deps with zero usage.

### 3A: Inline CreateCommand into `routes_app/src/routes_models/aliases.rs`

**What CreateCommand.execute() does** (cmd_create.rs:48-111):
1. Check alias exists via `data_service.find_user_alias()`
2. If exists and not update mode, return error (now `DataServiceError::AliasExists`)
3. Check local file via `hub_service.local_file_exists()`
4. If missing and auto_download, download; else error
5. Build `UserAlias` via `UserAliasBuilder`
6. Save via `data_service.save_alias()`

**Action**: Create private async fn `execute_create_alias(...)` in `aliases.rs`. Replace `CreateCommand::new(...).execute(...)` calls in `create_alias_handler` (line 402) and `update_alias_handler` (line 446).

### 3B: Inline PullCommand into `routes_app/src/routes_models/pull.rs`

**What PullCommand.execute() does** (cmd_pull.rs:40-98):
- **ByAlias**: Check alias not exists → find remote model → download → build UserAlias → save
- **ByRepoFile**: Check local exists → if not, download → no alias creation

**Action**: Create private async fns `execute_pull_by_alias(...)` and `execute_pull_by_repo_file(...)` in `pull.rs`.

### 3C: Update error types in `routes_app/src/routes_models/error.rs`

After Phase 2 (error consolidation) + Phase 3 (commands removal), route error enums simplify:

```rust
// CreateAliasError — wraps service error enums (no more standalone structs)
pub enum CreateAliasError {
  DataService(#[from] DataServiceError),   // includes AliasExists, AliasNotFound
  HubService(#[from] HubServiceError),     // includes FileNotFound
  ObjValidation(#[from] ObjValidationError),
  AliasMismatch { path: String, request: String },
}

// PullError — similarly simplified
pub enum PullError {
  DataService(#[from] DataServiceError),
  HubService(#[from] HubServiceError),
  ObjValidation(#[from] ObjValidationError),
  DbError(#[from] DbError),
}
```

### 3D: Delete commands crate and clean up dependencies

1. Delete entire `crates/commands/` directory
2. Workspace `Cargo.toml`:
   - Remove `"crates/commands"` from `members`
   - Remove `commands = { path = "crates/commands" }` from `[workspace.dependencies]`
   - Remove `prettytable = "0.10.0"` from `[workspace.dependencies]` (only used by commands)
3. `crates/routes_app/Cargo.toml`: Remove `commands = { workspace = true }`
4. `crates/server_app/Cargo.toml`: Remove `commands = { workspace = true }` (phantom dep)
5. `crates/lib_bodhiserver/Cargo.toml`: Remove `commands = { workspace = true }` (phantom dep)

### Phase 3 Verification

```bash
cargo check -p routes_app && cargo check -p server_app && cargo check -p lib_bodhiserver
cargo test -p routes_app
make test.backend
```

### Phase 3 Risks

- Error code changes from flattening — audit test assertions for `create_command_error-*` and `pull_command_error-*` codes
- The `update_download_status` fn in `pull.rs` takes `Result<(), commands::PullCommandError>` — signature changes to new local error type

---

## Phase 4: DbService Repository Trait Split

Split the 3300-line `db/service.rs` with 44+ method `DbService` trait into domain-specific repository traits. `DbService` becomes a super-trait. `AppService.db_service()` unchanged.

### 4A: Extract TimeService to own file

Move from `db/service.rs` lines 13-37 to `db/time_service.rs`:
- `TimeService` trait (with `#[mockall::automock]`)
- `DefaultTimeService` struct + impl

### 4B: Move DbError to `db/error.rs`

Currently `DbError` is in `db/service.rs` (lines 39-65). Move to `db/error.rs` alongside existing `SqlxError`, `SqlxMigrateError`. Note: `ItemNotFound` was already folded into `DbError` in Phase 2.

### 4C: Define repository sub-traits

Create 5 repository trait files. Each trait has `#[async_trait]` and extends `Send + Sync + Debug`:

**`db/model_repository.rs`** — ModelRepository (17 methods):

| Group | Methods |
|-------|---------|
| Downloads (5) | `create_download_request`, `get_download_request`, `update_download_request`, `list_download_requests`, `find_download_request_by_repo_filename` |
| API Model Aliases (8) | `create_api_model_alias`, `get_api_model_alias`, `update_api_model_alias`, `update_api_model_cache`, `delete_api_model_alias`, `list_api_model_aliases`, `get_api_key_for_alias`, `check_prefix_exists` |
| Model Metadata (4) | `upsert_model_metadata`, `get_model_metadata_by_file`, `batch_get_metadata_by_files`, `list_model_metadata` |

**`db/access_repository.rs`** — AccessRepository (6 methods):
`insert_pending_request`, `get_pending_request`, `list_pending_requests`, `list_all_requests`, `update_request_status`, `get_request_by_id`

**`db/token_repository.rs`** — TokenRepository (5 methods):
`create_api_token`, `list_api_tokens`, `get_api_token_by_id`, `get_api_token_by_prefix`, `update_api_token`

**`db/toolset_repository.rs`** — ToolsetRepository (14 methods):

| Group | Methods |
|-------|---------|
| Instances (8) | `get_toolset`, `get_toolset_by_name`, `create_toolset`, `update_toolset`, `list_toolsets`, `list_toolsets_by_scope_uuid`, `delete_toolset`, `get_toolset_api_key` |
| App Config (5) | `get_app_toolset_config_by_scope_uuid`, `get_app_toolset_config_by_scope`, `upsert_app_toolset_config`, `list_app_toolset_configs`, `list_app_toolset_configs_by_scopes` |
| App-Client (2) | `get_app_client_toolset_config`, `upsert_app_client_toolset_config` |

**`db/db_core.rs`** — DbCore (3 trait methods):
`migrate()`, `now()`, `encryption_key()`

### 4D: DbService becomes super-trait

In `db/mod.rs`:
```rust
pub trait DbService: ModelRepository + AccessRepository + TokenRepository
    + ToolsetRepository + DbCore + Send + Sync + std::fmt::Debug {}

impl<T> DbService for T where T: ModelRepository + AccessRepository
    + TokenRepository + ToolsetRepository + DbCore + Send + Sync + std::fmt::Debug {}
```

`AppService.db_service() -> Arc<dyn DbService>` unchanged.

### 4E: Implementation files

`db/service.rs` shrinks to:
- `SqliteDbService` struct
- `seed_toolset_configs()` private method
- `get_by_col()` private helper
- Separate `impl` blocks for each repository trait

Tests extracted to `db/tests.rs`.

### 4F: MockAll impact

Remove `#[mockall::automock]` from old `DbService` trait. Create composite `MockDbService` using `mockall::mock!` in `test_utils/db.rs`:

```rust
mockall::mock! {
  pub DbService {}
  #[async_trait::async_trait]
  impl ModelRepository for DbService { /* 17 methods */ }
  #[async_trait::async_trait]
  impl AccessRepository for DbService { /* 6 methods */ }
  #[async_trait::async_trait]
  impl TokenRepository for DbService { /* 5 methods */ }
  #[async_trait::async_trait]
  impl ToolsetRepository for DbService { /* 14 methods */ }
  impl DbCore for DbService { /* 3 methods */ }
}
```

Preserves `MockDbService` name used throughout all test files.

### 4G: Final `db/mod.rs` structure

```rust
pub mod encryption;
mod error;
mod objs;
mod time_service;
mod db_core;
mod model_repository;
mod access_repository;
mod token_repository;
mod toolset_repository;
mod service;
mod sqlite_pool;
#[cfg(test)] mod tests;

pub use error::*;
pub use objs::*;
pub use time_service::*;
pub use db_core::*;
pub use model_repository::*;
pub use access_repository::*;
pub use token_repository::*;
pub use toolset_repository::*;
pub use service::*;
pub use sqlite_pool::DbPool;
```

### Phase 4 Verification

```bash
cargo check -p services && cargo test -p services
make test.backend
```

### Phase 4 Risks

- Object safety: `DbService` super-trait must remain object-safe. All sub-traits use only `&self` methods — verified.
- `mockall::mock!` for 45+ methods is verbose but mechanical.
- Every crate using `services::db::DbService` continues to work since the trait name and methods are preserved.

---

## Flagged Issues (Not In Scope)

| Issue | Location | Notes |
|-------|----------|-------|
| `objs.rs` (AppRegInfo, AppStatus) are domain objects in services | `services/src/objs.rs` | Belong in `objs` crate. Leave as-is per constraint. |
| `service_ext.rs` (SecretServiceExt) convenience trait | `services/src/service_ext.rs` | Leave as-is. |
| Crypto duplication (secret_service.rs vs db/encryption.rs) | Both use AES-256-GCM/PBKDF2 | Different storage backends. Leave duplicated. |
| ExaService experimental status | `services/src/exa_service.rs` | Already isolated in own file. |
| Database types leaking into route responses | `routes_app` uses `services::db::{ApiToken, ...}` | Out of scope. |
| SqlxError/SqlxMigrateError intermediate structs | `services/src/db/error.rs` | Required by `impl_error_from!` orphan rule pattern. Keep as-is. |
| EncryptionError in db/encryption.rs | `services/src/db/encryption.rs` | Used via string conversion, not `#[from]`. Keep as-is. |

---

## File Change Summary

### Phase 1 (7 new, 3 deleted, 1 modified)
| Action | File |
|--------|------|
| NEW | `services/src/tool_service/mod.rs` |
| NEW | `services/src/tool_service/error.rs` |
| NEW | `services/src/tool_service/service.rs` |
| NEW | `services/src/tool_service/tests.rs` |
| NEW | `services/src/setting_service/mod.rs` |
| NEW | `services/src/setting_service/error.rs` |
| NEW | `services/src/setting_service/service.rs` |
| NEW | `services/src/setting_service/tests.rs` |
| DELETE | `services/src/tool_service.rs` |
| DELETE | `services/src/setting_service.rs` |
| DELETE | `services/src/obj_exts/mod.rs` |
| MODIFY | `services/src/lib.rs` |

### Phase 2 (services error consolidation + consumer updates)
| Action | File |
|--------|------|
| MODIFY | `services/src/hub_service.rs` (merge 4 error types → 1) |
| MODIFY | `services/src/data_service.rs` (fold 3 standalone structs) |
| MODIFY | `services/src/token.rs` (fold JsonWebTokenError → TokenError) |
| MODIFY | `services/src/db/error.rs` (fold ItemNotFound → DbError) |
| MODIFY | `routes_app/src/routes_models/aliases.rs` (AliasNotFoundError → DataServiceError) |
| MODIFY | `routes_app/src/routes_models/error.rs` (remove standalone struct imports) |
| MODIFY | `routes_app/src/routes_models/pull.rs` (RemoteModelNotFoundError, ItemNotFound) |
| MODIFY | `routes_app/src/routes_auth/error.rs` (JsonWebTokenError → TokenError) |
| MODIFY | `routes_oai/src/routes_oai_models.rs` (AliasNotFoundError → DataServiceError) |

### Phase 3 (3 modified, 5 Cargo.toml changes, 1 directory deleted)
| Action | File |
|--------|------|
| MODIFY | `routes_app/src/routes_models/aliases.rs` (inline CreateCommand) |
| MODIFY | `routes_app/src/routes_models/pull.rs` (inline PullCommand) |
| MODIFY | `routes_app/src/routes_models/error.rs` (remove command error wrapping) |
| MODIFY | `Cargo.toml` (workspace) |
| MODIFY | `routes_app/Cargo.toml` |
| MODIFY | `server_app/Cargo.toml` |
| MODIFY | `lib_bodhiserver/Cargo.toml` |
| DELETE | `crates/commands/` (entire directory) |

### Phase 4 (8 new, 3 modified)
| Action | File |
|--------|------|
| NEW | `services/src/db/time_service.rs` |
| NEW | `services/src/db/db_core.rs` |
| NEW | `services/src/db/model_repository.rs` |
| NEW | `services/src/db/access_repository.rs` |
| NEW | `services/src/db/token_repository.rs` |
| NEW | `services/src/db/toolset_repository.rs` |
| NEW | `services/src/db/tests.rs` |
| MODIFY | `services/src/db/mod.rs` |
| MODIFY | `services/src/db/service.rs` |
| MODIFY | `services/src/db/error.rs` (add DbError from service.rs) |
| MODIFY | `services/src/test_utils/db.rs` (MockDbService via mock!) |
