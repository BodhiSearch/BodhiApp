# Plan: Remove ErrorMessage and Unify Bootstrap Error Handling

## Context

`ErrorMessage` is a legacy flat struct (`code`, `type`, `message`) in `objs::error::common` from the old i18n/fluent era. It serves as the top-level bootstrap error type in `lib_bodhiserver` and `bodhi/src-tauri`, using `ApiError` as an intermediate conversion bus during service setup. This creates inconsistency: runtime errors use proper domain enums with thiserror/errmeta_derive, but bootstrap errors funnel through `ErrorMessage` and misuse `ApiError` (an HTTP-response type) for non-HTTP contexts.

**Goal**: Remove `ErrorMessage`, consolidate all bootstrap errors into a single `BootstrapError` enum in `lib_bodhiserver`, remove `ApiError` from the setup path, and extend `AppSetupError` in `bodhi/src-tauri` to properly wrap upstream errors.

---

## Execution Model

Each phase follows a strict workflow, implemented by a dedicated sub-agent:

1. **Code changes** -- edit files for the target crate
2. **Compile check** -- `cargo check -p <crate>` to verify compilation
3. **Test compile** -- `cargo test -p <crate> --no-run` to verify test compilation
4. **Test run + fix** -- `cargo test -p <crate>`, fix failures, add/update tests
5. **All tests pass** -- confirm green
6. **Local commit** -- commit changes for this phase

---

## Phase 1: lib_bodhiserver — Sub-agent 1

**Crate**: `crates/lib_bodhiserver`
**Verify**: `cargo check -p lib_bodhiserver` -> `cargo test -p lib_bodhiserver`
**Commit**: `refactor: consolidate lib_bodhiserver error enums into BootstrapError`

### 1.1: Merge error enums into `BootstrapError`

**File**: `crates/lib_bodhiserver/src/error.rs`

Remove `AppOptionsError` and `AppServiceBuilderError` enums entirely. Merge their variants into `BootstrapError` and add new transparent variants for service errors:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum BootstrapError {
  // --- existing BootstrapError variants ---
  #[error("failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotResolved,

  #[error("io_error: failed to create directory {path}, error: {source}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate {
    #[source]
    source: io::Error,
    path: String,
  },

  #[error("BODHI_HOME value must be set")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotSet,

  // --- absorbed from AppOptionsError ---
  #[error("validation_error: required property '{0}' is not set")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ValidationError(String),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, args_delegate = false)]
  Parse(#[from] strum::ParseError),

  #[error("unknown_system_setting: {0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UnknownSystemSetting(String),

  // --- absorbed from AppServiceBuilderError ---
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ServiceAlreadySet(String),

  #[error("Encryption key not properly configured.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PlaceholderValue(String),

  // --- new: replace .expect() panic ---
  #[error("AppServiceBuilder::build() called without BootstrapParts.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MissingBootstrapParts,

  // --- new: transparent service error variants ---
  #[error(transparent)]
  Db(#[from] DbError),

  #[error(transparent)]
  SecretService(#[from] SecretServiceError),

  #[error(transparent)]
  SessionService(#[from] SessionServiceError),

  #[error(transparent)]
  Keyring(#[from] KeyringError),

  #[error(transparent)]
  Io(#[from] IoError),
}
```

**Also remove from this file**:
- All three `From<_> for ErrorMessage` impls
- `use objs::ErrorMessage` import

### 1.2: Update `AppOptionsBuilder`

**File**: `crates/lib_bodhiserver/src/app_options.rs`

- `build()` return type: `Result<AppOptions, AppOptionsError>` -> `Result<AppOptions, BootstrapError>`
- `set_system_setting()` return type: similarly
- Replace `AppOptionsError::ValidationError(...)` -> `BootstrapError::ValidationError(...)`
- Replace `AppOptionsError::UnknownSystemSetting(...)` -> `BootstrapError::UnknownSystemSetting(...)`
- `#[from] strum::ParseError` auto-converts via `BootstrapError::Parse`

### 1.3: Update `AppServiceBuilder` and helpers

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

**`build()` method**:
- Return type: `Result<DefaultAppService, ErrorMessage>` -> `Result<DefaultAppService, BootstrapError>`
- Replace `.expect("build() requires BootstrapParts")` -> `.ok_or(BootstrapError::MissingBootstrapParts)?`

**Helper methods — change from `Result<_, ApiError>` to native errors**:
- `build_db_service()` -> `Result<_, BootstrapError>` (DbError from pool + migrate, both `#[from]`)
- `build_session_service()` -> `Result<_, BootstrapError>` (DbError + SessionServiceError, both `#[from]`)
- `build_hub_service()` -> `Result<_, BootstrapError>` (map `std::io::Error` to `IoError` via `.map_err(IoError::from)?`)
- `get_or_build_secret_service()` -> `Result<_, SecretServiceError>` (single source; `?` in `build()` converts to BootstrapError)
- `build_encryption_key()` -> `Result<_, BootstrapError>` (PlaceholderValue directly + KeyringError via `#[from]`)

**`build_app_service()`**: return `Result<DefaultAppService, BootstrapError>`

**`update_with_option()`**:
- Return type: `Result<(), ErrorMessage>` -> `Result<(), BootstrapError>`
- Remove manual `ErrorMessage::new("secret_service_error", ...)` — use `?` directly (SecretServiceError auto-converts)

### 1.4: Update lib.rs re-exports

**File**: `crates/lib_bodhiserver/src/lib.rs`

- Remove `ErrorMessage` from `pub use objs::{...}` (line 30)
- Check if `ApiError` / `OpenAIApiError` are still needed by downstream; remove if not

### 1.5: Update tests

- `crates/lib_bodhiserver/src/error.rs` inline tests: update error message assertions
- Search for any test files referencing `AppOptionsError` or `AppServiceBuilderError` and update
- Error codes change (e.g., `app_options_error-validation_error` -> `bootstrap_error-validation_error`)

---

## Phase 2: bodhi/src-tauri — Sub-agent 2

**Crate**: `crates/bodhi/src-tauri` (package name: `bodhi`)
**Verify**: `cargo check -p bodhi` + `cargo check -p bodhi --features native` -> `cargo test -p bodhi` + `cargo test -p bodhi --features native`
**Commit**: `refactor: replace ErrorMessage with AppSetupError in bodhi/src-tauri`

### 2.1: Extend `AppSetupError`

**File**: `crates/bodhi/src-tauri/src/error.rs`

- Remove `From<AppSetupError> for ErrorMessage` impl
- Remove `ErrorMessage` import
- Add transparent variants:

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppSetupError {
  #[error(transparent)]
  Bootstrap(#[from] lib_bodhiserver::BootstrapError),

  #[error("Failed to start application: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AsyncRuntime(#[from] io::Error),

  #[error(transparent)]
  Serve(#[from] ServeError),

  #[error(transparent)]
  SettingService(#[from] SettingServiceError),
}
```

**Note**: `NativeError` is NOT added here (depends on `tauri::Error`, only available with `native` feature). Handled via explicit match in native_init.rs (Step 2.4).

Add `#[allow(dead_code)]` on variants if needed for feature-gated unused warnings.

### 2.2: Update `build_app_options()`

**File**: `crates/bodhi/src-tauri/src/common.rs`

- Return type: `Result<AppOptions, ErrorMessage>` -> `Result<AppOptions, BootstrapError>`
- `AppOptionsBuilder::build()` now returns `BootstrapError` directly, `?` works

### 2.3: Update server_init.rs

**File**: `crates/bodhi/src-tauri/src/server_init.rs`

- `initialize_and_execute()`: `Result<(), ErrorMessage>` -> `Result<(), AppSetupError>`
- `set_feature_settings()`: `Result<(), ErrorMessage>` -> `Result<(), AppSetupError>`
- `build_app_options()?` — BootstrapError auto-converts to `AppSetupError::Bootstrap`
- `setup_app_dirs()?` / `setup_bootstrap_service()?` — same
- `setup_logs().map_err(AppSetupError::AsyncRuntime)?` — explicit map for io::Error
- `Builder::new_multi_thread().build().map_err(AppSetupError::AsyncRuntime)?` — same
- `build_app_service(parts).await?` — BootstrapError auto-converts
- `command.aexecute().await?` — ServeError auto-converts to `AppSetupError::Serve` (remove manual `ErrorMessage::new("serve_error", ...)`)
- `set_feature_settings` (non-production): remove manual `ErrorMessage::new("setting_service_error", ...)`, use `?` (SettingServiceError auto-converts)

### 2.4: Update native_init.rs

**File**: `crates/bodhi/src-tauri/src/native_init.rs`

- `initialize_and_execute()`: `Result<(), ErrorMessage>` -> `Result<(), AppSetupError>`
- `build_app_options()?` / `setup_app_dirs()?` / `setup_bootstrap_service()?` — auto-convert
- `Builder::new_multi_thread().build().map_err(AppSetupError::AsyncRuntime)?`
- `build_app_service(parts).await?` — auto-converts
- `NativeCommand::aexecute()` error — explicit match:

```rust
.map_err(|err| match err {
  NativeError::Serve(e) => AppSetupError::Serve(e),
  other => AppSetupError::AsyncRuntime(std::io::Error::other(other.to_string())),
})?;
```

### 2.5: Delete orphan file + update tests

- **Delete**: `crates/bodhi/src-tauri/src/error_test.rs` (orphan, not in any module)
- **Update**: `crates/bodhi/src-tauri/src/error.rs` inline tests — rewrite to test `AppSetupError` display directly, remove `ErrorMessage` conversion assertions

### 2.6: app.rs — no changes needed

`initialize_and_execute` now returns `AppSetupError` which implements `Display`. `tracing::error!("fatal error: {err}")` still works. Log output improves from JSON to human-readable.

---

## Phase 3: objs — Sub-agent 3

**Crate**: `crates/objs`
**Verify**: `cargo check -p objs` -> `cargo test -p objs`
**Commit**: `refactor: remove ErrorMessage struct from objs`

### 3.1: Remove ErrorMessage struct

**File**: `crates/objs/src/error/common.rs`
- Remove `ErrorMessage` struct (lines 5-10)
- Remove `impl Display for ErrorMessage` (lines 12-17)
- Remove `use serde::Serialize` if unused by remaining types
- Remove `use derive_new` path reference if only ErrorMessage used it

### 3.2: Remove From impls

**File**: `crates/objs/src/error/error_api.rs`
- Remove `From<ApiError> for ErrorMessage` impl (lines 58-68)
- Remove `ErrorMessage` from imports

**File**: `crates/objs/src/error/error_oai.rs`
- Remove `From<OpenAIApiError> for ErrorMessage` impl (lines 62-70)
- Remove `ErrorMessage` from imports

---

## Phase 4: lib_bodhiserver_napi — Sub-agent 4

**Crate**: `crates/lib_bodhiserver_napi`
**Verify**: `cargo check -p lib_bodhiserver_napi`
**Commit**: `refactor: update lib_bodhiserver_napi for BootstrapError changes`

### 4.1: Update server.rs

**File**: `crates/lib_bodhiserver_napi/src/server.rs`

- `build_app_service()` returns `BootstrapError` — existing `.map_err(|e| Error::new(Status::GenericFailure, format!(...)))` still works
- `update_with_option()` returns `BootstrapError` — may need `.map_err(|err| Error::new(Status::GenericFailure, format!("{}", err)))` adjustment
- `AppOptionsBuilder::build()` returns `BootstrapError` — existing pattern still works

---

## Phase 5: Full verification — Sub-agent 5

```bash
make test.backend
```

Final commit if any residual fixes needed.

---

## Error Code Migration Reference

| Old Code | New Code |
|----------|----------|
| `app_options_error-validation_error` | `bootstrap_error-validation_error` |
| `app_options_error-parse_error` | `bootstrap_error-parse` |
| `app_options_error-unknown_system_setting` | `bootstrap_error-unknown_system_setting` |
| `app_service_builder_error-service_already_set` | `bootstrap_error-service_already_set` |
| `app_service_builder_error-placeholder_value` | `bootstrap_error-placeholder_value` |
| (new) | `bootstrap_error-missing_bootstrap_parts` |
| `app_setup_error-async_runtime` | `app_setup_error-async_runtime` (unchanged) |

## Files Modified (Summary)

| Phase | File | Action |
|-------|------|--------|
| 1 | `crates/lib_bodhiserver/src/error.rs` | Merge 3 enums into BootstrapError, add service variants, remove ErrorMessage impls |
| 1 | `crates/lib_bodhiserver/src/app_options.rs` | Return BootstrapError instead of AppOptionsError |
| 1 | `crates/lib_bodhiserver/src/app_service_builder.rs` | Return BootstrapError, remove ApiError usage, replace .expect() |
| 1 | `crates/lib_bodhiserver/src/lib.rs` | Remove ErrorMessage re-export |
| 2 | `crates/bodhi/src-tauri/src/error.rs` | Extend AppSetupError, remove ErrorMessage impl, update tests |
| 2 | `crates/bodhi/src-tauri/src/error_test.rs` | Delete (orphan file) |
| 2 | `crates/bodhi/src-tauri/src/common.rs` | Return BootstrapError |
| 2 | `crates/bodhi/src-tauri/src/server_init.rs` | Return AppSetupError, remove manual ErrorMessage construction |
| 2 | `crates/bodhi/src-tauri/src/native_init.rs` | Return AppSetupError, explicit NativeError match |
| 3 | `crates/objs/src/error/common.rs` | Remove ErrorMessage struct + Display impl |
| 3 | `crates/objs/src/error/error_api.rs` | Remove From<ApiError> for ErrorMessage |
| 3 | `crates/objs/src/error/error_oai.rs` | Remove From<OpenAIApiError> for ErrorMessage |
| 4 | `crates/lib_bodhiserver_napi/src/server.rs` | Adjust .map_err() for BootstrapError |
