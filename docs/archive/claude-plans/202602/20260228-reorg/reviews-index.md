# Review Index — objs→services Merge + db→domain Moves

## Scope

- Commits: HEAD~1 (errmeta creation + objs merge into services) + HEAD (db→domain moves)
- Date: 2026-03-01
- Crates reviewed: `errmeta`, `errmeta_derive`, `services` (all modules), `auth_middleware`, `server_core`, `routes_app`, `server_app`, `lib_bodhiserver`, `bodhi/src-tauri`, `llama_server_proc`, `mcp_client`
- Total findings: 105 across 8 review files

## Quick Summary by Priority

| Priority | Count | Review Files |
|----------|-------|--------------|
| Critical | 5 | services-batch-c-review.md, cross-cutting-review.md |
| Important | 57 | all 8 review files |
| Nice-to-have | 43 | all 8 review files |

---

## Topic: Logic Bugs & Correctness Issues

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 1 | `Alias::can_serve` ignores prefix-aware routing for `Api` variant — uses `models.contains()` instead of `supports_model()` | services-batch-c-review.md#Finding-1 | Critical | Fix: replace `api_alias.models.contains(...)` with `api_alias.supports_model(model)` in `crates/services/src/models/model_objs.rs` |
| 2 | `update_toolset` returns input clone instead of DB-roundtripped row — inconsistent with all other mcps update ops | services-batch-d-review.md#Finding-3 | Important | Fix: `toolsets/toolset_repository.rs` use `active.update(&self.db)` instead of `Entity::update(...).exec` + `Ok(row.clone())` |
| 3 | `update_api_token` constructs `sea_orm::DbErr::RecordNotFound` directly — bypasses `DbError` abstraction | services-batch-a-review.md#Finding-5 | Important | Fix: `tokens/token_repository.rs` use `DbError::ItemNotFound{id, item_type}` |
| 4 | `args_delegate = false` on non-transparent variants silently ignored — no compile-time error | errmeta-review.md#Finding-14 | Important | Fix: add diagnostic in `errmeta_derive/src/generate.rs` `generate_args_method` |
| 5 | `Expr::cust("CASE WHEN enabled = true")` — not portable to SQLite (boolean literal syntax) | services-batch-d-review.md#Finding-10 | Important | Fix: `mcps/mcp_server_repository.rs` use `enabled = 1` or SeaORM typed expressions |
| 6 | `if request_id.contains("approve")` branch logic inside parameterized test body | services-batch-b-review.md#Finding-12 | Important | Fix: `app_access_requests/test_access_request_service.rs` split into two typed parameterized tests |
| 7 | Duplicate epoch sentinel construction — `parse_from_rfc3339(...).unwrap()` in two places | services-batch-c-review.md#Finding-2 | Important | Fix: `models/model_objs.rs` extract `fn epoch_sentinel() -> DateTime<Utc>` using `DateTime::UNIX_EPOCH` |
| 8 | `AppError::status()` fallback to 500 on unknown `error_type()` strings is untested | errmeta-review.md#Finding-5 | Important | Add unit test in `errmeta/src/` for unrecognized `error_type()` producing 500 |

---

## Topic: db/ Backward-Compat Shims (Highest Priority Structural Issue)

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 9 | `db/objs.rs` defines `ApiKeyUpdate` (domain type) and re-exports 20+ domain types — all should be removed | services-infra-review.md#Finding-1, services-infra-review.md#Finding-2 | Important | `ApiKeyUpdate` → `models/`; delete `db/objs.rs`; 64 callers in 23 files |
| 10 | `db/mod.rs` backward-compat re-exports of 20+ domain types — should be eliminated | services-infra-review.md#Finding-3, cross-cutting-review.md#Finding-11 | Important | 40+ usages in `auth_middleware` alone; update all callers first |
| 11 | Downstream crates use `services::db::<DomainType>` instead of `services::<DomainType>` — 64 references | services-infra-review.md#Finding-15 | Important | Files: `auth_middleware` (11 files), `routes_app` (multiple); update before removing shims |
| 12 | `routes_app` imports `DownloadStatus` from `services::db` (stale path) | cross-cutting-review.md#Finding-19 | Important | Fix: `routes_app/src/shared/openapi.rs` change to `use services::DownloadStatus` |
| 13 | `tokens/api_token_entity.rs` imports `TokenStatus` from `crate::db` (stale path) | cross-cutting-review.md#Finding-17 | Important | Fix: change to `use crate::auth::TokenStatus` or `use crate::TokenStatus` |
| 14 | `access_request_service.rs` imports domain types via `crate::db::` inside own module | services-batch-b-review.md#Finding-15 | Nice-to-have | Fix: use `super::` or `crate::` direct paths |
| 15 | `mcp_service.rs` imports own domain row types via `crate::db::` re-export path | services-batch-d-review.md#Finding-7 | Nice-to-have | Also: `tool_service.rs` imports `ToolsetRow` via `crate::db::ToolsetRow` |
| 16 | `test_utils/objs.rs` uses `crate::db::ModelMetadataRow` — wrong path after domain move | services-infra-review.md#Finding-7 | Important | Fix 4 return type annotations in `test_utils/objs.rs` |
| 17 | `db/mod.rs` uses `pub use objs::*` wildcard over cross-domain shim file | services-infra-review.md#Finding-12 | Important | Replace with explicit re-exports or delete with `db/objs.rs` |
| 18 | `test_utils/objs.rs` in `test_utils/mod.rs` re-exported as `pub use objs::*` — confusing with deleted crate | services-infra-review.md#Finding-13 | Important | Rename `test_utils/objs.rs` to `test_utils/fixtures.rs` |
| 19 | `db/model_repository.rs` backward-compat supertrait — no external callers | services-infra-review.md#Finding-4, cross-cutting-review.md#Finding-10 | Nice-to-have | Remove or move to `models/` |
| 20 | `routes_app/routes_toolsets/types.rs` duplicates `ApiKeyUpdateDto`; `routes_api_models/types.rs` has `ApiKeyUpdateAction` — both mirror `ApiKeyUpdate` | services-infra-review.md#Finding-16 | Nice-to-have | Fix when moving `ApiKeyUpdate` to `models/` with serde+utoipa derives |

---

## Topic: Module Organization Violations

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 21 | `token.rs` orphan at crate root — JWT claims and `TokenError` should be in `tokens/` | services-infra-review.md#Finding-6 | Important | `src/token.rs` → `tokens/claims.rs` + `tokens/token_error.rs`; remove `mod token;` from `lib.rs` |
| 22 | `users` module declared `pub mod users` in `lib.rs` — all other domain modules are private | services-batch-b-review.md#Finding-9 | Important | Fix: change to `mod users;` in `crates/services/src/lib.rs:34` |
| 23 | `AppInstanceError` defined inline in `app_instance_service.rs` — should be in `apps/error.rs` | services-batch-a-review.md#Finding-1 | Important | Extract to `crates/services/src/apps/error.rs` |
| 24 | `tokens/` module has no domain error type — returns raw `DbError` | services-batch-a-review.md#Finding-4 | Important | Create `TokenServiceError` in `crates/services/src/tokens/error.rs` |
| 25 | `SettingsMetadataError` defined in `setting_objs.rs` — should be in `settings/error.rs` | services-batch-a-review.md#Finding-9 | Important | Move to `crates/services/src/settings/error.rs` |
| 26 | `AiApiServiceError` inline in `ai_api_service.rs` (768 lines) — no `error.rs` | services-batch-c-review.md#Finding-10, cross-cutting-review.md#Finding-14 | Important | Create `crates/services/src/ai_apis/error.rs` |
| 27 | `ExaError` defined in `exa_service.rs` — should be in `toolsets/error.rs` | services-batch-d-review.md#Finding-11 | Nice-to-have | Move to `crates/services/src/toolsets/error.rs` |
| 28 | `AppAccessRequestRow` defined in entity file — should be in `access_request_objs.rs` | services-batch-b-review.md#Finding-6 | Important | Move struct to `crates/services/src/app_access_requests/access_request_objs.rs` |
| 29 | `UserAccessRequestStatus` defined in `app_access_requests/access_request_objs.rs` — belongs in `users/` | services-batch-b-review.md#Finding-8 | Important | Move to `crates/services/src/users/user_objs.rs` (new file) |
| 30 | `TokenStatus` defined in `auth/auth_objs.rs` — belongs in `tokens/` | services-batch-a-review.md#Finding-7 | Nice-to-have | Move to `tokens/` alongside the token entity |
| 31 | `db/default_service.rs` calls `seed_toolset_configs()` — domain seeding logic in infrastructure layer | services-infra-review.md#Finding-5 | Important | Move seed logic to `toolsets/`; call from `lib_bodhiserver` bootstrap |
| 32 | Repository test for `toolsets` declared from `mod.rs` instead of source file (Pattern A) | services-batch-d-review.md#Finding-4 | Nice-to-have | Move `#[cfg(test)]` declaration from `toolsets/mod.rs` to `toolset_repository.rs` |
| 33 | Repository tests declared from `mod.rs`, service tests from source file — inconsistent within same module | services-batch-a-review.md#Finding-3 | Nice-to-have | Applies to `apps/`, `tokens/`, `settings/` modules |

---

## Topic: Missing Derives & Type Issues

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 34 | `ErrorType` missing `PartialEq`, `Clone`, `Eq` derives — forces string comparisons everywhere | errmeta-review.md#Finding-7 | Important | Add to `#[derive(...)]` in `crates/errmeta/src/error_type.rs` |
| 35 | `ApiToken` is a type alias for SeaORM `Model` — no separation between domain type and entity | services-batch-a-review.md#Finding-6 | Important | Introduce `ApiTokenRow` in `tokens/` (follow `AppInstanceRow` pattern) |
| 36 | `AppAccessRequestRow` missing `Serialize`/`Deserialize` derives | services-batch-b-review.md#Finding-7 | Important | Add `#[derive(Serialize, Deserialize)]` in `app_access_request_entity.rs` |
| 37 | `AppInstanceRow` missing `PartialEq` | services-batch-a-review.md#Finding-2 | Nice-to-have | Add to `crates/services/src/apps/app_instance_entity.rs` |
| 38 | `model_metadata_entity.source` is untyped `String` — should be `AliasSource` | services-batch-c-review.md#Finding-11 | Important | Change field type + remove `.to_string()` calls in `model_metadata_repository.rs` |
| 39 | `ToSchema` derived on SeaORM entity models — conflates persistence and API concerns | services-batch-c-review.md#Finding-12 | Nice-to-have | `download_request_entity.rs`, `model_metadata_entity.rs` |
| 40 | `Box<dyn AppError>` has no `Display` impl — calling `.to_string()` will fail to compile | errmeta-review.md#Finding-6 | Nice-to-have | Add `impl std::fmt::Display for Box<dyn AppError>` in `errmeta/src/app_error.rs` |

---

## Topic: Dead Code & Stale References

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 41 | `use objs::ErrorMetas` in `test_args_as_expr.rs` — misleading holdover from deleted crate | errmeta-review.md#Finding-1 | Important | Fix: `crates/errmeta_derive/tests/test_args_as_expr.rs:4` → `use crate::objs::ErrorMetas` |
| 42 | `RwLockReadError` has zero downstream users | errmeta-review.md#Finding-11 | Nice-to-have | Verify intent; remove from `errmeta/src/rwlock_error.rs` if dead |
| 43 | Dead variant `KcUuidCollision` in `AccessRequestError` — never constructed | services-batch-b-review.md#Finding-13 | Nice-to-have | Remove from `crates/services/src/app_access_requests/error.rs` |
| 44 | `ObjValidationError` variant in `HubServiceError` is dead — no code produces it | services-batch-c-review.md#Finding-4 | Important | Remove variant + import from `crates/services/src/models/hub_service.rs` |
| 45 | `ObjValidationError` and `ModelValidationError` are duplicates — incomplete split | services-batch-c-review.md#Finding-3 | Important | Consolidate: remove `ObjValidationError`'s model-specific variants or delete entirely |
| 46 | `GGMLQuantizationType` enum unused — 30+ variants, 0 call sites | services-batch-c-review.md#Finding-13 | Nice-to-have | Remove or gate with `#[allow(dead_code)]` + comment in `models/gguf/constants.rs` |
| 47 | Stale "moved from objs" migration comments in source files — 12 instances | cross-cutting-review.md#Finding-15 | Nice-to-have | `crates/services/src/lib.rs:7`, `models/model_objs.rs` (11 occurrences) |
| 48 | Dead `AliasBuilder` type alias with stale backward-compat comment | cross-cutting-review.md#Finding-16 | Nice-to-have | Remove from `crates/services/src/test_utils/objs.rs:12` |
| 49 | `#[allow(dead_code)]` + `#[allow(unused_imports)]` on active `external_token` module | downstream-crates-review.md#Finding-2 | Important | Remove both attributes from `crates/server_app/tests/utils/mod.rs` |
| 50 | `#[allow(unused_imports)]` on `alias_response` re-export — likely dead test helpers | downstream-crates-review.md#Finding-3 | Nice-to-have | `crates/routes_app/src/test_utils/mod.rs:6` — audit callers, remove if dead |
| 51 | `#[allow(unused)]` on `build_frontend` in `lib_bodhiserver/build.rs` | downstream-crates-review.md#Finding-4 | Nice-to-have | Restructure or add explanatory comment |
| 52 | Stale `objs=trace` filter string in tracing fixture | downstream-crates-review.md#Finding-5 | Nice-to-have | `crates/services/src/test_utils/logs.rs:11` — remove `objs=trace` |
| 53 | Stale doc comment references `objs::gguf::` module path | services-batch-c-review.md#Finding-5, downstream-crates-review.md#Finding-6 | Important | `crates/services/src/utils/queue_service.rs:43-44` — update to `crate::models::gguf::` |

---

## Topic: Test Convention Violations

### `use super::*` (11 total instances in services)

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 54 | `use super::*` in auth objs test files (3 instances) | services-batch-b-review.md#Finding-1 | Important | `auth/test_auth_objs_role.rs:1`, `test_auth_objs_token_scope.rs:1`, `test_auth_objs_user_scope.rs:1` |
| 55 | `use super::*` in mcps and toolsets test files (3 instances) | services-batch-d-review.md#Finding-1 | Important | `mcps/test_mcp_objs_types.rs:1`, `test_mcp_objs_validation.rs:1`, `toolsets/test_exa_service.rs:1` |
| 56 | `use super::*` in `shared_objs/utils.rs` and `shared_objs/log.rs` inline test modules | services-batch-c-review.md#Finding-7 | Important | `crates/services/src/shared_objs/utils.rs:24`, `log.rs:81` |

### Error assertion style

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 57 | Error message string asserted instead of `.code()` — 3 instances in auth tests | services-batch-b-review.md#Finding-2 | Important | `test_auth_objs_role.rs:217`, `test_auth_objs_token_scope.rs:92`, `test_auth_objs_user_scope.rs:92` |
| 58 | `matches!(result.unwrap_err(), ToolsetError::X(_))` instead of `.code()` assertions — 4 instances | services-batch-d-review.md#Finding-5 | Important | `toolsets/test_tool_service.rs` |
| 59 | Redundant `assert!(result.is_err())` before `result.unwrap_err()` — 5 instances | services-batch-d-review.md#Finding-12 | Nice-to-have | `mcps/test_mcp_service.rs:950,1039,1116,1305,1390` |
| 60 | `assert_eq!(false, ...)` instead of `assert!(! ...)` | services-batch-b-review.md#Finding-5 | Nice-to-have | `test_auth_service.rs:661-662` |

### Missing `pretty_assertions::assert_eq`

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 61 | Missing `use pretty_assertions::assert_eq` in `test_setting_objs.rs` | services-batch-a-review.md#Finding-11 | Nice-to-have | `crates/services/src/settings/test_setting_objs.rs` |
| 62 | Missing `use pretty_assertions::assert_eq` in `test_setting_service_db.rs` | services-batch-a-review.md#Finding-12 | Nice-to-have | `crates/services/src/settings/test_setting_service_db.rs` |
| 63 | Missing `use pretty_assertions::assert_eq` in 2 mcp objs test files | services-batch-d-review.md#Finding-8 | Nice-to-have | `mcps/test_mcp_objs_types.rs`, `test_mcp_objs_validation.rs` |

### `anyhow_trace` and return type conventions

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 64 | `test_setting_service_db.rs` — 5 async tests return `()` without `anyhow_trace` | services-batch-a-review.md#Finding-10 | Important | All 5 functions in `crates/services/src/settings/test_setting_service_db.rs` |
| 65 | `test_token_service.rs` uses `.unwrap()` instead of `?` | services-batch-a-review.md#Finding-8 | Nice-to-have | 8 `.unwrap()` calls across token test files |

### Non-rstest / if-else in tests

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 66 | 6 redundant non-rstest `#[test]` functions duplicating parameterized coverage | services-batch-b-review.md#Finding-3 | Nice-to-have | 3 auth objs test files — serialization/deserialization tests |
| 67 | Single-variant `CrudOperation` enum with if-else branch in test body | services-batch-a-review.md#Finding-14 | Nice-to-have | `crates/services/src/settings/test_setting_service.rs` |
| 68 | `println!` debug logging in non-error test path | services-batch-a-review.md#Finding-13 | Nice-to-have | `test_setting_service.rs:1070-1079` — 4 `println!` calls |

### Inline tests exceeding 500-line threshold

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 69 | `data_service.rs` (556 lines) uses inline tests — should use `test_data_service.rs` sibling | services-batch-c-review.md#Finding-15, cross-cutting-review.md#Finding-13 | Nice-to-have | `crates/services/src/models/data_service.rs` |
| 70 | `progress_tracking.rs` uses inline tests despite sibling-file pattern in same module | services-batch-c-review.md#Finding-15 | Nice-to-have | `crates/services/src/models/progress_tracking.rs` |
| 71 | `ai_api_service.rs` (768 lines) uses inline tests | cross-cutting-review.md#Finding-14 | Important | `crates/services/src/ai_apis/ai_api_service.rs` (same file also missing `error.rs`) |

### Other test issues

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 72 | Test helper functions (`make_server`, `make_mcp`, `make_auth_header_row`) duplicated across 3 mcps test files | services-batch-d-review.md#Finding-2 | Important | Extract to shared `test_helpers` module in `mcps/` |
| 73 | Hardcoded `ENCRYPTION_KEY` constant instead of `ctx.service.encryption_key` — 2 mcps test files | services-batch-d-review.md#Finding-6 | Nice-to-have | `test_mcp_auth_repository.rs:13`, `test_mcp_instance_repository.rs:50` |
| 74 | Weak validation assertions — `assert!(is_err())` alone without error message check (~20 instances) | services-batch-d-review.md#Finding-9 | Nice-to-have | `mcps/test_mcp_objs_validation.rs`, `toolsets/test_toolset_objs.rs` |
| 75 | `DownloadStatus` imported via `crate::db` in models test — should use `crate::models` | services-batch-c-review.md#Finding-14 | Nice-to-have | `crates/services/src/models/test_download_repository.rs:2` |

---

## Topic: Utc::now() in Tests (Non-Determinism)

| # | Issue | File#Finding | Priority | Files |
|---|-------|--------------|----------|-------|
| 76 | `AppInstance::test_default()` uses `Utc::now()` for timestamps | services-batch-c-review.md#Finding-8, services-infra-review.md#Finding-9 | Important | `crates/services/src/test_utils/objs.rs:26-27` — replace with `fixed_dt()` |
| 77 | `ai_api_service.rs` tests use `Utc::now()` when constructing `ApiAlias` | services-batch-c-review.md#Finding-9 | Important | `crates/services/src/ai_apis/ai_api_service.rs:495,566` |
| 78 | `DbSetting` constructed with sentinel `UNIX_EPOCH` timestamps — API implies caller controls timestamps but doesn't | services-batch-a-review.md#Finding-15 | Nice-to-have | `settings/default_service.rs:314-320`, `test_setting_service_db.rs:34-35` |

---

## Topic: Documentation Staleness

### Root CLAUDE.md

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 79 | Wrong `TimeService` file path — says `db/service.rs`, actual is `db/time_service.rs` | cross-cutting-review.md#Finding-1 | Important | `CLAUDE.md` "Architectural Patterns" section |
| 80 | Ghost `commands` crate listed in "Key Crates Structure" — does not exist | cross-cutting-review.md#Finding-2 | Important | Remove or replace with actual CLI location |
| 81 | Dependency chain diagram shows `mcp_client` depending on `errmeta` — false | cross-cutting-review.md#Finding-3 | Important | `mcp_client` has no `errmeta` dependency |
| 82 | `services` keywords table missing `users/` domain module | cross-cutting-review.md#Finding-4 | Nice-to-have | Add `users/access_repository.rs` to table |

### Downstream CLAUDE.md files (8 files with ~14 stale `objs` references)

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 83 | 8 CLAUDE.md files still reference deleted `objs` crate as upstream dependency | downstream-crates-review.md#Finding-1 | Important | `auth_middleware`, `server_core`, `routes_app`, `server_app`, `lib_bodhiserver`, `bodhi/src-tauri`, `llama_server_proc` CLAUDE.md files |

### errmeta_derive CLAUDE.md

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 84 | Stale reference to deleted `objs` crate as foundation user | cross-cutting-review.md#Finding-5 | Important | `crates/errmeta_derive/CLAUDE.md` — replace "objs crate" with "services crate" |

### services CLAUDE.md

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 85 | Claims "all entities live in `src/db/entities/`" — directory does not exist | cross-cutting-review.md#Finding-6 | Critical | `crates/services/CLAUDE.md` — update to domain module paths |
| 86 | `session_service/` described as subdirectory — actually flat files in `auth/` | cross-cutting-review.md#Finding-7 | Important | `crates/services/CLAUDE.md` Session Security section |
| 87 | `users/` domain module entirely undocumented in CLAUDE.md | cross-cutting-review.md#Finding-21 | Important | Add `### User Management (users/)` section |

### services PACKAGE.md

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 88 | `db/service.rs` description wrong on all counts — `TimeService` moved to `time_service.rs`, repositories moved to domain modules | cross-cutting-review.md#Finding-8 | Critical | Rewrite `db/service.rs` entry |
| 89 | 8+ stale `db/` file entries for relocated repositories and the deleted `db/entities/` directory | cross-cutting-review.md#Finding-9 | Critical | Rewrite "Persistence (`db/`)" section entirely |
| 90 | `db/objs.rs` described as "Database row objects" — it only re-exports types defined elsewhere | cross-cutting-review.md#Finding-18 | Important | Update to: "Re-exports for `ApiKeyUpdate` and backward-compat domain type re-exports" |
| 91 | `users/` domain module absent from "Module Structure" section | cross-cutting-review.md#Finding-21 | Important | Add entry pointing to `users/access_repository.rs` |
| 92 | 2 `test_utils` sub-modules missing from table (`model_fixtures`, `network`) | cross-cutting-review.md#Finding-12 | Nice-to-have | Add entries |

### test_utils PACKAGE.md

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 93 | `PACKAGE.md` references deleted `secret` module; actual module is `objs` and others | services-infra-review.md#Finding-11 | Nice-to-have | `crates/services/src/test_utils/PACKAGE.md` — fix module listing |

---

## Topic: Coverage Gaps

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 94 | `IoError` tests missing `error_type()` and `args()` assertions | errmeta-review.md#Finding-3 | Important | `crates/errmeta/src/test_io_error.rs` |
| 95 | `EntityError` test missing `error_type()` and `args()` assertions | errmeta-review.md#Finding-4 | Nice-to-have | `crates/errmeta/src/test_entity_error.rs` |
| 96 | `test_error_type_from_str` missing 7 of 10 `ErrorType` variants | errmeta-review.md#Finding-13 | Nice-to-have | `crates/errmeta/src/test_error_type.rs` |
| 97 | `impl_error_from!` macro has no in-crate test | errmeta-review.md#Finding-12 | Nice-to-have | Add integration test in `errmeta/` |
| 98 | `EnumMetaHeader` / `parse_enum_meta_header` not unit-tested | errmeta-review.md#Finding-9 | Nice-to-have | `errmeta_derive/src/parse.rs` |
| 99 | No `compile_fail` trybuild test for struct missing `#[error_meta]` | errmeta-review.md#Finding-10 | Nice-to-have | Add `tests/fails/missing_struct_error_meta.rs` |
| 100 | `model_objs.rs` (931 lines) has zero tests | cross-cutting-review.md#Finding-20 | Nice-to-have | Add `test_model_objs.rs` sibling covering merge logic, OAI params |
| 101 | `unsafe` block without `SAFETY` comment + `.unwrap()` panic in `gguf/metadata.rs` | services-batch-c-review.md#Finding-16 | Nice-to-have | `crates/services/src/models/gguf/metadata.rs:17` |
| 102 | `pretty_assertions` unused dev-dependency in `errmeta/Cargo.toml` | errmeta-review.md#Finding-2 | Nice-to-have | Remove from `crates/errmeta/Cargo.toml` `[dev-dependencies]` |

---

## Topic: Minor Code Quality

| # | Issue | File#Finding | Priority | Notes |
|---|-------|--------------|----------|-------|
| 103 | `ServiceUnavailable`/`InvalidAppState` serialization lacks `_error` suffix — historical inconsistency | errmeta-review.md#Finding-8 | Nice-to-have | Document as intentional or standardize |
| 104 | `from_resource_role` uses 50-line verbose impl instead of `.parse::<ResourceRole>().max()` | services-batch-b-review.md#Finding-11 | Nice-to-have | `crates/services/src/auth/auth_objs.rs:87-143` |
| 105 | Hardcoded retry with `tokio::time::sleep` in `KeycloakAuthService::refresh_token` | services-batch-b-review.md#Finding-14 | Nice-to-have | `auth/auth_service.rs:452-471` — add configurable retry delays |
| 106 | Typo in public trait method: `reveiwer_token` instead of `reviewer_token` | services-batch-b-review.md#Finding-4 | Important | `crates/services/src/auth/auth_service.rs:90` trait definition |
| 107 | `access_request_service.rs` declares `pub(crate) mod` to expose mock — should use re-export | services-batch-b-review.md#Finding-10 | Nice-to-have | Add `pub use` for mock in `app_access_requests/mod.rs` |
| 108 | `settings/` inconsistent `pub(crate)` on repository module vs `apps/` module pattern | services-batch-a-review.md#Finding-16 | Nice-to-have | Standardize across modules |
| 109 | MCP instance `name` has no length validation or `NameTooLong` error — `McpServer.name` does | services-batch-d-review.md#Finding-13 | Nice-to-have | Add `MAX_MCP_INSTANCE_NAME_LEN`, `validate_mcp_instance_name()`, `McpError::NameTooLong` |
| 110 | `mcp_client::McpClientError` does not implement `AppError` / use `errmeta` | downstream-crates-review.md#Finding-7 | Nice-to-have | Add `errmeta` + `errmeta_derive` to `mcp_client`; apply `ErrorMeta` derive |
| 111 | `model_fixtures` module in `test_utils/mod.rs` — no re-export, private; intent unclear | services-infra-review.md#Finding-10 | Nice-to-have | Add comment explaining `impl` block visibility pattern |
| 112 | `to_safe_filename` (`pub(crate)`) vs `is_default` (`pub`) inconsistent visibility | services-batch-c-review.md#Finding-6 | Important | Align to both `pub` or both `pub(crate)` in `shared_objs/utils.rs` |

---

## Fix Batching Guide

When feeding to an AI assistant, address fixes in this order. Each batch is independent and safe to apply without the next batch.

### Batch 1: Dead Code Removal (Quick Wins, No Risk)

Safe one-liners and removals:

- Remove stale `println!` calls: `settings/test_setting_service.rs:1070-1079` (4 calls)
- Remove dead variant `KcUuidCollision` from `crates/services/src/app_access_requests/error.rs`
- Remove dead `ObjValidationError` variant + import from `crates/services/src/models/hub_service.rs`
- Remove dead `GGMLQuantizationType` enum from `crates/services/src/models/gguf/constants.rs` or add `#[allow(dead_code)]` + comment
- Remove 12 stale "moved from objs" migration comments: `crates/services/src/lib.rs:7`, `models/model_objs.rs` (11 locations)
- Remove `AliasBuilder` type alias + comment from `crates/services/src/test_utils/objs.rs:12`
- Remove `objs=trace` filter from `crates/services/src/test_utils/logs.rs:11`
- Update `queue_service.rs:43-44` doc comment from `objs::gguf::` to `crate::models::gguf::`
- Fix import: `errmeta_derive/tests/test_args_as_expr.rs:4` → `use crate::objs::ErrorMetas`
- Remove unused `pretty_assertions` dev-dependency from `crates/errmeta/Cargo.toml`
- Fix typo in auth_service trait: `reveiwer_token` → `reviewer_token` in `crates/services/src/auth/auth_service.rs:90`
- Fix `assert_eq!(false, ...)` → `assert!(! ...)` in `test_auth_service.rs:661-662`
- Remove `#[allow(dead_code)]` + `#[allow(unused_imports)]` from `server_app/tests/utils/mod.rs`
- Remove or investigate `#[allow(unused_imports)]` on `alias_response` in `routes_app/src/test_utils/mod.rs:6`

### Batch 2: Logic Bug Fixes (High Priority Correctness)

- Fix `Alias::can_serve` in `crates/services/src/models/model_objs.rs`: replace `api_alias.models.contains(...)` with `api_alias.supports_model(model)`
- Fix `update_toolset` in `crates/services/src/toolsets/toolset_repository.rs`: use `active.update(&self.db)` and return `Ok(ToolsetRow::from(model))`
- Fix `update_api_token` in `crates/services/src/tokens/token_repository.rs`: use `DbError::ItemNotFound{id, item_type}` instead of `sea_orm::DbErr::RecordNotFound`
- Fix SQL portability: `crates/services/src/mcps/mcp_server_repository.rs:100-106` replace `enabled = true/false` with `enabled = 1/0`
- Fix epoch sentinel duplication in `crates/services/src/models/model_objs.rs`: extract `fn epoch_sentinel()` using `DateTime::UNIX_EPOCH`
- Fix `model_metadata_entity.source` field type: change `String` → `AliasSource` in `crates/services/src/models/model_metadata_entity.rs`

### Batch 3: Stale Import Paths (db/ Shim Callers)

These must be done together before removing shims:

- `crates/services/src/tokens/api_token_entity.rs:1` — `use crate::db::TokenStatus` → `use crate::auth::TokenStatus`
- `crates/services/src/models/test_download_repository.rs:2` — `use crate::db::DownloadStatus` → `use crate::models::DownloadStatus`
- `crates/services/src/test_utils/objs.rs:148,169,194,208` — `crate::db::ModelMetadataRow` → `crate::models::ModelMetadataRow`
- `crates/services/src/app_access_requests/access_request_service.rs:8` — remove domain types from `crate::db::` import
- `crates/services/src/mcps/mcp_service.rs:7-10` — import row types from `super::` not `crate::db::`
- `crates/routes_app/src/shared/openapi.rs:83` — `use services::db::DownloadStatus` → `use services::DownloadStatus`
- All 11 files in `auth_middleware` and `routes_app` using `services::db::<DomainType>` → `services::<DomainType>`

### Batch 4: Module Organization (Error.rs, Structural)

- Extract `AppInstanceError` → `crates/services/src/apps/error.rs`
- Create `TokenServiceError` → `crates/services/src/tokens/error.rs`
- Move `SettingsMetadataError` → `crates/services/src/settings/error.rs`
- Move `AiApiServiceError` → `crates/services/src/ai_apis/error.rs` + extract inline tests to `test_ai_api_service.rs`
- Move `ExaError` → `crates/services/src/toolsets/error.rs`
- Move `AppAccessRequestRow` → `crates/services/src/app_access_requests/access_request_objs.rs`
- Move `UserAccessRequestStatus` → `crates/services/src/users/` (new `user_objs.rs`)
- Move `token.rs` content → `crates/services/src/tokens/claims.rs` + `tokens/token_error.rs`; remove `mod token;` from `lib.rs`
- Change `pub mod users` → `mod users` in `crates/services/src/lib.rs:34`
- Add `PartialEq`, `Clone`, `Eq` derives to `ErrorType` in `crates/errmeta/src/error_type.rs`
- Add `Serialize`, `Deserialize` derives to `AppAccessRequestRow` in entity file
- Move `ApiKeyUpdate` → `crates/services/src/models/` (add serde+utoipa derives); delete `db/objs.rs`
- Remove `db/model_repository.rs` backward-compat supertrait
- Move `seed_toolset_configs()` from `db/default_service.rs` to `toolsets/` module

### Batch 5: db/ Shim Elimination

After Batch 3 (stale paths fixed) and Batch 4 (ApiKeyUpdate moved):

- Delete `crates/services/src/db/objs.rs`
- Remove all "backward compatibility" re-export blocks from `crates/services/src/db/mod.rs` (lines 11-35)
- Remove `pub use objs::*` from `crates/services/src/db/mod.rs`
- Verify compilation after each deletion

### Batch 6: Test Convention Alignment

- Replace all `use super::*` in 6 test files with explicit imports (auth objs tests, mcp objs tests, shared_objs tests, test_exa_service)
- Update 3 auth test error assertions from `.to_string()` to `.code()` with correct code format
- Update 4 toolset test error assertions from `matches!()` to `.code()` assertions
- Add `#[anyhow_trace]` + `-> anyhow::Result<()>` to 5 tests in `test_setting_service_db.rs`
- Add `use pretty_assertions::assert_eq` to `test_setting_objs.rs`, `test_setting_service_db.rs`, `test_mcp_objs_types.rs`, `test_mcp_objs_validation.rs`
- Replace `Utc::now()` with `fixed_dt()` in `test_utils/objs.rs:26-27`
- Replace `Utc::now()` with fixed values in `ai_api_service.rs` test functions
- Extract inline tests from `data_service.rs` → `test_data_service.rs`; from `progress_tracking.rs` → `test_progress_tracking.rs`
- Consolidate duplicate `make_server()`/`make_mcp()`/`make_auth_header_row()` helpers in mcps test files
- Replace hardcoded `ENCRYPTION_KEY` in 2 mcps test files with `ctx.service.encryption_key`
- Move `test_toolset_repository` declaration from `toolsets/mod.rs` to `toolset_repository.rs`
- Remove 6 redundant non-rstest test functions in auth objs tests (serialization/deserialization)
- Remove `CrudOperation` single-variant enum + if-else from `test_setting_service.rs`
- Fix `test_access_request_service.rs` if-else branch in test body — split into typed parameterized tests

### Batch 7: Documentation Updates

All after code changes are stable:

- Update root `CLAUDE.md`: fix `TimeService` path, remove `commands` crate, fix dependency diagram, add `users/` to services keywords
- Update `crates/errmeta_derive/CLAUDE.md`: replace `objs` crate references with `services`
- Update `crates/services/CLAUDE.md`: fix `db/entities/` path claim, fix session_service subdirectory claim, add `users/` module documentation
- Update `crates/services/PACKAGE.md`: rewrite `db/` section entirely, fix `db/objs.rs` description, add `users/`, add `model_fixtures`/`network` to test_utils table
- Update `test_utils/PACKAGE.md`: fix module listing (remove `secret`, add actual modules)
- Update 8 downstream CLAUDE.md files: replace all `objs` crate references with `services`/`errmeta` as appropriate
- Rename `test_utils/objs.rs` → `test_utils/fixtures.rs` and update `mod.rs` + PACKAGE.md
