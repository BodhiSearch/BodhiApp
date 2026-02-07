# Error Handling Simplification Plan

**Status**: COMPLETED (2026-02-07)
**Executed across**: 4 Claude Code sessions (context limit reached 3 times)
**Total commits**: 6

## Context

After the recent FTL-to-thiserror migration (20260129), the codebase has ~45 error types with ~150 variants. While the migration was successful, the error system has accumulated structural debt:

1. **7 generic HTTP wrapper structs** in objs (BadRequestError, NotFoundError, etc.) leak HTTP concerns into domain logic. ~96 usages across 14+ files.
2. **6 separate IO error structs** in objs (IoError, IoFileReadError, IoFileWriteError, etc.) that could be one enum.
3. **Dual response paths** - both `ApiError` and `HttpError<E>` convert errors to HTTP. `HttpError<E>` in server_core is unused.
4. **~25 duplicate header extraction patterns** across route handlers manually extracting headers with BadRequestError.
5. **IO operations returning wrong HTTP status codes** - file-not-found returning 500 instead of 404 in HubService and DataService.

**Goal**: Simplify to idiomatic Rust error handling following best practices: domain-specific errors, proper Axum extractors, pre-check patterns for correct HTTP status codes, and elimination of generic HTTP error wrappers.

**JSON format preserved**: `{ "error": { "code": string, "message": string, "type": string, "param": object } }`

---

## Decisions

| Decision | Choice |
|----------|--------|
| Error codes | Internal only - can freely restructure |
| IO errors | Consolidate 6 structs into 1 enum |
| Generic HTTP wrappers | Eliminate entirely - domain types only |
| Service errors | Flat enum per service (keep standalone leaf structs) |
| Response path | Keep `ApiError` only, remove `HttpError<E>` |
| IO status codes | Pre-check pattern (file.exists() before IO) |
| Header extraction | Axum `FromRequestParts` extractors |
| Route errors | Per-domain error enums |
| errmeta_derive macro | Keep as-is |
| Phasing | Local commit per crate, `cargo test` after each |

---

## Commit Log

| Commit | Phase | Message |
|--------|-------|---------|
| `7fed48cc` | 1a | `refactor(objs): consolidate 6 IO error structs into single IoError enum` |
| `541883bb` | 1b | `refactor(server_core): remove unused HttpError<E> wrapper` |
| `2ffabc0a` | 2 | `feat(auth_middleware): add Axum FromRequestParts extractors for header extraction` |
| *(none)* | 3 | No changes needed - pre-check patterns already in place |
| `55be0b9c` | 4 | `refactor(routes_app): replace generic errors with domain enums and Axum extractors` |
| `434e9342` | 5 | `refactor(routes_oai): replace BadRequestError with HttpError::InvalidRequest` |
| `c5769ea2` | 6 | `refactor(objs): remove 7 generic HTTP error wrapper structs` |

---

## Phase 1: IO Error Consolidation in objs - COMPLETED

### 1a: Replace 6 IO structs with 1 IoError enum - COMPLETED

Executed as planned. Consolidated `IoError` (struct), `IoWithPathError`, `IoDirCreateError`, `IoFileReadError`, `IoFileWriteError`, `IoFileDeleteError` into a single `IoError` enum with 6 variants and convenience constructors.

**Files modified**: `crates/objs/src/error/objs.rs`, `crates/services/src/data_service.rs`, `crates/services/src/hub_service.rs`, `crates/services/src/secret_service.rs`, `crates/services/src/setting_service.rs`, `crates/server_app/src/server.rs`, `crates/llama_server_proc/src/error.rs`, `crates/objs/src/gguf/error.rs`

**Deviations**: None.

### 1b: Remove `server_core::HttpError<E>` - COMPLETED

Executed as planned. Deleted `crates/server_core/src/error_response.rs` and removed module/re-export from `lib.rs`.

**Deviations**: None.

---

## Phase 2: Axum Extractors for Header Extraction - COMPLETED

Created `crates/auth_middleware/src/extractors.rs` with `HeaderExtractionError` enum and 7 extractor types: `ExtractToken`, `ExtractUserId`, `ExtractUsername`, `ExtractRole`, `ExtractScope`, `MaybeToken`, `MaybeRole`.

**Deviations**: None.

---

## Phase 3: Service Pre-Check Patterns - COMPLETED (NO CHANGES)

Upon investigation, the existing code already had adequate pre-check patterns:
- `DataService::delete_alias()` already checked existence before deletion
- `HubService::find_local_file()` and `local_file_exists()` already had proper error handling

**Deviations**: No commit produced. The plan anticipated changes but investigation showed the patterns were already correct.

---

## Phase 4: Domain Error Enums for routes_app - COMPLETED

The largest phase. Replaced ~50 generic HTTP error usages across 10 route handler files.

### New domain error enums created

| File | Enum | Variants |
|------|------|----------|
| `routes_access_request.rs` | `AccessRequestError` | `AlreadyPending`, `AlreadyHasAccess`, `PendingRequestNotFound`, `RequestNotFound(i64)`, `InsufficientPrivileges`, `FetchFailed(String)` |
| `routes_users_list.rs` | `UserManagementError` | `ListFailed(String)`, `RoleChangeFailed(String)`, `RemoveFailed(String)` |
| `routes_models_metadata.rs` | `MetadataError` | `InvalidRepoFormat(String)`, `ListAliasesFailed`, `AliasNotFound(ModelAliasResponse)`, `ExtractionFailed(String)`, `EnqueueFailed` |
| `routes_toolsets.rs` | `ToolsetValidationError` | `Validation(String)` |
| `routes_models.rs` | `ModelError` | `MetadataFetchFailed` |
| `routes_user.rs` | `UserInfoError` | `InvalidHeader(String)`, `EmptyToken` |

### Existing error enums extended

| File | Enum | Added Variants |
|------|------|----------------|
| `error.rs` | `LoginError` | `StateDigestMismatch`, `MissingState`, `MissingCode` (removed `BadRequest(#[from] BadRequestError)`) |
| `routes_api_token.rs` | `ApiTokenError` | `PrivilegeEscalation`, `InvalidScope`, `InvalidRole(String)` |
| `routes_setup.rs` | `AppServiceError` | `ServerNameTooShort` |

### Handler signature changes

Replaced manual `HeaderMap` extraction with typed extractors in:
- `routes_access_request.rs` - all handlers use `ExtractToken`
- `routes_users_list.rs` - all 3 handlers use `ExtractToken`
- `routes_toolsets.rs` - all 8 handlers use `ExtractUserId` and/or `ExtractToken` (removed `extract_user_id_from_headers` and `extract_token_from_headers` helper functions)
- `routes_api_token.rs` - all 3 handlers use `ExtractToken`

**Note**: Some handlers still retain `HeaderMap` parameter alongside extractors for `is_oauth_auth()` and `extract_allowed_toolset_scopes()` which read different headers.

### Deviations

1. **`routes_user.rs`**: Plan said "minimal changes, mostly handled by extractors" but we created a full `UserInfoError` enum because `user_info_handler` uses complex optional header patterns (`MaybeToken` wouldn't work since the handler has custom logic for empty-vs-missing tokens).

2. **`routes_models.rs`**: Plan said "check for generic error usage, add domain enum if needed" - we added `ModelError` enum with `MetadataFetchFailed` to replace one `InternalServerError::new` usage.

3. **`routes_models_metadata.rs`**: The `AliasNotFound` variant uses `ModelAliasResponse` (boxed) instead of separate `{ repo, filename, snapshot }` fields as planned, because the existing code already had the response object available.

4. **Test fix**: After removing `KEY_HEADER_BODHIAPP_USER_ID` and `KEY_HEADER_BODHIAPP_TOKEN` from module-level imports in `routes_toolsets.rs`, the test module (using `use super::*;`) lost access to these constants. Fixed by adding explicit `use auth_middleware::{KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USER_ID};` to the test module.

---

## Phase 5: Migrate routes_oai - COMPLETED

Added `InvalidRequest(String)` variant to the existing `HttpError` enum in `routes_chat.rs`. Changed `validate_chat_completion_request` return type from `Result<(), BadRequestError>` to `Result<(), HttpError>`. Replaced 3 `BadRequestError::new(...)` calls.

**Deviations**: None.

---

## Phase 6: Remove Generic HTTP Error Structs from objs - COMPLETED

Removed all 7 structs from `crates/objs/src/error/objs.rs`:
1. `BadRequestError`
2. `NotFoundError`
3. `InternalServerError`
4. `UnauthorizedError`
5. `ConflictError`
6. `UnprocessableEntityError`
7. `ServiceUnavailableError`

Re-exports were handled automatically via `pub use objs::*;` in `error/mod.rs` - no changes needed to `mod.rs` or `lib.rs`.

Updated `error_api.rs` test to use a test-only `TestError` enum with `BadInput` and `Internal` variants.

### Deviations

1. **Re-exports**: Plan said to update `crates/objs/src/error/mod.rs` and `crates/objs/src/lib.rs`, but these use wildcard re-exports (`pub use objs::*;`) so no changes were needed.

2. **Test fix**: The `errmeta_derive` macro auto-generates `param` fields from unnamed enum variant fields using `var_0`, `var_1` etc. Initial test expectations omitted the `param` field, causing 2 test failures. Fixed by adding `"param": {"var_0": "..."}` to expected JSON.

---

## Phase 7: Final Verification - COMPLETED

- `cargo test` - All tests pass across entire workspace (0 failures)
- `cargo test -p bodhi --features native` - All 7 tests pass
- `cargo check` - Clean compilation, no errors
- `make build.ui-rebuild` and `make test.ui` - Not executed (no UI changes were made; error code/message changes are internal API and don't affect UI components)

---

## Critical Files Summary

| Phase | Files Modified |
|-------|---------------|
| 1a | `crates/objs/src/error/objs.rs`, `crates/services/src/data_service.rs`, `crates/services/src/hub_service.rs`, `crates/services/src/secret_service.rs`, `crates/services/src/setting_service.rs`, `crates/server_app/src/server.rs`, `crates/llama_server_proc/src/error.rs`, `crates/objs/src/gguf/error.rs` |
| 1b | `crates/server_core/src/error_response.rs` (deleted), `crates/server_core/src/lib.rs` |
| 2 | `crates/auth_middleware/src/extractors.rs` (new), `crates/auth_middleware/src/lib.rs` |
| 3 | *(no files modified)* |
| 4 | `crates/routes_app/src/error.rs`, `routes_access_request.rs`, `routes_users_list.rs`, `routes_models_metadata.rs`, `routes_user.rs`, `routes_toolsets.rs`, `routes_api_token.rs`, `routes_login.rs`, `routes_setup.rs`, `routes_models.rs` |
| 5 | `crates/routes_oai/src/routes_chat.rs` |
| 6 | `crates/objs/src/error/objs.rs`, `crates/objs/src/error/error_api.rs` |

---

## What Did NOT Change (as planned)

- **AppError trait** - stays as-is (error_type, code, args, status)
- **errmeta_derive macro** - stays as-is
- **ApiError struct** - stays as single response path
- **OpenAIApiError / ErrorBody** - stays as-is (JSON format preserved)
- **ErrorType enum** - stays with all 12 variants
- **Standalone leaf error structs** (AliasNotFoundError, HubFileNotFoundError, etc.) - kept for testing and cross-service reuse
- **Service error enums** (HubServiceError, DataServiceError, etc.) - kept as flat enums per service, simplified where IO variants consolidated
- **impl_error_from! macro** - kept where needed for external error conversion

---

## Lessons Learned

1. **`use super::*;` fragility**: When route handler modules import header constants at the module level and tests use `use super::*;`, removing those imports from the handler (because extractors replaced manual extraction) breaks tests. Tests need explicit imports added.

2. **errmeta_derive auto-generates `param` fields**: Positional enum variant fields (`{0}`) generate `var_0` keys in the args HashMap. Named struct fields generate their field names. This matters for test assertions.

3. **Phase 3 was a no-op**: The pre-check patterns anticipated by the plan were already in place. Investigation before coding saved unnecessary changes.

4. **Handlers retaining HeaderMap**: Even after adding typed extractors, some handlers still need `HeaderMap` for auxiliary functions like `is_oauth_auth()` and `extract_allowed_toolset_scopes()`. This is expected and not a problem - the extractors eliminated the boilerplate for the most common patterns.

5. **Wildcard re-exports simplify removal**: `pub use module::*;` means removing items from the source module automatically removes them from the public API without touching re-export files.
