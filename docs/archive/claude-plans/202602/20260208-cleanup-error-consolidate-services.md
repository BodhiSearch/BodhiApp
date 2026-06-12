# Error Organization & Route Consolidation Plan

## Context

After completing the error cleanup (20260207) and route reorganization (20260208), the `routes_app` crate has domain-specific error enums but they live in inconsistent locations: some in `types.rs` mixed with DTOs, some in a shared `error.rs`, and some inline in handler files. This plan establishes a uniform pattern:

- **Folder modules**: errors live in `error.rs`, DTOs stay in `types.rs`
- **Standalone files**: errors stay inline (already the case)
- **LoginError** moves from shared/ to its logical home in routes_auth/
- **routes_app_request_access.rs** moves into routes_auth/ (it's an auth concern)
- **HttpError** in routes_oai renamed to **OAIRouteError** for domain clarity

**Delivery**: One PR with 6 phased commits. Each commit independently compiles and passes tests.

---

## Commit Sequence

### Commit 1: Extract errors from `routes_users/types.rs` into `routes_users/error.rs` ✅ DONE (36f109e3)

**Create:**
- `crates/routes_app/src/routes_users/error.rs` — `UserRouteError` (5 variants) + `AccessRequestError` (6 variants)

**Modify:**
- `crates/routes_app/src/routes_users/types.rs` — remove both error enums + their imports (`AppError`, `ErrorType`), keep all DTOs
- `crates/routes_app/src/routes_users/mod.rs` — add `mod error; pub use error::*;`

**Import chain**: Handler files (`user_info.rs`, `management.rs`, `access_request.rs`) import errors via `crate::UserRouteError` / `crate::AccessRequestError` — still resolves through `routes_users/mod.rs → lib.rs` re-export chain.

**Verify:** `cargo check -p routes_app && cargo test -p routes_app`

---

### Commit 2: Extract errors from `routes_models/types.rs` into `routes_models/error.rs` ✅ DONE (f33c7a7e)

**Create:**
- `crates/routes_app/src/routes_models/error.rs` — `ModelError` (1), `CreateAliasError` (3), `PullError` (3), `MetadataError` (5)

**Modify:**
- `crates/routes_app/src/routes_models/types.rs` — remove all 4 error enums + their imports (`commands::CreateCommandError`, `AppError`, `ErrorType`, `ObjValidationError`, `services::AliasNotFoundError`), keep all DTOs
- `crates/routes_app/src/routes_models/mod.rs` — add `mod error; pub use error::*;`

**Verify:** `cargo check -p routes_app && cargo test -p routes_app`

---

### Commit 3: Extract error from `routes_toolsets/types.rs` into `routes_toolsets/error.rs` ✅ DONE (b9cf230a)

**Create:**
- `crates/routes_app/src/routes_toolsets/error.rs` — `ToolsetValidationError` (1 variant)

**Modify:**
- `crates/routes_app/src/routes_toolsets/types.rs` — remove error enum + `AppError`/`ErrorType` imports, keep all DTOs
- `crates/routes_app/src/routes_toolsets/mod.rs` — add `mod error; pub use error::*;`

**Verify:** `cargo check -p routes_app && cargo test -p routes_app`

---

### Commit 4: Move `LoginError` from `shared/error.rs` to `routes_auth/error.rs` ✅ DONE (d8e6739b)

**Create:**
- `crates/routes_app/src/routes_auth/error.rs` — `LoginError` (13 variants), exact content from `shared/error.rs`

**Delete:**
- `crates/routes_app/src/shared/error.rs`

**Modify:**
- `crates/routes_app/src/shared/mod.rs` — remove `mod error;` and `pub use error::*;` (remaining: common, openapi, pagination, utils)
- `crates/routes_app/src/routes_auth/mod.rs` — add `mod error; pub use error::*;`

**Re-export chain change:**
- Old: `shared/error.rs` → `shared/mod.rs (pub use error::*)` → `lib.rs (pub use shared::*)`
- New: `routes_auth/error.rs` → `routes_auth/mod.rs (pub use error::*)` → `lib.rs (pub use routes_auth::*)`
- `crate::LoginError` continues to resolve. No consumer changes needed.

**Risk**: `openapi.rs` and other files import `LoginError` via `crate::` — verified they don't use `super::` paths.

**Verify:** `cargo check -p routes_app && cargo test -p routes_app && cargo check -p routes_all`

---

### Commit 5: Move `routes_app_request_access.rs` into `routes_auth/` ✅ DONE (cbad3285)

Source file: 611 lines total. Handler code: lines 1-156. Tests: lines 157-611.

**Create:**
- `crates/routes_app/src/routes_auth/request_access.rs` — handler code (lines 1-156)
- `crates/routes_app/src/routes_auth/tests/request_access_test.rs` — test code (extracted from `#[cfg(test)] mod tests`)

**Delete:**
- `crates/routes_app/src/routes_app_request_access.rs`

**Modify:**
- `crates/routes_app/src/lib.rs`:
  - Remove `mod routes_app_request_access;` (line 21)
  - Remove `pub use routes_app_request_access::*;` (line 43)
- `crates/routes_app/src/routes_auth/mod.rs` — add `mod request_access; pub use request_access::*;`
- `crates/routes_app/src/routes_auth/tests/mod.rs` — add `mod request_access_test;`

**Import safety**: File already uses `crate::{LoginError, ENDPOINT_APPS_REQUEST_ACCESS}` — no `super::` usage. Tests use `super::*` which will need updating to explicit `crate::` imports.

**Critical re-export**: `request_access_handler` and `__path_request_access_handler` must remain accessible at crate root for:
- `shared/openapi.rs` — imports `__path_request_access_handler` via `use crate::*`
- `routes_all/src/routes.rs` — imports `request_access_handler` via `use routes_app::*`

Both work through the re-export chain: `routes_auth/mod.rs (pub use request_access::*)` → `lib.rs (pub use routes_auth::*)`.

**Verify:** `cargo check -p routes_app && cargo test -p routes_app && cargo check -p routes_all && cargo test -p routes_all`

---

### Commit 6: Rename `HttpError` → `OAIRouteError` in `routes_oai` ✅ DONE (ec7bed73)

**Modify:**
- `crates/routes_oai/src/routes_chat.rs` — rename enum and all 8 references:
  - `pub enum HttpError` → `pub enum OAIRouteError`
  - `HttpError::InvalidRequest(...)` (3 occurrences)
  - `HttpError::Http` (2 occurrences)
  - `HttpError::Serialization` (1 occurrence)
  - Function signature: `Result<(), HttpError>` → `Result<(), OAIRouteError>`

**Error code changes** (auto-generated, no test assertions found for `http_error-`):
- `http_error-http` → `oai_route_error-http`
- `http_error-serialization` → `oai_route_error-serialization`
- `http_error-invalid_request` → `oai_route_error-invalid_request`

**Consumer safety**: `HttpError` only referenced in `routes_oai/src/routes_chat.rs`. Not imported by name in any other crate. No test assertions on error codes.

**Verify:** `cargo check -p routes_oai && cargo test -p routes_oai`

---

## Post-Implementation Verification

```
make test.backend
```

Runs `cargo test` (all crates) + `cargo test -p bodhi --features native`. All tests should pass — no behavior changes, only structural movement.

---

## Files Summary

| Commit | Created | Deleted | Modified |
|--------|---------|---------|----------|
| 1 | `routes_users/error.rs` | — | `routes_users/types.rs`, `routes_users/mod.rs` |
| 2 | `routes_models/error.rs` | — | `routes_models/types.rs`, `routes_models/mod.rs` |
| 3 | `routes_toolsets/error.rs` | — | `routes_toolsets/types.rs`, `routes_toolsets/mod.rs` |
| 4 | `routes_auth/error.rs` | `shared/error.rs` | `shared/mod.rs`, `routes_auth/mod.rs` |
| 5 | `routes_auth/request_access.rs`, `routes_auth/tests/request_access_test.rs` | `routes_app_request_access.rs` | `lib.rs`, `routes_auth/mod.rs`, `routes_auth/tests/mod.rs` |
| 6 | — | — | `routes_oai/src/routes_chat.rs` |

**Total**: 6 files created, 2 files deleted, ~12 file modifications across 6 commits.

---

## What Does NOT Change

- Standalone files keep inline errors: `routes_setup.rs` (AppServiceError), `routes_settings.rs` (SettingsError), `routes_api_token.rs` (ApiTokenError), `routes_dev.rs` (DevError)
- `routes_api_models/` — no errors to move (DTOs only)
- `routes_auth/types.rs` — no errors to move (DTOs only)
- `OllamaError` in routes_oai — stays as-is (contract requirement, not an AppError)
- All error enums keep their names and variants (except HttpError → OAIRouteError)
- No behavior changes, no new error types, no variant additions/removals

---

## Implementation Status

**All 6 commits completed.** Pending `make test.backend` full verification by user.

### Deviations from Plan

None. All commits executed exactly as planned:
- Commits 1-3: Clean error extraction from `types.rs` to `error.rs` for each folder module
- Commit 4: Git detected the move as a rename (`shared/error.rs` → `routes_auth/error.rs`) since content was identical
- Commit 5: Tests converted from `super::*` to explicit `crate::` imports as planned
- Commit 6: `replace_all` edit on `HttpError` → `OAIRouteError` covered all 8 references in one operation

### Per-Commit Test Results

| Commit | Test Command | Result |
|--------|-------------|--------|
| 1 | `cargo check -p routes_app` | pass |
| 2 | `cargo test -p routes_app` | 217 pass, 2 ignored |
| 3 | `cargo test -p routes_app` | 217 pass, 2 ignored |
| 4 | `cargo test -p routes_app` + `cargo check -p routes_all` | 217 pass + routes_all compiles |
| 5 | `cargo test -p routes_app` + `cargo test -p routes_all` | 217 pass + 6 pass |
| 6 | `cargo test -p routes_oai` | 12 pass |

### Discoveries

- No unexpected import chain issues — all `crate::` paths resolved correctly through the re-export chains
- `routes_models/types.rs` had section comments (`// === From routes_*.rs ===`) that were naturally cleaned up when errors were extracted, leaving a cleaner DTO-only file
- `routes_toolsets/types.rs` had section header comments (`// Error Types`) that were removed along with the error enum
