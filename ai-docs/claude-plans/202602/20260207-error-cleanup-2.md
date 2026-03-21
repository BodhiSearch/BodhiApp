# Error Handling Architecture: Holistic Simplification & Uniformity

## Status: COMPLETE

All 6 phases implemented and verified. `make test.backend` passes with 0 failures.

---

## Context

The error handling system across BodhiApp's Rust workspace had accumulated structural debt across multiple dimensions:
- **ErrorType duplication**: `Validation`+`BadRequest` both map to 400, `Authentication`+`Unauthorized` both map to 401
- **Wrong status codes**: 4 error types mapped to incorrect HTTP status codes
- **Dead code**: 3 localization error types, 2 dead enum variants never constructed
- **Non-uniform messages**: SecretServiceError uses snake_case codes ("key_mismatch"), SettingServiceError same ("lock_error"), KeyringError same ("keyring_error") — while most other errors use proper sentences
- **Redundant type pairs**: SerdeJsonError + SerdeJsonWithPathError, SerdeYamlError + SerdeYamlWithPathError could each be a single type with optional path
- **Small isolated enums**: LogoutError (1 variant), ModelError (1), ToolsetValidationError (1) are unnecessarily granular
- **String field antipattern**: UserManagementError, AccessRequestError use `.to_string()` at boundaries, losing structured error info where transparent forwarding would be better

This refactoring establishes a **uniform error pattern** across all crates, fixes correctness bugs, removes dead code, and consolidates error types — while preserving the OpenAI JSON response format `{ "error": { "code", "message", "type", "param" } }` and the core infrastructure (AppError trait, errmeta_derive, ApiError).

No backwards compatibility requirements for error codes, messages, or types.

---

## Uniform Error Pattern (applied everywhere)

### 1. Message format
All `#[error("...")]` messages: **Subject-first, sentence case, ends with period.**
- Good: `"Model configuration '{alias}' not found."`
- Good: `"Failed to read file '{path}': {source}."`
- Bad: `"key_mismatch"`, `"lock_error"`, `"keyring_error"`

### 2. Field policy
Every captured field MUST appear in the error message. If a field isn't shown to the user, don't capture it.

### 3. Source handling (three valid patterns)
- **Pattern A**: `#[source]` or `#[from]` with `{source}` in message — source displayed. Used by IoError, SerdeError, SqlxError. **Keep as-is.**
- **Pattern B**: `#[error(transparent)]` with `#[from]` — delegates Display to inner error. Used by service error union types. **Keep as-is.**
- **Pattern C**: `#[from]` with `args_delegate=false` and opaque message like `"keyring_error"` — source captured but hidden. **Fix: either display the source or use a proper human-readable message.**

### 4. Service error enums
Transparent pass-through union types. Don't add messages at the service layer — let leaf errors carry context.

### 5. Route error enums
Group by domain. Merge 1-3 variant enums into their domain group. Keep 4+ variant enums separate.

### 6. No dead code
Remove any variant/type never constructed in non-test code.

### 7. Serde errors
Unify pairs into single types with `path: Option<String>`.

---

## Implementation Insights

### errmeta_derive limitation with `Option<String>` fields

**Problem:** The `errmeta_derive::ErrorMeta` proc macro generates `format!("{}", self.field)` for ALL named struct fields to produce the `args()` HashMap. `Option<String>` does not implement `Display`, so the derive macro fails to compile.

**Symptom:** Compile error like:
```
error[E0277]: `Option<String>` doesn't implement `std::fmt::Display`
```

**Solution:** For any error struct with `Option<T>` fields, you CANNOT use `#[derive(errmeta_derive::ErrorMeta)]`. Instead, manually implement `Display`, `Error`, and `AppError`:

```rust
#[derive(Debug)]
pub struct SerdeJsonError {
  source: serde_json::Error,
  path: Option<String>,
}

// Manual Display with conditional path
impl std::fmt::Display for SerdeJsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.path {
      Some(path) => write!(f, "Failed to process JSON file '{}': {}.", path, self.source),
      None => write!(f, "Failed to process JSON data: {}.", self.source),
    }
  }
}

// Manual Error with source chain
impl std::error::Error for SerdeJsonError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.source)
  }
}

// Manual AppError (what errmeta_derive would generate)
impl AppError for SerdeJsonError {
  fn error_type(&self) -> String { ErrorType::InternalServer.to_string() }
  fn code(&self) -> String { "serde_json_error".to_string() }
  fn args(&self) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("source".to_string(), self.source.to_string());
    if let Some(path) = &self.path {
      map.insert("path".to_string(), path.clone());
    }
    map
  }
}
```

Also add convenience constructors and manual `From` impl:
```rust
impl SerdeJsonError {
  pub fn new(source: serde_json::Error) -> Self { Self { source, path: None } }
  pub fn with_path(source: serde_json::Error, path: impl Into<String>) -> Self {
    Self { source, path: Some(path.into()) }
  }
}

impl From<serde_json::Error> for SerdeJsonError {
  fn from(source: serde_json::Error) -> Self { Self::new(source) }
}
```

### Dead code verification is critical

**Problem:** The original plan claimed `ContextError::Unreachable` and `ContextError::ExecNotExists` were "never constructed in non-test code." This was WRONG — both are used in `shared_rw.rs` (production code).

**Root cause:** The dead-code search only checked test files, not all source files.

**Lesson:** Before removing any variant, search ALL `.rs` files in the crate (and dependents) for the variant name. Use `Grep` across the entire `crates/` directory, not just test directories.

### Renaming error enums changes auto-generated error codes

When `UserInfoError` was renamed to `UserRouteError`, the auto-generated error code changed from `user_info_error-empty_token` to `user_route_error-empty_token`. This broke a test assertion.

**Rule:** After renaming any error enum, search all tests for the old snake_case error code pattern (e.g. `user_info_error`) and update assertions.

### AuthServiceApiError restructuring ripple effects

Converting `AuthServiceApiError(String)` to `AuthServiceApiError { status: u16, body: String }` required updates in:
1. The enum definition and `#[error("...")]` message template
2. All construction sites (using `{ status: code, body: msg }` syntax)
3. All `From<KeycloakError>` implementations (using `status: 0` as sentinel)
4. All test pattern matches: `AuthServiceApiError(_)` → `AuthServiceApiError { .. }` and `AuthServiceApiError(msg) if msg == "..."` → `AuthServiceApiError { body, .. } if body == "..."`
5. Test assertions checking the JSON `"param"` field (now has `"status"` and `"body"` keys instead of `"var_0"`)

---

## Phase 1: `objs` — Foundation cleanup ✅

**Status: COMPLETE** — All 377 objs tests pass.

**Files changed:**
- `crates/objs/src/error/common.rs`
- `crates/objs/src/error/objs.rs`
- `crates/objs/src/error/l10n.rs` → **DELETED**
- `crates/objs/src/error/mod.rs`

**Changes made:**

1. **`common.rs` — Removed duplicate ErrorType variants:**
   - Removed `Validation` variant (never used in `#[error_meta]` annotations; `BadRequest` covers 400)
   - Removed `Unauthorized` variant (only 1 use in exa_service, switched to `Authentication` in Phase 2)
   - Removed their `status()` match arms and strum serialize attributes

2. **`objs.rs` — Fixed status codes:**
   - `RwLockReadError`: `ErrorType::BadRequest` → `ErrorType::InternalServer` (concurrency bug = 500)
   - `ObjValidationError::FilePatternMismatch`: `ErrorType::InternalServer` → `ErrorType::BadRequest` (bad input = 400)

3. **`objs.rs` — Unified Serde error pairs (see Implementation Insights above):**
   - Replaced `SerdeJsonError` + `SerdeJsonWithPathError` with single `SerdeJsonError { source, path: Option<String> }`
   - Replaced `SerdeYamlError` + `SerdeYamlWithPathError` with single `SerdeYamlError { source, path: Option<String> }`
   - Both use manual `Display`, `Error`, `AppError` impls (cannot use `errmeta_derive` with `Option<String>`)
   - Added `new()` and `with_path()` constructors, manual `From` impls
   - Updated `impl_error_from!` macros

4. **Deleted `l10n.rs`** — Removed dead types: `LocalizationSetupError`, `LocalizationMessageError`, `LocaleNotSupportedError`. Removed `mod l10n` and `pub use l10n::*` from `mod.rs`.

---

## Phase 2: `services` — Status codes, dead variants, message standardization, field cleanup ✅

**Status: COMPLETE** — All 283 services tests pass.

**Files changed:**
- `crates/services/src/data_service.rs`
- `crates/services/src/secret_service.rs`
- `crates/services/src/setting_service.rs`
- `crates/services/src/keyring_service.rs`
- `crates/services/src/exa_service.rs`
- `crates/services/src/auth_service.rs`
- `crates/services/src/db/encryption.rs`
- `crates/services/src/session_service.rs`
- `crates/routes_app/src/routes_login.rs` (AuthServiceApiError construction in tests)
- `crates/routes_app/src/routes_users_list.rs` (AuthServiceApiError construction in tests)

**Changes made:**

1. **`data_service.rs`:**
   - `DataFileNotFoundError`: `ErrorType::BadRequest` → `ErrorType::NotFound` (file not found = 404)
   - Removed dead `DataServiceError::DirMissing` variant
   - Collapsed `SerdeYamlErrorWithPath` + `SerdeYamlError` variants into single `SerdeYaml(#[from] SerdeYamlError)`
   - Updated construction sites: `SerdeYamlWithPathError::new(err, path)` → `SerdeYamlError::with_path(err, path)`

2. **`exa_service.rs`:** `ExaError::InvalidApiKey`: `ErrorType::Unauthorized` → `ErrorType::Authentication`

3. **`secret_service.rs` — Standardized 5 messages:**
   - `KeyMismatch`: `"key_mismatch"` → `"Secret key mismatch."`
   - `KeyNotFound`: `"key_not_found"` → `"Secret key not found."`
   - `EncryptionError(String)`: `"encryption_error"` → `"Encryption failed: {0}."`
   - `DecryptionError(String)`: `"decryption_error"` → `"Decryption failed: {0}."`
   - `InvalidFormat(String)`: `"invalid_format"` → `"Invalid secret format: {0}."`

4. **`setting_service.rs` — Standardized 2 messages:**
   - `LockError(String)`: `"lock_error"` → `"Settings lock failed: {0}."`
   - `InvalidSource`: `"invalid_source"` → `"Invalid settings source."`

5. **`keyring_service.rs` — Standardized 2 messages (kept `args_delegate=false`):**
   - `KeyringError`: `"keyring_error"` → `"Keyring access failed: {0}."`
   - `DecodeError`: `"decode_error"` → `"Keyring data decode failed: {0}."`

6. **`auth_service.rs` — Restructured AuthServiceApiError:**
   - `AuthServiceApiError(String)` → `AuthServiceApiError { status: u16, body: String }`
   - Message: `"Authentication service API error (status {status}): {body}."`
   - Updated `From<KeycloakError>` impl, all construction sites, all test matches

7. **`session_service.rs`:** Added trailing period: `"Session store error: {0}"` → `"Session store error: {0}."`

8. **`db/encryption.rs` — Standardized 3 messages:**
   - `EncryptionFailed`: `"encryption_failed"` → `"Encryption failed."`
   - `DecryptionFailed`: `"decryption_failed"` → `"Decryption failed."`
   - `InvalidFormat(String)`: `"invalid_format"` → `"Invalid encryption format: {0}."`

---

## Phase 3: `llama_server_proc` — Status code fix ✅

**Status: COMPLETE** — All 5 llama_server_proc tests pass.

**Files changed:**
- `crates/llama_server_proc/src/error.rs`

**Changes made:**
- `ServerNotReady`: `ErrorType::InternalServer` (500) → `ErrorType::ServiceUnavailable` (503)
- Updated test assertions for error_type and status_code

---

## Phase 4: `server_core` — No changes (plan correction) ✅

**Status: COMPLETE** — No changes needed.

**Plan correction:** The original plan claimed `ContextError::Unreachable` and `ContextError::ExecNotExists` were dead code. **This was wrong** — both are actively used in `crates/server_core/src/shared_rw.rs` (lines 155, 201, 227, 248, 269). The variants and their tests were preserved unchanged.

The `SerdeJson` variant already used the unified `SerdeJsonError` type (the `From` impl remained compatible), so no Serde changes were needed either.

---

## Phase 5: `routes_oai` — Message standardization ✅

**Status: COMPLETE** — All routes_oai tests pass.

**Files changed:**
- `crates/routes_oai/src/routes_chat.rs`

**Changes made — Standardized 3 `HttpError::InvalidRequest` messages:**
- `"'model' field is required and must be a string"` → `"Field 'model' is required and must be a string."`
- `"'messages' field is required and must be an array"` → `"Field 'messages' is required and must be an array."`
- `"'stream' field must be a boolean"` → `"Field 'stream' must be a boolean."`

No test assertions referenced these exact strings, so no test updates needed.

---

## Phase 6: `routes_app` — Consolidate small enums, remove dead variants ✅

**Status: COMPLETE** — All 217 routes_app tests pass (2 ignored).

**Files changed:**
- `crates/routes_app/src/error.rs` (LoginError — added SessionDelete variant)
- `crates/routes_app/src/routes_login.rs` (removed LogoutError enum, updated logout_handler, updated 2 test assertions for AuthServiceApiError)
- `crates/routes_app/src/routes_user.rs` (renamed UserInfoError → UserRouteError, added 3 management variants, updated 1 test assertion for error code)
- `crates/routes_app/src/routes_users_list.rs` (removed UserManagementError enum, imported UserRouteError)
- `crates/routes_app/src/routes_create.rs` (removed dead AliasNotPresent variant)

**Changes made:**

1. **Merged LogoutError (1 variant) into LoginError:**
   - Added `SessionDelete(tower_sessions::session::Error)` to LoginError with `args_delegate=false`, `error_type=ErrorType::InternalServer`, code `"login_error-session_delete"`
   - Updated `logout_handler`: `LogoutError::from` → `LoginError::SessionDelete`
   - Removed `LogoutError` enum from `routes_login.rs`

2. **Merged UserInfoError (2) + UserManagementError (3) → UserRouteError (5):**
   - Created `UserRouteError` in `routes_user.rs` with all 5 variants
   - Updated all references in `routes_user.rs` and `routes_users_list.rs`
   - Added `use crate::UserRouteError` import to `routes_users_list.rs`
   - Removed `UserManagementError` enum from `routes_users_list.rs`

3. **Removed dead `CreateAliasError::AliasNotPresent`** — never constructed

4. **Fixed 3 test assertions:**
   - `routes_login.rs`: 2 tests updated for new `AuthServiceApiError { status, body }` JSON format
   - `routes_user.rs`: 1 test updated for `user_info_error-empty_token` → `user_route_error-empty_token`

**Did NOT change:** ModelError, ToolsetValidationError, DevError, AppServiceError, SettingsError, AccessRequestError, ApiTokenError, MetadataError, PullError — all either substantial enough or in isolated domains.

---

## Phases with no changes (verified)

- **auth_middleware**: AuthError (18 variants) correctly typed. No dead variants, no consolidation needed.
- **commands**: PullCommandError, CreateCommandError use all-transparent delegation. Correct.
- **routes_all**: No error types.
- **server_app**: TaskJoinError, ServerError, ServeError properly structured.
- **bodhi/src-tauri**: AppSetupError, NativeError properly structured.

---

## impl_error_from! evaluation

All ~20 usages are **required** due to Rust's orphan rule. Every usage converts an external crate's error through an intermediate wrapper in a different crate. No usages can be eliminated. Keep all as-is.

---

## Summary of all changes

| Category | Count | Details |
|----------|-------|---------|
| ErrorType variants removed | 2 | `Validation`, `Unauthorized` |
| Dead error types removed | 3 types | `LocalizationSetupError`, `LocalizationMessageError`, `LocaleNotSupportedError` |
| Dead enum variants removed | 2 | `DataServiceError::DirMissing`, `CreateAliasError::AliasNotPresent` |
| Status code fixes | 4 | RwLockRead→500, DataFileNotFound→404, ServerNotReady→503, FilePatternMismatch→400 |
| Serde type unifications | 2 | SerdeJson pair→1, SerdeYaml pair→1 |
| Message standardizations | ~15 | SecretService(5), SettingService(2), KeyringService(2), Encryption(3), SessionService(1), HttpError(3) |
| Field restructuring | 1 | `AuthServiceApiError` String→structured `{status, body}` |
| Error enum consolidations | 2 | LogoutError→LoginError, UserInfoError+UserManagementError→UserRouteError |
| Plan corrections | 1 | ContextError::Unreachable and ExecNotExists are NOT dead — kept |

## Verification

```bash
make test.backend  # All tests pass, 0 failures
```
