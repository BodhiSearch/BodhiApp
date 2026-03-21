# Plan: Eliminate `objs` Crate — Migrate to `services` + `errmeta`

## Context

The `objs` crate was BodhiApp's universal foundation layer (~26 modules). As the domain model matures and `services` has been restructured into domain-grouped modules, `objs` is now an unnecessary indirection. This plan distributes all `objs` types to their proper domain homes: a minimal `errmeta` companion crate for core error infrastructure, and domain modules within `services` for everything else.

**No production deployment** — clean direct migration, no backward compatibility. Take opportunity to clean up code patterns, apply rstest fixtures/case/values, and establish uniform test file conventions.

## Implementation Principles

1. **Simplest first** — move loosely linked, recent, fewer cross-dependency domains first
2. **Gate check after every step** — `cargo build -p objs -p services && cargo test -p objs -p services` + `cargo build` (full workspace through routes_app/bodhi)
3. **Local commit after each gate-checked step** — incremental, reversible progress
4. **Sub-agent per step** — each step implemented by a specialized sequential sub-agent
5. **Foundation first** — errmeta + mcp_client before domain migrations
6. **Gradual objs dissolution** — types removed from objs as they move; objs shrinks each step

## Final Dependency Chain

```
errmeta_derive (proc-macro)
       |
    errmeta (AppError, ErrorType, IoError, EntityError — minimal, no frameworks)
    /      \
   /        \
llama_server_proc    mcp_client (McpTool self-contained, no errmeta needed)
       \                 /
        \               /
         services (domain types + business logic + ApiError + framework error wrappers)
        /          \
server_core    auth_middleware
        \          /
         routes_app (API_TAG_* constants here)
             |
         server_app
             |
      lib_bodhiserver
      /             \
lib_bodhiserver_napi  bodhi/src-tauri
```

---

## Type Distribution Summary

### `errmeta` crate (NEW — minimal, only std + thiserror + strum + errmeta_derive)

| Type | Source File |
|---|---|
| `AppError` trait | `objs/src/error/common.rs` |
| `ErrorType` enum | `objs/src/error/common.rs` (rewrite `status()` to return `u16` instead of `axum::StatusCode`) |
| `impl_error_from!` macro | `objs/src/lib.rs` |
| `IoError` enum + constructors | `objs/src/error/objs.rs` (std::io only) |
| `EntityError` enum | `objs/src/error/objs.rs` |
| `RwLockReadError` struct | `objs/src/error/objs.rs` (manual `new()`, no derive_new) |

### `services/apps/` → `app_objs.rs` (simplest — 1 enum)

| Type | Source |
|---|---|
| `AppStatus` | `objs/src/db_enums.rs` |
| `AppInstance` | already in `services/src/objs.rs` |

### `services/app_access_requests/` → `access_request_objs.rs`

| Type | Source |
|---|---|
| `AppAccessRequestStatus`, `FlowType`, `UserAccessRequestStatus` | `objs/src/access_request.rs` |
| `ApprovalStatus`, `ToolsetTypeRequest`, `ToolsetApproval`, `ToolsetInstance` | `objs/src/access_request.rs` |
| `McpServerRequest`, `McpApproval`, `McpInstance` | `objs/src/access_request.rs` |
| `ApprovedResources`, `RequestedResources` | `objs/src/access_request.rs` |

### `services/toolsets/` → `toolset_objs.rs`

| Type | Source |
|---|---|
| `Toolset`, `ToolsetDefinition`, `ToolDefinition`, `FunctionDefinition` | `objs/src/toolsets.rs` |
| `AppToolsetConfig`, `ToolsetExecutionRequest`, `ToolsetExecutionResponse` | `objs/src/toolsets.rs` |
| Validation fns + constants (`MAX_TOOLSET_SLUG_LEN`, etc.) | `objs/src/toolsets.rs` |

### `services/mcps/` → `mcp_objs.rs`

| Type | Source |
|---|---|
| `McpServer`, `McpServerInfo`, `Mcp` | `objs/src/mcp.rs` |
| `McpAuthType`, `RegistrationType` | `objs/src/mcp.rs` |
| `McpAuthHeader`, `McpOAuthConfig`, `McpOAuthToken` | `objs/src/mcp.rs` |
| `McpExecutionRequest`, `McpExecutionResponse` | `objs/src/mcp.rs` |
| `CreateMcpAuthConfigRequest`, `McpAuthConfigResponse` | `objs/src/mcp.rs` |
| MCP validation fns + constants | `objs/src/mcp.rs` |

### `services/settings/` → `setting_objs.rs`

| Type | Source |
|---|---|
| `Setting`, `SettingInfo`, `SettingMetadata`, `SettingSource`, `SettingType`, `NumberRange` | `objs/src/envs.rs` |
| `SettingsMetadataError` | `objs/src/envs.rs` |
| `EnvType`, `AppType`, `LogLevel` + tracing `From` impls | `objs/src/envs.rs` |
| `AppCommand` | `objs/src/app_command.rs` |

### `services/auth/` → `auth_objs.rs`

| Type | Source |
|---|---|
| `ResourceRole`, `RoleError`, `AppRole` | `objs/src/resource_role.rs` |
| `TokenScope`, `TokenScopeError` | `objs/src/token_scope.rs` |
| `UserScope`, `UserScopeError` | `objs/src/user_scope.rs` |
| `UserInfo` | `objs/src/user.rs` |
| `TokenStatus` | `objs/src/db_enums.rs` |

### `services/models/` → `model_objs.rs` + `gguf/` (largest, most cross-deps)

| Type | Source |
|---|---|
| `Repo` | `objs/src/repo.rs` |
| `HubFile` | `objs/src/hub_file.rs` |
| `Alias`, `AliasSource`, `UserAlias`, `ModelAlias` | `objs/src/alias.rs`, `user_alias.rs`, `model_alias.rs` |
| `ApiAlias`, `ApiAliasBuilder`, `ApiFormat` | `objs/src/api_model_alias.rs` |
| `OAIRequestParams`, `OAIRequestParamsBuilder` | `objs/src/oai.rs` |
| `ModelMetadata`, `ModelCapabilities`, `ToolCapabilities`, `ContextLimits`, `ModelArchitecture` | `objs/src/model_metadata.rs` |
| `DownloadStatus` | `objs/src/db_enums.rs` |
| `ModelValidationError` (new: `FilePatternMismatch` + `ForwardAllRequiresPrefix`) | split from `ObjValidationError` |
| GGUF module | `objs/src/gguf/` (entire directory) |

### `services/objs/` → cross-cutting directory module

| Type | Source |
|---|---|
| `JsonVec` | `objs/src/json_vec.rs` |
| `is_default()`, `to_safe_filename()`, `ILLEGAL_CHARS` | `objs/src/utils.rs` |
| `ApiError`, `OpenAIApiError`, `ErrorBody` | `objs/src/error/error_api.rs`, `error_oai.rs` |
| `JsonRejectionError` | `objs/src/error/objs.rs` (axum-specific) |
| `ReqwestError`, `BuilderError` | `objs/src/error/objs.rs` |
| `SerdeJsonError`, `SerdeYamlError` | `objs/src/error/objs.rs` |
| `ObjValidationError` (only `ValidationErrors` variant) | `objs/src/error/objs.rs` |
| `log` module | `objs/src/log.rs` |

### Other moves

| Type | Target | Source |
|---|---|---|
| `McpTool` | `mcp_client` crate | `objs/src/mcp.rs` |
| `API_TAG_*` constants | `routes_app/src/constants.rs` | `objs/src/api_tags.rs` |

### `services/test_utils/` → sub-module hierarchy

```
services/test_utils/
├── mod.rs          (pub mod common; pub mod models; pub mod auth; ...)
├── common.rs       (temp_bodhi_home, temp_dir, parse<T>, enable_tracing, fixed_dt, SNAPSHOT, copy_test_dir)
├── models.rs       (Repo factories, HubFileBuilder, AliasBuilder, OAIRequestParams builders, GGUF test data)
├── auth.rs         (merge with existing auth test utils)
├── mcps.rs         (MCP test builders)
├── settings.rs     (env setup utilities, merge with existing)
├── ... existing files (app.rs, data.rs, db.rs, hf.rs, objs.rs, queue.rs, sea.rs, session.rs)
```

Downstream imports: `use services::test_utils::models::{Repo, HubFileBuilder};`

---

## Step-by-Step Execution Plan

Each step is implemented by a sub-agent, gate-checked, and locally committed.

### Step 1: Create `errmeta` companion crate

**Sub-agent scope**: Create new crate, zero changes to existing code.

**Actions**:
1. Create `crates/errmeta/Cargo.toml` — deps: `errmeta_derive` (workspace), `thiserror`, `strum`
2. Register in workspace `Cargo.toml` (`members` + `[workspace.dependencies]`)
3. Create source files:
   - `src/lib.rs` — re-exports, `impl_error_from!` macro, `pub use errmeta_derive::ErrorMeta`
   - `src/error_type.rs` — `ErrorType` enum with `status() -> u16` (no axum dep)
   - `src/app_error.rs` — `AppError` trait + `Box<dyn AppError>` impls
   - `src/io_error.rs` — `IoError` enum + convenience constructors
   - `src/entity_error.rs` — `EntityError` enum
   - `src/rwlock_error.rs` — `RwLockReadError` struct (manual `new()`)
4. Create test files: `test_<name>.rs` sibling pattern, rstest where applicable

**Gate**: `cargo check -p errmeta && cargo test -p errmeta`
**Commit**: `feat: create errmeta companion crate with core error infrastructure`

---

### Step 2: Move McpTool to `mcp_client`

**Sub-agent scope**: Define McpTool in mcp_client, update mcp_client to drop objs dep.

**Actions**:
1. Define `McpTool` struct in `mcp_client/src/lib.rs` (with serde/utoipa derives)
2. Remove `objs` dep from `mcp_client/Cargo.toml`, add serde/utoipa
3. Update `services` to import `McpTool` from `mcp_client` instead of `objs`
4. Update `services/lib.rs` re-export of McpTool
5. Do NOT remove McpTool from objs yet (other crates still import objs)

**Gate**: `cargo build -p mcp_client -p services && cargo test -p mcp_client -p services`
**Commit**: `refactor: move McpTool to mcp_client crate`

---

### Step 3: Move `AppStatus` → `services/apps/app_objs.rs`

**Sub-agent scope**: Simplest domain move — 1 enum, minimal cross-deps.

**Actions**:
1. Create `services/src/apps/app_objs.rs` with `AppStatus` enum
2. Move `AppInstance` from `services/src/objs.rs` into `app_objs.rs`
3. Create `test_app_objs.rs` (if tests exist)
4. Update `services/apps/mod.rs`: add declarations + re-exports
5. Update `services/src/objs.rs`: remove AppInstance and AppStatus re-export
6. Fix internal services imports
7. Remove `AppStatus` from `objs/src/db_enums.rs` and `objs/src/lib.rs`

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build` (workspace)
**Commit**: `refactor: move AppStatus to services/apps`

---

### Step 4: Move access request types → `services/app_access_requests/access_request_objs.rs`

**Sub-agent scope**: Relatively new types, contained within access request domain.

**Actions**:
1. Create `services/src/app_access_requests/access_request_objs.rs`
2. Move: `AppAccessRequestStatus`, `FlowType`, `UserAccessRequestStatus`, `ApprovalStatus`, `ToolsetTypeRequest`, `ToolsetApproval`, `ToolsetInstance`, `McpServerRequest`, `McpApproval`, `McpInstance`, `ApprovedResources`, `RequestedResources`
3. Create `test_access_request_objs.rs` with rstest patterns
4. Update `services/app_access_requests/mod.rs`
5. Fix internal services imports (access_request_service.rs, db/objs.rs, db entities)
6. Remove from `objs/src/access_request.rs` and `objs/src/lib.rs`

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move access request types to services/app_access_requests`

---

### Step 5: Move toolset types → `services/toolsets/toolset_objs.rs`

**Sub-agent scope**: Self-contained toolset domain.

**Actions**:
1. Create `services/src/toolsets/toolset_objs.rs`
2. Move: `Toolset`, `ToolsetDefinition`, `ToolDefinition`, `FunctionDefinition`, `AppToolsetConfig`, `ToolsetExecutionRequest`, `ToolsetExecutionResponse`, validation fns + constants
3. Create `test_toolset_objs.rs` with rstest patterns
4. Update `services/toolsets/mod.rs`
5. Fix internal services imports (tool_service.rs, exa_service.rs, db entities)
6. Remove from `objs/src/toolsets.rs` and `objs/src/lib.rs`

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move toolset types to services/toolsets`

---

### Step 6: Move MCP types → `services/mcps/mcp_objs.rs`

**Sub-agent scope**: MCP domain types (McpTool already moved in Step 2).

**Actions**:
1. Create `services/src/mcps/mcp_objs.rs`
2. Move: `McpServer`, `McpServerInfo`, `Mcp`, `McpAuthType`, `RegistrationType`, `McpAuthHeader`, `McpOAuthConfig`, `McpOAuthToken`, `McpExecutionRequest`, `McpExecutionResponse`, `CreateMcpAuthConfigRequest`, `McpAuthConfigResponse`, MCP validation fns + constants
3. Create `test_mcp_objs.rs` with rstest patterns
4. Update `services/mcps/mod.rs`
5. Fix internal services imports (mcp_service.rs, db entities, db/objs.rs)
6. Remove from `objs/src/mcp.rs` and `objs/src/lib.rs` (McpTool should already be gone)

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move MCP types to services/mcps`

---

### Step 7: Move settings/config types → `services/settings/setting_objs.rs`

**Sub-agent scope**: Settings domain — used by downstream crates (lib_bodhiserver, server_app).

**Actions**:
1. Create `services/src/settings/setting_objs.rs`
2. Move: `Setting`, `SettingInfo`, `SettingMetadata`, `SettingSource`, `SettingType`, `NumberRange`, `SettingsMetadataError`, `EnvType`, `AppType`, `LogLevel`, `AppCommand`
3. Create `test_setting_objs.rs` with rstest patterns
4. Update `services/settings/mod.rs`
5. Fix internal services imports (setting_service.rs, bootstrap_parts.rs, etc.)
6. Remove from `objs/src/envs.rs`, `objs/src/app_command.rs`, and `objs/src/lib.rs`

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move settings and config types to services/settings`

---

### Step 8: Move auth types → `services/auth/auth_objs.rs`

**Sub-agent scope**: Auth types — heavily used by auth_middleware, routes_app.

**Actions**:
1. Create `services/src/auth/auth_objs.rs`
2. Move: `ResourceRole`, `RoleError`, `AppRole`, `TokenScope`, `TokenScopeError`, `UserScope`, `UserScopeError`, `UserInfo`, `TokenStatus`
3. Create `test_auth_objs.rs` with rstest patterns
4. Update `services/auth/mod.rs`
5. Fix internal services imports (auth_service.rs, access_request_service.rs, token_service.rs, db entities)
6. Remove from `objs/src/resource_role.rs`, `token_scope.rs`, `user_scope.rs`, `user.rs`, `db_enums.rs`, `lib.rs`

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move auth types to services/auth`

---

### Step 9: Move cross-cutting types → `services/objs/` directory module

**Sub-agent scope**: Convert services/objs.rs to directory, move framework-dependent errors + utilities.

**Actions**:
1. Convert `services/src/objs.rs` → `services/src/objs/` directory module
2. Create sub-files:
   - `mod.rs` — declarations + re-exports
   - `json_vec.rs` — `JsonVec` from `objs/src/json_vec.rs`
   - `utils.rs` — `is_default()`, `to_safe_filename()`, `ILLEGAL_CHARS` from `objs/src/utils.rs`
   - `error_api.rs` — `ApiError` + `IntoResponse` from `objs/src/error/error_api.rs`
   - `error_oai.rs` — `OpenAIApiError`, `ErrorBody` from `objs/src/error/error_oai.rs`
   - `error_wrappers.rs` — `ReqwestError`, `BuilderError`, `SerdeJsonError`, `SerdeYamlError`, `ObjValidationError` (ValidationErrors variant only), `JsonRejectionError`
   - `log.rs` — log utilities from `objs/src/log.rs`
3. Create test files with rstest patterns
4. Fix internal services imports
5. Remove from objs

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move cross-cutting types to services/objs`

---

### Step 10: Move model types → `services/models/model_objs.rs` + `gguf/`

**Sub-agent scope**: Largest migration — most cross-deps. May sub-split into sub-steps.

**Actions**:
1. Create `services/src/models/model_objs.rs`
2. Move: `Repo`, `HubFile`, `Alias`, `AliasSource`, `UserAlias`, `UserAliasBuilder`, `ModelAlias`, `ApiAlias`, `ApiAliasBuilder`, `ApiFormat`, `OAIRequestParams`, `OAIRequestParamsBuilder`, `ModelMetadata`, `ModelCapabilities`, `ToolCapabilities`, `ContextLimits`, `ModelArchitecture`, `DownloadStatus`
3. Create `ModelValidationError` (split from `ObjValidationError`: `FilePatternMismatch` + `ForwardAllRequiresPrefix`)
4. Move `objs/src/gguf/` → `services/src/models/gguf/`
5. Create `test_model_objs.rs` with rstest patterns
6. Update `services/models/mod.rs`
7. Fix internal services imports (data_service.rs, hub_service.rs, ai_api_service.rs, db entities, db/objs.rs)
8. Remove from objs (repo.rs, hub_file.rs, alias.rs, user_alias.rs, model_alias.rs, api_model_alias.rs, oai.rs, model_metadata.rs, gguf/, db_enums.rs remaining)

**Gate**: `cargo build -p objs -p services && cargo test -p objs -p services && cargo build`
**Commit**: `refactor: move model types and GGUF to services/models`

---

### Step 11: Move API_TAG_* constants → `routes_app`

**Sub-agent scope**: 12 string constants, only used in routes_app.

**Actions**:
1. Create `routes_app/src/constants.rs` (or add to existing shared module)
2. Move 12 `API_TAG_*` constants from `objs/src/api_tags.rs`
3. Update all `use objs::API_TAG_*` in routes_app to crate-internal imports
4. Remove from `objs/src/api_tags.rs` and `objs/src/lib.rs`

**Gate**: `cargo build -p objs -p routes_app && cargo test -p routes_app`
**Commit**: `refactor: move API_TAG constants to routes_app`

---

### Step 12: Update `llama_server_proc` to use `errmeta` instead of `objs`

**Sub-agent scope**: Replace objs dependency with errmeta, define own error wrappers.

**Actions**:
1. Replace `objs` with `errmeta` in `llama_server_proc/Cargo.toml`
2. Change `use objs::{AppError, ErrorType, IoError, impl_error_from}` → `use errmeta::{...}`
3. Define own `LlamaReqwestError` and `LlamaBuildError` wrappers
4. Test code using `Repo`/`HubFile`: replace with direct `PathBuf` construction

**Gate**: `cargo build -p llama_server_proc && cargo test -p llama_server_proc && cargo build`
**Commit**: `refactor: llama_server_proc uses errmeta instead of objs`

---

### Step 13: Move test_utils to `services/test_utils/` sub-module hierarchy

**Sub-agent scope**: Merge objs test_utils into services test_utils.

**Actions**:
1. Reorganize services test_utils with sub-module hierarchy: `common.rs`, `models.rs`, etc.
2. Merge objs test_utils content into appropriate sub-modules
3. Update `services/Cargo.toml` `test-utils` feature: absorb objs' test deps
4. Refactor merged tests to use rstest case/values/fixture patterns
5. Remove `objs/src/test_utils/`

**Gate**: `cargo build -p services && cargo test -p services && cargo build`
**Commit**: `refactor: merge test_utils into services with sub-module hierarchy`

---

### Step 14: Update all downstream crate imports + remove objs dep

**Sub-agent scope**: Final import sweep across all downstream crates.

**Import replacement rules**:
| Old | New |
|---|---|
| `use objs::{AppError, ErrorType, IoError, EntityError, ...}` | `use errmeta::{...}` |
| `use objs::impl_error_from` | `use errmeta::impl_error_from` |
| `use objs::{ResourceRole, Alias, ApiError, ...}` | `use services::{...}` |
| `use objs::test_utils::*` | `use services::test_utils::<module>::*` |

**Cargo.toml updates** — remove `objs`, add `errmeta`:
- `server_core`, `auth_middleware`, `routes_app`, `server_app`, `lib_bodhiserver`, `bodhi/src-tauri`

**Gate**: `cargo build && cargo test -p server_core -p auth_middleware -p routes_app -p server_app`
**Commit**: `refactor: update all downstream crates to use services + errmeta`

---

### Step 15: Delete `objs` crate

**Sub-agent scope**: Final cleanup.

**Actions**:
1. Verify: `grep -r "use objs::" crates/` returns nothing
2. Verify: no `Cargo.toml` references objs
3. Delete `crates/objs/` directory
4. Remove from workspace `Cargo.toml` (`members` + `[workspace.dependencies]`)

**Gate**: `cargo build && cargo test`
**Commit**: `refactor: remove objs crate`

---

### Step 16: Full regression + documentation

**Sub-agent scope**: Full test suite, formatting, docs update.

**Actions**:
```bash
make test.backend
cargo run --package xtask openapi
make build.ts-client
make format.all
```

**Documentation updates**:
- Create `crates/errmeta/CLAUDE.md` and `PACKAGE.md`
- Update root `CLAUDE.md` (replace objs in crate table, update dependency chain)
- Update `crates/services/CLAUDE.md` and `PACKAGE.md`
- Update affected crate docs

**Gate**: `make test.backend`
**Commit**: `docs: update documentation for objs elimination`

---

## Test Conventions (Applied Throughout)

- **File pattern**: `test_<name>.rs` sibling files, included via `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`
- **rstest**: Apply `#[case]`/`#[values]`/`#[fixture]` to reduce duplication in migrated tests
- **No `use super::*`**: Use explicit imports in test modules
- **assert_eq!(expected, actual)**: JUnit convention

## Key Risk Mitigations

1. **Step 10 blast radius**: Model types are most widely imported — sub-split if too large
2. **ObjValidationError split**: Code matching `ObjValidationError::FilePatternMismatch` changes to `ModelValidationError::FilePatternMismatch`
3. **ErrorType.status() rewrite**: Returns `u16` instead of `axum::StatusCode` — safe, all callers use numeric value
4. **Feature flag propagation**: `services/test-utils` must absorb objs' test-only deps
5. **llama_server_proc**: Must define own error wrappers since it can't depend on services

## Critical Files Reference

| File | Role |
|---|---|
| `crates/objs/src/lib.rs` | Master re-export + impl_error_from! — dissolve across errmeta + services |
| `crates/objs/src/error/common.rs` | AppError + ErrorType — move to errmeta |
| `crates/objs/src/error/objs.rs` | Error types — split across errmeta + services |
| `crates/objs/src/error/error_api.rs` | ApiError — move to services/objs/ |
| `crates/services/src/lib.rs` | Must absorb all domain type re-exports |
| `crates/services/src/app_service.rs` | DI registry — validates re-export chain compiles |
| `crates/errmeta_derive/src/generate.rs` | Generated code uses no external paths — no changes needed |
