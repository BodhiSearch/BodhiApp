# Remove `models_cache` from ApiAlias

## Context

`ApiAlias` has two model-list fields: `models` (primary) and `models_cache` (populated only via `/sync` endpoint). Previously `models_cache` was needed because `forward_all_with_prefix` mode didn't populate `models` with all provider models at create/update time. Now both modes populate `models` at create/update, making `models_cache` redundant. Keeping it adds confusion and maintenance burden.

Additionally, the frontend caps models at 20 — needs to be raised to 1000.

## Phase 1: Services Crate

### 1.1 Remove fields from `ApiAlias` struct
**File**: `crates/services/src/models/model_objs.rs`
- Remove `models_cache: ApiModelVec` field (line 801) and `cache_fetched_at` field (line 804) with their annotations
- Remove from `ApiAlias::new()` (lines 834-835): `models_cache: ApiModelVec::default()`, `cache_fetched_at: epoch_sentinel()`
- Remove from `ApiAliasBuilder::build_with_time()` (lines 909-910): `models_cache`, `cache_fetched_at`
- Remove `is_cache_stale()` (line 874-876) and `is_cache_empty()` (line 879-881)
- Remove `CACHE_TTL_HOURS` constant (line 753)
- Remove `epoch_sentinel()` function (lines 813-815) — only used by cache fields
- Update `ApiAliasResponse.models` doc comment (line 1503) — remove "merged from cache" reference

### 1.2 Remove fields from DB entity
**File**: `crates/services/src/models/api_model_alias_entity.rs`
- Remove `models_cache` (line 21) and `cache_fetched_at` (line 22) from `Model`
- Remove same from `ApiAliasView` (lines 49-50)
- Remove from `From<ApiAliasView>` impl (lines 64-65)

### 1.3 Repurpose repository method
**File**: `crates/services/src/models/api_alias_repository.rs`
- Rename trait method `update_api_model_cache` → `update_api_model_models` (line 38). Remove `fetched_at` param.
- Rename impl (lines 251-278). Change to update `models` column + `updated_at` instead of `models_cache` + `cache_fetched_at`
- In `create_api_model_alias` impl (line 123): remove `models_cache: Set(...)` and `cache_fetched_at: Set(...)`

### 1.4 Add validation error variant
**File**: `crates/services/src/shared_objs/error_wrappers.rs`
- Add to `ObjValidationError`:
  ```rust
  #[error("Sync models is only available for aliases with forward_all_with_prefix enabled.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  SyncRequiresForwardAll,
  ```

### 1.5 Repurpose `sync_cache` → `sync_models`
**File**: `crates/services/src/models/api_model_service.rs`
- Rename trait method `sync_cache` → `sync_models` (line 84)
- Rewrite impl (lines 304-348):
  1. Fetch alias, check `forward_all_with_prefix == true` (else return `SyncRequiresForwardAll` error)
  2. Fetch API key, fetch models from remote
  3. Call `db_service.update_api_model_models(tenant_id, id, models)` (updates `models` column)
  4. Re-fetch and return `ApiAliasResponse`

### 1.6 Update auth-scoped service
**File**: `crates/services/src/models/auth_scoped_api_models.rs`
- Rename `sync_cache` → `sync_models` (line 70), update inner call

### 1.7 Update TestDbService
**File**: `crates/services/src/test_utils/db.rs`
- Rename `update_api_model_cache` → `update_api_model_models` (lines 274-285, 1310). Remove `fetched_at` param.

### 1.8 DB Migration
**New file**: `crates/services/src/db/sea_migrations/m20250101_000019_drop_models_cache.rs`
- Drop columns `models_cache` and `cache_fetched_at` from `api_model_aliases`
- Register in `sea_migrations/mod.rs`

### 1.9 Services tests
- `test_api_alias_repository.rs`: Remove `models_cache`/`cache_fetched_at` from fixtures. Rename/update `test_update_api_model_cache` → `test_update_api_model_models`.
- `test_api_alias_repository_isolation.rs`: Remove from `make_alias()`
- `test_model_objs.rs`: Remove `test_api_alias_new_epoch_sentinel` and any cache-related tests

**Gate**: `cargo test -p services --lib`

## Phase 2: Routes App Crate

### 2.1 Anthropic handler
**File**: `crates/routes_app/src/anthropic/routes_anthropic.rs`
- Line 134: `alias.models.iter().chain(alias.models_cache.iter())` → `alias.models.iter()`

### 2.2 Sync endpoint handler
**File**: `crates/routes_app/src/models/api/routes_api_models.rs`
- Line 358: `.sync_cache(&id)` → `.sync_models(&id)`

### 2.3 Stale comment
**File**: `crates/routes_app/src/oai/routes_oai_models.rs`
- Line 77: Remove cache reference from comment

### 2.4 Dev routes
**File**: `crates/routes_app/src/routes_dev.rs`
- Line 327: Remove `models_cache: vec![].into()` and `cache_fetched_at` from fixture

### 2.5 Routes tests
- `test_api_models_sync.rs`: Update to use `sync_models`. Add test `test_sync_models_rejects_non_forward_all` (create non-forward_all alias, POST sync, expect 400).

**Gate**: `cargo test -p routes_app --lib`

## Phase 3: Frontend

### 3.1 Validation limits
**File**: `crates/bodhi/src/schemas/apiModel.ts`
- Line 31: `.max(20, ...)` → `.max(1000, 'Maximum 1000 models allowed')`
- Line 88: `.max(20, ...)` → `.max(1000, 'Maximum 1000 models allowed')`

**Gate**: `cd crates/bodhi && npm test`

## Phase 4: OpenAPI + TypeScript Regeneration

1. `cargo run --package xtask openapi`
2. `make build.ts-client`

Removes `models_cache` and `cache_fetched_at` from `openapi.json`, `openapi-schema.ts`, `types.gen.ts`.

## Phase 5: Full Verification

1. `cargo test -p services --lib -p routes_app -p server_app` — backend tests
2. `cargo test -p routes_app -- openapi` — OpenAPI spec matches
3. `cd crates/bodhi && npm test` — frontend tests
4. `make build.ui-rebuild` — rebuild embedded UI
5. `make app.run` — smoke test: create forward_all alias, sync models, verify models list
6. E2E: `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-forward-all.spec.mjs` — no changes expected (doesn't reference `models_cache`)

## Files Modified (Summary)

| File | Change |
|------|--------|
| `services/src/models/model_objs.rs` | Remove fields, methods, constants |
| `services/src/models/api_model_alias_entity.rs` | Remove DB entity fields |
| `services/src/models/api_alias_repository.rs` | Rename + repurpose repo method |
| `services/src/models/api_model_service.rs` | Rename sync_cache → sync_models, add guard |
| `services/src/models/auth_scoped_api_models.rs` | Rename method |
| `services/src/shared_objs/error_wrappers.rs` | Add SyncRequiresForwardAll variant |
| `services/src/test_utils/db.rs` | Rename mock method |
| `services/src/db/sea_migrations/` | New migration + register |
| `routes_app/src/anthropic/routes_anthropic.rs` | Remove chain with models_cache |
| `routes_app/src/models/api/routes_api_models.rs` | Rename handler call |
| `routes_app/src/oai/routes_oai_models.rs` | Fix comment |
| `routes_app/src/routes_dev.rs` | Remove fixture fields |
| `bodhi/src/schemas/apiModel.ts` | 20 → 1000 |
| Test files (4-5 files) | Update fixtures, add rejection test |
| `openapi.json`, `ts-client/` | Regenerated |
