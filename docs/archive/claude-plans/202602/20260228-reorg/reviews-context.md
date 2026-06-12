# Post-Reorg Architecture: services Crate + errmeta Crate

## Reorganization Overview

Two commits completed the reorganization:

- **HEAD~1** (`errmeta creation + objs merge`): Deleted the `objs` crate entirely. Created `errmeta` + `errmeta_derive` as the minimal error foundation. Moved all domain types from `objs` into `services`, organized by domain module.
- **HEAD** (`db→domain moves`): Moved SeaORM entities, repositories, and row types from `services/src/db/` sub-directories into their respective domain modules. Left `db/` as thin infrastructure with backward-compat re-exports.

The `objs` crate no longer exists in the workspace. No `Cargo.toml` retains it as a dependency.

---

## Old Structure → New Structure

```
OLD                                 NEW
─────────────────────────────────── ─────────────────────────────────────
objs crate (domain types + errors)  errmeta crate (error contract only)
  + services (business logic)         + services (domain types + logic)
─────────────────────────────────── ─────────────────────────────────────
services/src/db/entities/           services/src/*/  *_entity.rs files
  *.rs  (SeaORM entities)             (co-located with domain modules)
─────────────────────────────────── ─────────────────────────────────────
services/src/db/                    services/src/db/
  access_repository.rs               db_core.rs, default_service.rs,
  mcp_repository.rs                  encryption.rs, error.rs, mod.rs,
  toolset_repository.rs              model_repository.rs, objs.rs,
  ...15+ repository files            sea_migrations/, service.rs,
                                     time_service.rs
                                     (+ backward-compat re-exports)
```

---

## errmeta Crate

**Purpose**: Zero-framework error contract foundation. Enables lightweight crates to participate in structured error handling without pulling in `axum`, `serde`, or `sea-orm`.

**Key types**:
- `AppError` trait — `error_type()`, `code()`, `args()`, derived `status()`. The universal error interface.
- `ErrorType` enum — 10 HTTP error categories mapping to HTTP status codes. Notable: `ServiceUnavailable` serializes as `"service_unavailable"` and `InvalidAppState` as `"invalid_app_state"` (no `_error` suffix — historical inconsistency).
- `IoError` enum — 6 filesystem error variants (`Io`, `WithPath`, `DirCreate`, `FileRead`, `FileWrite`, `FileDelete`) each capturing source error + path context.
- `EntityError` enum — `NotFound(String)` for generic 404 entity lookups.
- `RwLockReadError` — manual `AppError` impl for poisoned RwLock reads.
- `impl_error_from!` macro — bridges Rust orphan rule for external error conversions (~20 usages across workspace).

**errmeta_derive proc macro** (`errmeta_derive` crate): Provides `#[derive(ErrorMeta)]` and `#[error_meta(trait_to_impl = AppError)]`. Auto-generates error codes as `{enum_name_snake_case}-{variant_name_snake_case}`. Supports `args_delegate = false` for transparent variants to suppress inner error arg forwarding.

**Direct downstream users**: `llama_server_proc` (adopted), `mcp_client` (has NOT adopted — uses plain `thiserror`), `services` (re-exports all errmeta types so downstream crates need not add `errmeta` directly).

---

## services Crate Structure

### Domain Modules

Each domain module is private from `lib.rs` (declared as `mod x;`) and exposes types via `pub use x::*` wildcard re-exports at the crate root. Exception: `users` is incorrectly declared `pub mod users` (pending cleanup).

| Module | Key Contents |
|--------|-------------|
| `apps/` | `AppStatus` (objs), `AppInstance` domain type, `AppInstanceRow` entity row, `AppInstanceRepository` trait + impl, `AppInstanceService` trait + impl, `AppInstanceError` (inline in service file — pending extraction to `error.rs`) |
| `auth/` | `ResourceRole`, `TokenScope`, `UserScope`, `AppRole`, `UserInfo`, `TokenStatus` (objs types); `AuthService` + `KeycloakAuthService`; `SessionService` + `DefaultSessionService`; session store, postgres/sqlite backends; `SessionServiceError` in `session_error.rs` |
| `tokens/` | `ApiToken` (type alias for SeaORM `Model` — pending separation), `TokenRepository` + impl, `TokenService` + impl; no domain error type (returns raw `DbError` — pending `TokenServiceError` creation) |
| `users/` | `UserAccessRequest` (SeaORM Model directly), `UserAccessRequestStatus` (misplaced here — belongs in `users/`), `AccessRepository` trait + impl |
| `settings/` | `Setting`, `EnvType`, `AppType`, `LogLevel`, `AppCommand`, `SettingMetadata`; `SettingsMetadataError` (misplaced in `setting_objs.rs` — should be in `error.rs`); `SettingServiceError` in `error.rs`; `SettingsRepository` + impl, `SettingService` + impl |
| `app_access_requests/` | `AppAccessRequest`, `AppAccessRequestStatus`, `FlowType`, `UserAccessRequestStatus` (misplaced — belongs in `users/`); `AppAccessRequestRow` (in entity file — should be in `access_request_objs.rs`); `AccessRequestRepository`, `AccessRequestService`; `AccessRequestError` in `error.rs` (has dead variant `KcUuidCollision`) |
| `models/` | `Repo`, `HubFile`, `Alias` (User/Model/Api variants), `UserAlias`, `ModelAlias`, `ApiAlias`, `OAIRequestParams`, `ModelMetadata`, `JsonVec`, `DownloadStatus`, `BuilderError`; `HubService`, `DataService`; download/alias/metadata repositories; `gguf/` sub-module |
| `ai_apis/` | `AiApiService` + impl; `AiApiServiceError` (inline in service file — no `error.rs`, pending extraction) |
| `mcps/` | `McpServer`, `Mcp`, `McpRow`, `McpServerRow`, etc.; 5 entity files; 3 repository traits + impls; `McpService`; `McpError` in `error.rs` |
| `toolsets/` | `ToolsetRow`, `Toolset`, `AppToolsetConfigRow`; `ToolsetRepository`, `ToolService`, `ExaService`; `ToolsetError` + `ExaError` (ExaError misplaced in `exa_service.rs`) in `error.rs` |
| `shared_objs/` | Cross-cutting framework-dependent types: `ApiError`, `OpenAIApiError`, `ErrorBody`, `SerdeJsonError`, `SerdeYamlError`, `ReqwestError`, `JsonRejectionError`, `ObjValidationError`; log masking utilities |

### Infrastructure Modules

#### `db/`

**What remains** (thin infrastructure):
- `db_core.rs` — `DbCore` trait (`migrate`, `now`, `encryption_key`, `reset_all_tables`)
- `default_service.rs` — `DefaultDbService` struct, SeaORM connection management, `seed_toolset_configs()` (domain seeding logic — layer violation, pending move to `toolsets/`)
- `encryption.rs` — AES-GCM encryption utilities
- `error.rs` — `DbError` enum
- `service.rs` — `DbService` supertrait combining all repository sub-traits
- `time_service.rs` — `TimeService` trait + `DefaultTimeService`
- `sea_migrations/` — 14 migration files (SQLite + PostgreSQL dual support)
- `model_repository.rs` — `ModelRepository` supertrait (backward-compat shim, pending deletion)
- `objs.rs` — `ApiKeyUpdate` enum definition (pending move to `models/`) + 20 backward-compat re-exports

**Backward-compat re-exports in `db/mod.rs`** (pending elimination):
`AccessRepository`, `UserAccessRequest`, `AccessRequestRepository`, `AppToolsetConfigRow`, `ToolsetRepository`, `ToolsetRow`, `ApiAliasRepository`, `DownloadRepository`, `DownloadRequest`, `ModelMetadataRepository`, `ModelMetadataRow`, `UserAliasRepository`, `McpAuthHeaderRow`, `McpAuthRepository`, `McpInstanceRepository`, `McpOAuthConfigRow`, `McpOAuthTokenRow`, `McpRow`, `McpServerRepository`, `McpServerRow`, `McpWithServerRow`, `ApiToken`, `TokenRepository` + all from `objs::*`.

**Downstream callers using `services::db::` paths**: 11 files across `auth_middleware` and `routes_app`, ~64 total `services::db::` references.

#### `test_utils/`

Test infrastructure behind `test-utils` feature flag:
- `db.rs` — `TestDbService`, `MockDbService`, `SeaTestContext`
- `sea.rs` — `sea_context("sqlite"|"postgres")` fixture
- `objs.rs` — test factory functions (naming causes confusion with deleted `objs` crate)
- `model_fixtures.rs` — impl blocks on `Repo`, `HubFileBuilder`, `UserAlias`, etc.
- `auth.rs`, `session.rs`, `envs.rs`, `settings.rs`, `bodhi.rs`, `data.rs`, `hf.rs`, `http.rs`, `io.rs`, `logs.rs`, `queue.rs`, `network.rs`, `test_data.rs`
- `AppServiceStub` builder, `FrozenTimeService`, `OfflineHubService`, `SecretServiceStub`

#### `utils/`

Utility services: `concurrency_service.rs`, `queue_service.rs`, `cache_service.rs`, `keyring_service.rs`, `network_service.rs`.

#### Root-level orphan: `token.rs`

`src/token.rs` contains `TokenError`, `AccessRequestValidationError`, JWT claim structs (`ScopeClaims`, `Claims`, `OfflineClaims`, etc.), and `extract_claims<T>()`. This belongs in `tokens/` but is currently a sibling of `tokens/`. Creates two parallel token modules (`token` and `tokens`).

### Module File Conventions

Expected layout for each well-structured domain module:

```
<domain>/
  mod.rs              — module declarations + pub use re-exports only
  <domain>_objs.rs    — domain types (structs, enums, validation)
  <domain>_entity.rs  — SeaORM entity definition + row struct
  <domain>_repository.rs — repository trait + DefaultDbService impl
  <domain>_service.rs — business logic service trait + impl
  error.rs            — domain error enum with #[derive(ErrorMeta)]
```

Not all modules fully conform. Known deviations documented in Known Structural Issues below.

---

## Error Architecture

```
errmeta_derive → errmeta (AppError trait, ErrorType, IoError, EntityError)
                     |
services domain modules: <domain>/error.rs with #[derive(ErrorMeta)]
each error variant → error_type() → HTTP status
                     |
services/shared_objs/error_api.rs: ApiError (From<T: AppError>)
                     |
services/shared_objs/error_oai.rs: OpenAIApiError (OpenAI-compatible envelope)
                     |
axum IntoResponse → JSON HTTP response
```

Error code format: `{enum_name_snake_case}-{variant_name_snake_case}`.
Transparent variants delegate to inner error's code.

---

## Known Structural Issues (Pending Cleanup)

1. **`db/objs.rs`** — defines `ApiKeyUpdate` (should be in `models/`) + 20 backward-compat re-exports that duplicate `lib.rs` re-exports. 64 downstream usages via `services::db::` path across 23 files in `auth_middleware` and `routes_app`.

2. **`db/mod.rs` backward-compat re-exports** — 20+ types re-exported "for backward compatibility" from domain modules. Should be removed once downstream callers updated.

3. **`token.rs` at crate root** — JWT logic and `TokenError` should move into `tokens/` module. Currently creates two separate `token` vs `tokens` modules.

4. **`mcp_client::McpClientError`** — does not implement `AppError` / does not use `errmeta`. `llama_server_proc` has adopted `errmeta`; `mcp_client` has not.

5. **`users` module is `pub mod`** — unique in `lib.rs`; all other domain modules are private. Should be `mod users;`.

6. **`AppInstanceError`** — defined inline in `apps/app_instance_service.rs`, not in `apps/error.rs`.

7. **`tokens/` module has no domain error type** — methods return raw `DbError`.

8. **`SettingsMetadataError`** — defined in `settings/setting_objs.rs`, not `settings/error.rs`.

9. **`UserAccessRequestStatus`** — defined in `app_access_requests/access_request_objs.rs`, belongs in `users/`.

10. **`AppAccessRequestRow`** — defined in entity file, should be in `access_request_objs.rs`.

11. **`AiApiServiceError`** — inline in `ai_apis/ai_api_service.rs` (768-line file), no `error.rs`.

12. **`ExaError`** — inline in `toolsets/exa_service.rs`, should be in `toolsets/error.rs`.

13. **`db/default_service.rs` seeds toolset domain** — `seed_toolset_configs()` is domain logic in infrastructure layer.

14. **`ApiToken` = type alias for SeaORM `Model`** — domain type and entity type are the same; no separation.

15. **`test_utils/objs.rs`** — module named `objs`, causing confusion with the deleted `objs` crate. Has stale backward-compat comment.

16. **`db/model_repository.rs`** — backward-compat supertrait with no callers outside `db/` itself.

---

## Dependency Flow

```
errmeta_derive → errmeta → [llama_server_proc, services]
                mcp_client (standalone, no errmeta) → services
                              ↓
                           services
                          /         \
              auth_middleware    server_core
                          \         /
                          routes_app
                              |
                          server_app
                              |
                        lib_bodhiserver
                        /             \
          lib_bodhiserver_napi    bodhi/src-tauri
```

Note: `mcp_client` does NOT depend on `errmeta` despite the root CLAUDE.md diagram showing otherwise.
