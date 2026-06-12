# Plan: Move DB Entities & Repositories to Domain Folders

## Context

After the `objs` crate elimination, domain types (`*_objs.rs`) and business logic services (`*_service.rs`) now live in domain folders under `services/src/`. However, the persistence layer remains centralized in `db/` — 16 SeaORM entity files, 9 repository traits, 9 impl files, 9 test files, and row types in `db/objs.rs`. This creates a disconnect where developers must navigate between `db/` and domain folders when working on a single feature.

This plan moves all domain-specific persistence code into its owning domain folder, so each domain module fully owns its types, entities, repositories, services, and tests. The `db/` module retains only infrastructure: `DefaultDbService`, `DbService` super-trait, `DbCore`, `DbError`, encryption, `TimeService`, and migrations.

Additionally, `McpRepository` (6 tables, ~40 methods) and `ModelRepository` (3 tables, ~20 methods) are split into focused sub-repositories for better cohesion.

---

## Execution Model

Each phase is implemented by a **specialized sub-agent** launched by the main agent. The sub-agent autonomously performs the full cycle for its phase:

1. **Code changes** — move entity files, merge repository trait+impl, move row types, update mod.rs wiring, fix imports
2. **Test changes** — move test files, update test imports
3. **Code compile** — `cargo check -p services`
4. **Test compile** — `cargo build -p services --tests`
5. **Test run + pass** — `cargo test -p services` (and `cargo build` for phases 7-8 where DbService changes)
6. **Format** — `cargo fmt --all`
7. **Lint** — `cargo clippy -p services` (if applicable)
8. **Local commit** — git add + commit with descriptive message

The sub-agent resolves any compilation or test failures before committing. Once complete, control returns to the main agent which launches the next phase's sub-agent.

For phases 7-8 (repository splits that change `DbService`), the sub-agent also runs `cargo build` for the full workspace and fixes any downstream breakage in `auth_middleware`, `routes_app`, etc.

---

## Naming Conventions

| Artifact | Pattern | Example |
|---|---|---|
| SeaORM entity (one per table) | `<table_name>_entity.rs` | `setting_entity.rs` |
| Domain types (pure, no DB) | `<domain>_objs.rs` | `setting_objs.rs` |
| Row types (DB representations) | Inside `*_entity.rs` | `DbSetting` in `setting_entity.rs` |
| Repository (trait + impl merged) | `<name>_repository.rs` | `settings_repository.rs` |
| Repository tests | `test_<name>_repository.rs` | `test_settings_repository.rs` |
| Entity module visibility | `pub(crate)` | SeaORM internals not leaked publicly |

---

## Entity-to-Domain Mapping

| Entity File (current) | Table | Domain Folder | Target Entity File | Row Type(s) | Repository Target |
|---|---|---|---|---|---|
| `entities/setting.rs` | `settings` | `settings/` | `setting_entity.rs` | `DbSetting` | `settings_repository.rs` |
| `entities/app_instance.rs` | `apps` | `apps/` | `app_instance_entity.rs` | `AppInstanceRow` | `app_instance_repository.rs` |
| `entities/api_token.rs` | `api_tokens` | `tokens/` | `api_token_entity.rs` | `ApiToken` (alias) | `token_repository.rs` |
| `entities/access_request.rs` | `access_requests` | `users/` (NEW) | `access_request_entity.rs` | `UserAccessRequest` (alias) | `access_repository.rs` |
| `entities/app_access_request.rs` | `app_access_requests` | `app_access_requests/` | `app_access_request_entity.rs` | `AppAccessRequestRow` | `access_request_repository.rs` |
| `entities/toolset.rs` | `toolsets` | `toolsets/` | `toolset_entity.rs` | `ToolsetRow` | `toolset_repository.rs` |
| `entities/app_toolset_config.rs` | `app_toolset_configs` | `toolsets/` | `app_toolset_config_entity.rs` | `AppToolsetConfigRow` | `toolset_repository.rs` |
| `entities/download_request.rs` | `download_requests` | `models/` | `download_request_entity.rs` | `DownloadRequest` (alias) | `download_repository.rs` (NEW) |
| `entities/api_model_alias.rs` | `api_model_aliases` | `models/` | `api_model_alias_entity.rs` | `ApiAliasView` | `api_alias_repository.rs` (NEW) |
| `entities/model_metadata.rs` | `model_metadata` | `models/` | `model_metadata_entity.rs` | `ModelMetadataRow` (alias) | `model_metadata_repository.rs` (NEW) |
| `entities/user_alias.rs` | `user_aliases` | `models/` | `user_alias_entity.rs` | `UserAlias` (TryFrom) | `user_alias_repository.rs` |
| `entities/mcp_server.rs` | `mcp_servers` | `mcps/` | `mcp_server_entity.rs` | `McpServerRow` | `mcp_server_repository.rs` (NEW) |
| `entities/mcp.rs` | `mcps` | `mcps/` | `mcp_entity.rs` | `McpRow`, `McpWithServerRow` | `mcp_instance_repository.rs` (NEW) |
| `entities/mcp_auth_header.rs` | `mcp_auth_headers` | `mcps/` | `mcp_auth_header_entity.rs` | `McpAuthHeaderRow` | `mcp_auth_repository.rs` (NEW) |
| `entities/mcp_oauth_config.rs` | `mcp_oauth_configs` | `mcps/` | `mcp_oauth_config_entity.rs` | `McpOAuthConfigRow` | `mcp_auth_repository.rs` (NEW) |
| `entities/mcp_oauth_token.rs` | `mcp_oauth_tokens` | `mcps/` | `mcp_oauth_token_entity.rs` | `McpOAuthTokenRow` | `mcp_auth_repository.rs` (NEW) |

## Repository Split Details

### ModelRepository → 3 new traits
- **`DownloadRepository`**: `create_download_request`, `get_download_request`, `update_download_request`, `list_download_requests`, `find_download_request_by_repo_filename`
- **`ApiAliasRepository`**: `create_api_model_alias`, `get_api_model_alias`, `update_api_model_alias`, `update_api_model_cache`, `delete_api_model_alias`, `list_api_model_aliases`, `get_api_key_for_alias`, `check_prefix_exists`
- **`ModelMetadataRepository`**: `upsert_model_metadata`, `get_model_metadata_by_file`, `batch_get_metadata_by_files`, `list_model_metadata`
- **`UserAliasRepository`**: stays independent, moves as-is

### McpRepository → 3 new traits
- **`McpServerRepository`**: `create_mcp_server`, `update_mcp_server`, `get_mcp_server`, `get_mcp_server_by_url`, `list_mcp_servers`, `count_mcps_by_server_id`, `clear_mcp_tools_by_server_id`
- **`McpInstanceRepository`**: `create_mcp`, `get_mcp`, `get_mcp_by_slug`, `list_mcps_with_server`, `update_mcp`, `delete_mcp`
- **`McpAuthRepository`**: all `*_mcp_auth_header_*`, `*_mcp_oauth_config_*`, `*_mcp_oauth_token_*` methods (20 methods total)

### DbService Super-Trait (final form)
```
DbService: DownloadRepository + ApiAliasRepository + ModelMetadataRepository
         + AccessRepository + AccessRequestRepository + AppInstanceRepository
         + TokenRepository + ToolsetRepository
         + McpServerRepository + McpInstanceRepository + McpAuthRepository
         + UserAliasRepository + SettingsRepository + DbCore
         + Send + Sync + Debug
```
(13 sub-traits, up from 9)

---

## Phase Execution Plan

Each phase: move files → update mod.rs wiring → update imports → gate check → commit.

### Phase 1: settings/

**Move:**
- `db/entities/setting.rs` → `settings/setting_entity.rs`
- `db/settings_repository.rs` + `db/service_settings.rs` → `settings/settings_repository.rs` (merged)
- `DbSetting` struct from `db/settings_repository.rs` → `settings/setting_entity.rs`
- `db/test_settings_repository.rs` → `settings/test_settings_repository.rs`

**Update:**
- `settings/mod.rs`: add `pub(crate) mod setting_entity;`, `mod settings_repository;`, test module, re-exports
- `db/mod.rs`: remove 3 modules + re-export
- `db/entities/mod.rs`: remove `pub mod setting;`
- `db/service.rs`: import `SettingsRepository` from `crate::settings`
- `test_utils/db.rs`: update imports for `DbSetting`, `SettingsRepository`

**Gate:** `cargo check -p services && cargo build -p services --tests && cargo test -p services`
**Commit:** `refactor: move settings entity and repository to settings/`

---

### Phase 2: apps/

**Move:**
- `db/entities/app_instance.rs` → `apps/app_instance_entity.rs`
- `db/app_instance_repository.rs` + `db/service_app_instance.rs` → `apps/app_instance_repository.rs` (merged)
- `AppInstanceRow` struct from `db/app_instance_repository.rs` → `apps/app_instance_entity.rs`
- `db/test_app_instance_repository.rs` → `apps/test_app_instance_repository.rs`

**Update:**
- `apps/mod.rs`: add entity + repository modules
- `db/mod.rs`, `db/entities/mod.rs`, `db/service.rs`: remove/update
- `apps/app_instance_service.rs`: change `crate::db::AppInstanceRepository` → `super::AppInstanceRepository`
- `test_utils/db.rs`: update imports

**Gate + Commit:** `refactor: move app instance entity and repository to apps/`

---

### Phase 3: tokens/

**Move:**
- `db/entities/api_token.rs` → `tokens/api_token_entity.rs`
- `db/token_repository.rs` + `db/service_token.rs` → `tokens/token_repository.rs` (merged, replaces existing file)
- `db/test_token_repository.rs` → `tokens/test_token_repository.rs`

**Note:** `tokens/` already has `token_service.rs`. The existing `token_repository` trait methods merge with the impl. `ApiToken` type alias moves from `db/entities/mod.rs` re-export into `tokens/api_token_entity.rs`.

**Update:**
- `tokens/mod.rs`: add entity + repository modules, re-export `ApiToken`
- `db/mod.rs`: remove `pub use entities::ApiToken` + repo modules
- `db/entities/mod.rs`: remove `pub mod api_token;` + `pub use api_token::ApiToken;`
- Downstream: `routes_app/src/api_dto.rs` uses `services::db::ApiToken` → `services::ApiToken`

**Gate + Commit:** `refactor: move token entity and repository to tokens/`

---

### Phase 4: users/ (NEW domain)

**Create:**
- `users/mod.rs`
- `users/user_objs.rs` — `UserAccessRequestStatus` moved from `app_access_requests/access_request_objs.rs`
- `users/access_request_entity.rs` — from `db/entities/access_request.rs` + `UserAccessRequest` alias
- `users/access_repository.rs` — merged from `db/access_repository.rs` + `db/service_access.rs`
- `users/test_access_repository.rs` — from `db/test_access_repository.rs`

**Update:**
- `lib.rs`: add `mod users;` + `pub use users::*;`
- `app_access_requests/access_request_objs.rs`: remove `UserAccessRequestStatus`
- `db/objs.rs`: update re-export of `UserAccessRequestStatus` to `crate::users`
- `db/mod.rs`, `db/entities/mod.rs`, `db/service.rs`: remove/update
- Downstream: `routes_app/src/routes_users/types.rs` uses `services::db::{UserAccessRequest, UserAccessRequestStatus}` → `services::{UserAccessRequest, UserAccessRequestStatus}`

**Gate + Commit:** `refactor: create users domain with access request entity and repository`

---

### Phase 5: app_access_requests/

**Move:**
- `db/entities/app_access_request.rs` → `app_access_requests/app_access_request_entity.rs`
- `AppAccessRequestRow` from `db/objs.rs` → `app_access_requests/app_access_request_entity.rs`
- `db/access_request_repository.rs` + `db/service_access_request.rs` → `app_access_requests/access_request_repository.rs` (merged)
- `db/test_access_request_repository.rs` → `app_access_requests/test_access_request_repository.rs`

**Update:**
- `app_access_requests/mod.rs`: add entity + repository modules
- `db/objs.rs`: remove `AppAccessRequestRow`
- `db/mod.rs`, `db/entities/mod.rs`, `db/service.rs`: remove/update
- `auth_middleware` imports: `services::db::{AccessRequestRepository, AppAccessRequestRow, FlowType}` → `services::{...}`

**Gate + Commit:** `refactor: move app access request entity and repository to app_access_requests/`

---

### Phase 6: toolsets/

**Move:**
- `db/entities/toolset.rs` → `toolsets/toolset_entity.rs`
- `db/entities/app_toolset_config.rs` → `toolsets/app_toolset_config_entity.rs`
- `ToolsetRow` from `db/objs.rs` → `toolsets/toolset_entity.rs`
- `AppToolsetConfigRow` from `db/objs.rs` → `toolsets/app_toolset_config_entity.rs`
- `db/toolset_repository.rs` + `db/service_toolset.rs` → `toolsets/toolset_repository.rs` (merged)
- `db/test_toolset_repository.rs` → `toolsets/test_toolset_repository.rs`

**Update:**
- `toolsets/mod.rs`: add entity + repository modules
- `db/objs.rs`: remove `ToolsetRow`, `AppToolsetConfigRow`
- `db/default_service.rs`: change `crate::db::entities::app_toolset_config` → `crate::toolsets::app_toolset_config_entity`
- `db/mod.rs`, `db/entities/mod.rs`, `db/service.rs`: remove/update

**Gate + Commit:** `refactor: move toolset entities and repository to toolsets/`

---

### Phase 7: models/ (repository split)

**Move entities:**
- `db/entities/download_request.rs` → `models/download_request_entity.rs`
- `db/entities/api_model_alias.rs` → `models/api_model_alias_entity.rs`
- `db/entities/model_metadata.rs` → `models/model_metadata_entity.rs`
- `db/entities/user_alias.rs` → `models/user_alias_entity.rs`

**Split ModelRepository → 3 new traits:**
- `models/download_repository.rs` — `DownloadRepository` trait + impl (from `db/model_repository.rs` Downloads section + `db/service_model.rs` download impl)
- `models/api_alias_repository.rs` — `ApiAliasRepository` trait + impl (API Model Aliases section)
- `models/model_metadata_repository.rs` — `ModelMetadataRepository` trait + impl (Model Metadata section)

**Move UserAliasRepository:**
- `db/user_alias_repository.rs` + `db/service_user_alias.rs` → `models/user_alias_repository.rs` (merged)

**Split test file:**
- `db/test_model_repository.rs` → split into `models/test_download_repository.rs`, `models/test_api_alias_repository.rs`, `models/test_model_metadata_repository.rs`
- `db/test_user_alias_repository.rs` → `models/test_user_alias_repository.rs`

**Update DbService super-trait (`db/service.rs`):**
- Remove: `ModelRepository`, `UserAliasRepository`
- Add: `DownloadRepository + ApiAliasRepository + ModelMetadataRepository + UserAliasRepository` (from `crate::models`)

**Update test infrastructure (`test_utils/db.rs`):**
- TestDbService: replace `impl ModelRepository` with 3 new trait impls; update `impl UserAliasRepository` imports
- MockDbService: replace `impl ModelRepository` with 3 new trait impls in `mock!` macro

**Update consuming services:**
- `models/data_service.rs`, `models/hub_service.rs`: no change (they use `Arc<dyn DbService>` which still includes all methods)
- `utils/queue_service.rs`: same — uses `DbService`, methods are the same

**Remove from db/:**
- `db/mod.rs`: 4 module decls + 2 test modules + re-exports (`DownloadRequest`, `ModelMetadataRow`, `ModelMetadataRowBuilder`)
- `db/entities/mod.rs`: 4 entity modules + re-exports

**Gate:** `cargo check -p services && cargo build --tests && cargo test -p services && cargo build` (full workspace — DbService changed)
**Commit:** `refactor: move model entities and split repositories into models/`

---

### Phase 8: mcps/ (repository split)

**Move entities:**
- `db/entities/mcp_server.rs` → `mcps/mcp_server_entity.rs` + `McpServerRow`
- `db/entities/mcp.rs` → `mcps/mcp_entity.rs` + `McpRow`, `McpWithServerRow`
- `db/entities/mcp_auth_header.rs` → `mcps/mcp_auth_header_entity.rs` + `McpAuthHeaderRow`
- `db/entities/mcp_oauth_config.rs` → `mcps/mcp_oauth_config_entity.rs` + `McpOAuthConfigRow`
- `db/entities/mcp_oauth_token.rs` → `mcps/mcp_oauth_token_entity.rs` + `McpOAuthTokenRow`

**Critical: Update SeaORM `Relation` references.**
Entity files reference sibling entities via `super::mcp_server::Entity`. After move, these become `super::mcp_server_entity::Entity`. All `Relation` enum variants and `Related<>` impl targets need updating.

**Split McpRepository → 3 new traits:**
- `mcps/mcp_server_repository.rs` — `McpServerRepository` (7 methods)
- `mcps/mcp_instance_repository.rs` — `McpInstanceRepository` (6 methods)
- `mcps/mcp_auth_repository.rs` — `McpAuthRepository` (20 methods: auth headers + OAuth configs + OAuth tokens)

**Split test file:**
- `db/test_mcp_repository.rs` → split into `mcps/test_mcp_server_repository.rs`, `mcps/test_mcp_instance_repository.rs`, `mcps/test_mcp_auth_repository.rs`

**Update DbService super-trait:**
- Remove: `McpRepository`
- Add: `McpServerRepository + McpInstanceRepository + McpAuthRepository` (from `crate::mcps`)

**Update test infrastructure:**
- TestDbService + MockDbService: replace `impl McpRepository` with 3 new trait impls

**Remove from db/:**
- `db/objs.rs`: remove all MCP row types (`McpServerRow`, `McpRow`, `McpWithServerRow`, `McpAuthHeaderRow`, `McpOAuthConfigRow`, `McpOAuthTokenRow`)
- `db/mod.rs`: repo + impl + test modules
- `db/entities/mod.rs`: 5 entity modules

**Downstream updates:**
- `routes_app/src/routes_mcp/test_oauth_utils.rs`: `services::db::{McpOAuthConfigRow, McpOAuthTokenRow, McpServerRow}` → `services::{...}`

**Gate:** `cargo build && cargo test -p services && cargo build --tests && cargo fmt --all -- --check`
**Commit:** `refactor: move MCP entities and split repositories into mcps/`

---

### Phase 9: db/ cleanup + full validation

**Verify `db/entities/mod.rs` is empty** → delete `db/entities/` directory, remove `pub mod entities;` from `db/mod.rs`

**Clean up `db/objs.rs`:**
- All row types removed (phases 5-8)
- Re-exports (`AppAccessRequestStatus`, `FlowType`, etc.) — verify if still needed. After entities moved to domain folders, they import directly. Remove redundant re-exports.
- Only `ApiKeyUpdate` remains → keep or move to a more appropriate location

**Verify `db/mod.rs` final state:**
```
mod db_core;
mod default_service;
pub mod encryption;
mod error;
mod objs;          // only ApiKeyUpdate
pub mod sea_migrations;
mod service;
mod time_service;
+ re-exports
```

**Full validation:**
```bash
cargo build && cargo test && cargo fmt --all -- --check
make test.backend
```

**Downstream crate import sweep:**
Verify no remaining `services::db::` imports for types that moved. Key files:
- `auth_middleware/src/token_service/tests.rs` — multiple `services::db::` imports
- `routes_app/src/` — scattered `services::db::` imports
- These should all use `services::TypeName` directly (via `lib.rs` `pub use` chain)

**Commit:** `refactor: clean up db module after entity/repository migration`

---

## db/ Module Final State

After all phases, `db/` contains only infrastructure:

| File | Content |
|---|---|
| `mod.rs` | Module declarations + re-exports |
| `db_core.rs` | `DbCore` trait (migrate, now, encryption_key, reset_all_tables) |
| `default_service.rs` | `DefaultDbService` struct + `DbCore` impl |
| `service.rs` | `DbService` super-trait (13 sub-traits + DbCore) |
| `error.rs` | `DbError` enum |
| `encryption.rs` | AES-GCM encrypt/decrypt helpers |
| `time_service.rs` | `TimeService` trait + `DefaultTimeService` |
| `objs.rs` | `ApiKeyUpdate` enum only |
| `sea_migrations/` | SeaORM migration files |

---

## Critical Files to Modify

| File | Impact |
|---|---|
| `services/src/db/service.rs` | DbService super-trait — updated in phases 7-8 for repo splits |
| `services/src/db/mod.rs` | Modules removed every phase |
| `services/src/db/objs.rs` | Row types evacuated phases 5-8 |
| `services/src/db/entities/mod.rs` | Entity modules removed every phase, deleted in phase 9 |
| `services/src/db/default_service.rs` | Entity import path update (phase 6) |
| `services/src/test_utils/db.rs` | TestDbService + MockDbService updated every phase; major rewrite in phases 7-8 |
| `services/src/lib.rs` | Add `mod users;` in phase 4 |
| Each domain `mod.rs` | Add entity + repository modules + re-exports |
| `auth_middleware/src/token_service/tests.rs` | Multiple `services::db::` imports to update |
| `routes_app/src/` (multiple files) | `services::db::` imports to update |

## Verification

After each phase:
```bash
cargo check -p services && cargo build -p services --tests && cargo test -p services
```

After phases 7-8 (DbService changes):
```bash
cargo build && cargo build --tests && cargo test -p services
```

Final validation (phase 9):
```bash
make test.backend
cargo fmt --all -- --check
```
