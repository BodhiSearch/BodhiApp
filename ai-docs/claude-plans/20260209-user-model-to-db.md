# Plan: Migrate UserAlias from YAML Files to SQLite Database

## Context

UserAlias is currently stored as individual YAML files in `$BODHI_HOME/aliases/`. This is the only remaining file-based storage in the application -- ApiAlias already uses SQLite, ModelAlias is runtime-discovered. Moving UserAlias to SQLite:
- Unifies storage patterns (everything persistent is in SQLite)
- Enables UUID-based identity consistent with other entities
- Eliminates filesystem I/O for alias CRUD
- Makes tests cleaner (DB seeding vs copying YAML fixture files)

Additionally, RemoteModel (`models.yaml`) never went live and is being removed entirely.

No backward compatibility required -- no production release exists.

## Decisions Made

| Decision | Choice |
|----------|--------|
| UserAlias primary key | UUID (`id TEXT PRIMARY KEY`), `alias` as UNIQUE column |
| ModelAlias changes | None -- stays auto-discovered, no UUID, no DB |
| RemoteModel | Remove entirely (struct, models.yaml, pull-by-alias endpoint) |
| Sync/async | All DataService alias methods become async |
| New endpoints | `DELETE /api/models/{uuid}`, `POST /api/models/{uuid}/copy` |
| Existing endpoints | Switch to UUID path params |
| Copy semantics | `POST /api/models/{uuid}/copy` with `{"alias": "new-name"}`, creates new UUID |
| chat_template field | Keep excluded (legacy, unused) |
| Dead code removal | `read_file`/`write_file`/`find_file`, `MODELS_YAML`, `aliases_dir()`, `HubServiceError::RemoteModelNotFound` |

## Orchestration Strategy

Each phase is implemented by launching a **Task sub-agent** with full context. The main agent:
1. Launches sub-agent with phase-specific instructions and all relevant context
2. Sub-agent implements changes, runs tests, validates correctness
3. Sub-agent returns summary of what was done and any deviations
4. Main agent reviews summary, updates plan progress, makes local commit
5. Main agent launches next sub-agent for the next phase

---

## Phase 1: Remove RemoteModel and Pull-by-Alias
**Status**: [x] Complete
**Sub-agent type**: `general-purpose`
**Validation**: `cargo test`

### Context for sub-agent
Self-contained cleanup. Remove RemoteModel type, models.yaml, pull-by-alias endpoint, and all related code. No structural changes to UserAlias or DataService beyond removing RemoteModel methods.

### Files to modify

**DELETE files:**
- `crates/objs/src/remote_file.rs` -- RemoteModel struct definition
- `crates/objs/tests/data/bodhi/models.yaml` -- test fixture
- `crates/integration-tests/tests/data/live/bodhi/models.yaml` -- integration test fixture

**objs crate:**
- `crates/objs/src/lib.rs` -- remove `mod remote_file` + `pub use remote_file::*`
- `crates/objs/src/test_utils/objs.rs` -- remove `RemoteModel::llama3()`, `RemoteModel::testalias()` factory methods and RemoteModel import

**services crate:**
- `crates/services/src/setting_service/mod.rs` -- remove `MODELS_YAML` constant
- `crates/services/src/setting_service/service.rs` -- remove `models_yaml()` default method from SettingService trait
- `crates/services/src/hub_service.rs` -- remove `RemoteModelNotFound` variant from `HubServiceError`
- `crates/services/src/data_service.rs` -- remove `list_remote_models()` and `find_remote_model()` from DataService trait + LocalDataService impl + `models_yaml()` helper + MODELS_YAML/RemoteModel imports + ~4 RemoteModel tests (around lines 345-416)
- `crates/services/src/test_utils/data.rs` -- remove delegate methods for RemoteModel + RemoteModel import

**routes_app crate:**
- `crates/routes_app/src/routes_models/pull.rs` -- delete `pull_by_alias_handler` (lines 237-302) and `execute_pull_by_alias` (lines 390-419), clean up unused imports (UserAliasBuilder, HubServiceError)
- `crates/routes_app/src/routes_models/mod.rs` -- remove `pull_by_alias_handler` from re-exports
- `crates/routes_app/src/routes_models/tests/pull_test.rs` -- remove pull-by-alias tests
- `crates/routes_app/src/shared/openapi.rs` -- remove `__path_pull_by_alias_handler` from OpenAPI doc registration
- `crates/routes_app/src/routes_models/error.rs` -- verify PullError variants, remove any dead ones

**routes_all crate:**
- `crates/routes_all/src/routes.rs` -- remove `pull_by_alias_handler` import (line ~30) and route registration (line ~208: `.route(&format!("{ENDPOINT_MODEL_PULL}/{{id}}"), post(pull_by_alias_handler))`)

### Verification
```bash
cargo test
```

---

## Phase 2: UserAlias Struct + DB Schema + DataService Migration (objs + services)
**Status**: [x] Complete
**Sub-agent type**: `general-purpose`
**Validation**: `cargo test -p objs && cargo test -p services`

### Context for sub-agent
Changes to foundation crates only. Route handlers in routes_app/routes_all will have compilation errors until Phase 3 -- **that is expected and acceptable**. Focus ONLY on objs and services crates compiling and passing tests.

### 2.1 DB Migration
Create `crates/services/migrations/0009_user_aliases.up.sql`:
```sql
CREATE TABLE user_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    alias TEXT NOT NULL UNIQUE,
    repo TEXT NOT NULL,
    filename TEXT NOT NULL,
    snapshot TEXT NOT NULL,
    request_params_json TEXT NOT NULL DEFAULT '{}',
    context_params_json TEXT NOT NULL DEFAULT '[]',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

### 2.2 Update UserAlias struct
**File**: `crates/objs/src/user_alias.rs`

Add `id: String`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>` fields.
- `#[builder(setter(skip))]` on `id`, `created_at`, `updated_at` (same pattern as `ApiAliasBuilder` in `crates/objs/src/api_model_alias.rs:134-166`)
- Add `build_with_time(now: DateTime<Utc>) -> Result<UserAlias>` method on `UserAliasBuilder` that generates UUID and sets timestamps
- Remove `config_filename()` method
- Remove `to_safe_filename` import (still used by ModelAlias, stays in utils.rs)
- Update/remove YAML serialization tests

### 2.3 Update test builders
**File**: `crates/objs/src/test_utils/objs.rs`
- Update `AliasBuilder` factory methods and `UserAlias::testalias()`, `UserAlias::llama3()`, `UserAlias::tinyllama()` to use `build_with_time()` with fixed timestamps

### 2.4 Create UserAliasRepository trait
**New file**: `crates/services/src/db/user_alias_repository.rs`

Follow `ModelRepository` pattern (`crates/services/src/db/model_repository.rs`):
```rust
#[async_trait]
pub trait UserAliasRepository: Send + Sync {
    async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError>;
    async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError>;
    async fn get_user_alias_by_name(&self, alias: &str) -> Result<Option<UserAlias>, DbError>;
    async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError>;
    async fn delete_user_alias(&self, id: &str) -> Result<(), DbError>;
    async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError>;
}
```
- Add `mod user_alias_repository` + `pub use` in `crates/services/src/db/mod.rs`
- Add `UserAliasRepository` to `DbService` supertrait in `crates/services/src/db/service.rs`

### 2.5 Implement UserAliasRepository on SqliteDbService
**File**: `crates/services/src/db/service.rs`
- Follow API alias SQL patterns (lines 339-685 of same file)
- Store `request_params` as JSON (`serde_json::to_string`), `context_params` as JSON array, `repo` as String

### 2.6 Rewrite DataService trait
**File**: `crates/services/src/data_service.rs`

New trait definition:
```rust
#[async_trait]
pub trait DataService: Send + Sync + std::fmt::Debug {
    async fn list_aliases(&self) -> Result<Vec<Alias>>;
    async fn find_alias(&self, alias: &str) -> Option<Alias>;
    async fn find_user_alias(&self, alias: &str) -> Option<UserAlias>;
    async fn get_user_alias_by_id(&self, id: &str) -> Option<UserAlias>;
    async fn save_alias(&self, alias: &UserAlias) -> Result<()>;
    async fn copy_alias(&self, id: &str, new_alias: &str) -> Result<UserAlias>;
    async fn delete_alias(&self, id: &str) -> Result<()>;
}
```

Remove: `alias_filename()`, `read_file()`, `write_file()`, `find_file()`, `aliases_dir()`, `construct_path()`, private `list_user_aliases()` YAML reader, all `fs::*` operations

### 2.7 Remove ALIASES_DIR and aliases_dir()
- `crates/services/src/setting_service/mod.rs` -- remove `ALIASES_DIR` constant
- `crates/services/src/setting_service/service.rs` -- remove `aliases_dir()` from SettingService trait
- `crates/lib_bodhiserver/src/app_dirs_builder.rs` -- remove alias directory creation from `setup_bodhi_subdirs()` (lines ~228-234) and update test assertions

### 2.8 Clean up DataServiceError
- Remove `SerdeYamlError` variant if only used for YAML parsing
- Remove `FileNotFound` variant if only used for `models.yaml` / file ops
- Keep `AliasNotFound`, `AliasExists`

### 2.9 Update TestDataService + test fixtures
- `crates/services/src/test_utils/data.rs` -- match new trait, delegate to inner
- `crates/objs/src/test_utils/bodhi.rs` -- `temp_bodhi_home` no longer copies alias YAML files
- Delete `crates/objs/tests/data/bodhi/aliases/` directory and all YAML files
- Seed aliases into DB in `test_data_service` fixture
- Add `seed_test_user_aliases()` helper in `crates/services/src/test_utils/objs.rs` (follow `seed_test_api_models()` pattern)

### 2.10 Update tests
- Rewrite alias tests in `data_service.rs` for DB-backed storage
- Remove file I/O tests (`read_file`/`write_file`/`find_file`)
- Add UserAliasRepository tests in `crates/services/src/db/tests.rs`
- Check if `serde_yaml` dependency can be removed from services Cargo.toml

### Verification
```bash
cargo test -p objs && cargo test -p services
```
Note: routes_app and routes_all may NOT compile yet -- that's expected.

---

## Phase 3: Route Handlers + Server Changes (routes_app + routes_all + server_core)
**Status**: [x] Complete
**Sub-agent type**: `general-purpose`
**Validation**: `cargo test`

### Context for sub-agent
Fix all route handlers and server code to work with the new DB-backed UserAlias. Add UUID-based endpoints. All crates must compile and all tests pass.

### 3.1 Update UserAliasResponse DTO
**File**: `crates/routes_app/src/api_dto.rs`

Add `id: String`, `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>` to `UserAliasResponse`. Update `From<UserAlias>` impl.

### 3.2 Update alias route handlers
**File**: `crates/routes_app/src/routes_models/aliases.rs`

- `get_user_alias_handler`: Path param is UUID, use `data_service.get_user_alias_by_id()`
- `create_alias_handler`: generate UUID via `build_with_time(time_service.utc_now())`, use async `save_alias()`
- `update_alias_handler`: path param is UUID, fetch by UUID, update, save
- **NEW** `delete_alias_handler`: `DELETE /api/models/{uuid}` -> `data_service.delete_alias(uuid)`
- **NEW** `copy_alias_handler`: `POST /api/models/{uuid}/copy` with `{"alias": "new-name"}` body

### 3.3 Update OAI/Ollama model timestamps
- `crates/routes_app/src/routes_oai/models.rs:169` -- `user_alias_to_oai_model()`: replace `bodhi_home.join("aliases").join(alias.config_filename())` with `alias.created_at.timestamp()`
- `crates/routes_app/src/routes_ollama/handlers.rs:82` -- `user_alias_to_ollama_model()`: same change

### 3.4 Update async callers
All callers of `find_user_alias()` need `.await` added:
- `get_user_alias_handler`, `create_alias_handler`, `update_alias_handler` in aliases.rs
- `execute_pull_by_alias` is already removed (Phase 1)
- Check pull.rs for any remaining `find_user_alias` calls

### 3.5 Update route registration
**File**: `crates/routes_all/src/routes.rs`

Power user APIs -- add:
```rust
.route(&format!("{ENDPOINT_MODELS}/{{id}}"), delete(delete_alias_handler))
.route(&format!("{ENDPOINT_MODELS}/{{id}}/copy"), post(copy_alias_handler))
```
Import new handlers from routes_app.

### 3.6 Update OpenAPI
**File**: `crates/routes_app/src/shared/openapi.rs`
- Add `__path_delete_alias_handler` and `__path_copy_alias_handler`
- Add utoipa annotations to new handlers

### 3.7 Update tests
- `crates/server_core/src/model_router.rs` -- MockDataService tests, verify compilation
- `crates/routes_app/src/routes_models/tests/aliases_test.rs` -- update for UUID paths and new mock signatures
- `crates/routes_app/src/test_utils/alias_response.rs` -- add id/timestamps to factory methods
- Add tests for delete_alias_handler and copy_alias_handler

### Verification
```bash
cargo test
```

---

## Phase 4: UI Rebuild + Integration Tests
**Status**: [x] Complete (user-driven)
**Sub-agent type**: Manual (user fixed remaining issues)
**Validation**: `make test.backend` + `make test.ui`

### Context for sub-agent
Regenerate OpenAPI specs, rebuild UI, run integration tests, fix any failures.

### Steps
1. `cargo run --package xtask openapi` -- regenerate OpenAPI spec (done in Phase 3)
2. `cd ts-client && npm run generate` -- regenerate TypeScript types
3. `make build.ui-rebuild` -- rebuild embedded UI
4. `make test.ui` -- run frontend tests
5. Fix any failures (frontend code referencing old response shape, endpoint paths)
6. `make test.backend` -- backend test suite

### Deviations
- OpenAPI spec was regenerated during Phase 3 (not Phase 4)
- User manually fixed remaining test.backend and test.ui failures
- E2e test fix: `editLocalModel` in Playwright tests changed from `searchParams.get('alias')` to `searchParams.get('id')` (UUID-based), added `data-test-model-id` attribute to table rows (see `tidy-wiggling-reef.md`)

### Verification
```bash
make test.backend  # passed
make test.ui       # passed
```

---

## Existing Patterns to Reuse

| Pattern | Reference File | Reuse For |
|---------|---------------|-----------|
| Repository trait | `crates/services/src/db/model_repository.rs` | UserAliasRepository trait |
| SqliteDbService CRUD | `crates/services/src/db/service.rs:339-685` | SQL implementation |
| UUID generation | `uuid::Uuid::new_v4().to_string()` (DownloadRequest) | UserAlias id |
| Builder with time | `crates/objs/src/api_model_alias.rs:134-166` | UserAliasBuilder |
| JSON column storage | `models_json` in api_model_aliases table | request_params_json/context_params_json |
| Test seeding | `seed_test_api_models()` in `crates/services/src/test_utils/objs.rs` | seed_test_user_aliases() |
| TimeService | `time_service.utc_now()` everywhere | Timestamps |

## Progress Tracking

| Phase | Status | Commit | Deviations |
|-------|--------|--------|------------|
| 1: Remove RemoteModel | [x] Done | 539d8e99 | None -- clean removal, 1322 tests pass |
| 2: objs + services migration | [x] Done | bf21a841 | `created_at`/`updated_at` stored as INTEGER (Unix timestamp) not TIMESTAMP; added `build_test()` and `build_with_id()` convenience methods to UserAliasBuilder; `serde_yaml` kept in services (used by settings/secrets) |
| 3: routes + server changes | [x] Done | 2fab4aa2 | `update_alias_handler` builds UserAlias directly instead of going through `execute_create_alias`; test assertions changed from full struct equality to field-by-field (timestamps/UUIDs are dynamic); chat completion tests use predicate matching instead of exact `Alias` equality |
| 4: UI rebuild + integration | [x] Done | (pending) | User fixed remaining failures; e2e `editLocalModel` changed from alias-based to UUID-based URL params |
