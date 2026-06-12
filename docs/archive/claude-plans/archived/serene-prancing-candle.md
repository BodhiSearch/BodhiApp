# Plan: Move BootstrapService from services to lib_bodhiserver

## Context

`BootstrapService` is defined in `services` but is only constructed (`::new()`) and consumed (`.bodhi_home()`, `.into_parts()`) inside `lib_bodhiserver`. Downstream crates (`bodhi/src-tauri`, `lib_bodhiserver_napi`) already import it from `lib_bodhiserver`. Moving it makes the ownership explicit and reduces coupling.

`BootstrapParts` must stay in `services` because `DefaultSettingService::from_parts(parts: BootstrapParts, ...)` lives there, and `services` cannot depend on `lib_bodhiserver`.

`BootstrapServiceError` (single variant: `BodhiHomeNotFound`) can be eliminated — `BootstrapService::new()` will return `AppDirsBuilderError` directly since it's now in the same crate.

---

## Phase 1: services crate — remove BootstrapService, keep BootstrapParts

### 1.1 Replace `bootstrap_service.rs` with `bootstrap_parts.rs`

**Delete** `crates/services/src/setting_service/bootstrap_service.rs`

**Create** `crates/services/src/setting_service/bootstrap_parts.rs` containing only `BootstrapParts`:
- Imports: `crate::EnvWrapper`, `objs::{AppCommand, Setting}`, `serde_yaml::Value`, std types
- Struct `BootstrapParts` with 7 `pub` fields (unchanged from current)

### 1.2 Remove `BootstrapServiceError` from error.rs

**File**: `crates/services/src/setting_service/error.rs`

Delete the `BootstrapServiceError` enum (lines 33-39).

### 1.3 Update mod.rs

**File**: `crates/services/src/setting_service/mod.rs`

Change `mod bootstrap_service` → `mod bootstrap_parts`, and `pub use bootstrap_service::*` → `pub use bootstrap_parts::*`.

### 1.4 No-change verification

These files import only `BootstrapParts` (not `BootstrapService`) — no changes needed:
- `crates/services/src/setting_service/default_service.rs` (line 2: `use super::BootstrapParts`)
- `crates/services/src/setting_service/tests.rs` (line 3)
- `crates/services/src/setting_service/test_service_db.rs` (line 4)

**Verify**: `cargo test -p services`

---

## Phase 2: lib_bodhiserver crate — add BootstrapService

### 2.1 Create `bootstrap_service.rs`

**Create** `crates/lib_bodhiserver/src/bootstrap_service.rs`

Move `BootstrapService` struct + impl from the deleted file. Key adaptations:
- Import constants from `services::` (`BODHI_HOME`, `BODHI_LOGS`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`, `DEFAULT_LOG_LEVEL`, `DEFAULT_LOG_STDOUT`, `LOGS_DIR`)
- Import `services::{BootstrapParts, EnvWrapper}`
- Import `objs::{AppCommand, LogLevel, Setting}`
- `new()` returns `Result<Self, AppDirsBuilderError>` instead of `Result<Self, BootstrapServiceError>`
- Replace `BootstrapServiceError::BodhiHomeNotFound` → `AppDirsBuilderError::BootstrapBodhiHomeNotFound`

### 2.2 Update error.rs

**File**: `crates/lib_bodhiserver/src/error.rs`

- Remove `Bootstrap(#[from] services::BootstrapServiceError)` variant from `AppDirsBuilderError`
- Add `BootstrapBodhiHomeNotFound` variant with same message `"BODHI_HOME value must be set"` and `ErrorType::InternalServer`
- Add test case for the new variant in the rstest at bottom of file

### 2.3 Register module and update re-exports in lib.rs

**File**: `crates/lib_bodhiserver/src/lib.rs`

- Add `mod bootstrap_service;` (after line 8)
- Add `pub use bootstrap_service::BootstrapService;` (after line 22)
- Remove `BootstrapService` from the `pub use services::{...}` block (line 36)

### 2.4 Update app_dirs_builder.rs imports

**File**: `crates/lib_bodhiserver/src/app_dirs_builder.rs`

- Move `BootstrapService` from `use services::{BootstrapService, ...}` to `use crate::BootstrapService;`
- `BootstrapService::new(...)` now returns `Result<Self, AppDirsBuilderError>` — the `?` in `setup_bootstrap_service()` works directly, no `#[from]` needed.

### 2.5 Update app_service_builder.rs imports

**File**: `crates/lib_bodhiserver/src/app_service_builder.rs`

- Move `BootstrapService` from `use services::{...BootstrapService...}` to `use crate::BootstrapService;`

**Verify**: `cargo test -p services -p lib_bodhiserver`

---

## Phase 3: Downstream crates — no changes expected

These crates import `BootstrapService` from `lib_bodhiserver` (not `services`), so no changes needed:
- `crates/bodhi/src-tauri/src/server_init.rs` (line 3)
- `crates/lib_bodhiserver_napi/src/server.rs` (line 7)

These test files use `services::BootstrapParts` (still in services), so no changes needed:
- `crates/routes_app/src/routes_settings/test_settings.rs`
- `crates/server_app/tests/utils/live_server_utils.rs`

**Verify**: `make test.backend`

---

## Phase 4: Documentation

Update `crates/lib_bodhiserver/PACKAGE.md` — add `src/bootstrap_service.rs` to module structure, note BootstrapService lives here.

Update `crates/services/PACKAGE.md` — note BootstrapParts (data carrier) remains, BootstrapService moved to lib_bodhiserver.

---

## Summary of changes

| File | Action |
|------|--------|
| `services/src/setting_service/bootstrap_service.rs` | **Delete** |
| `services/src/setting_service/bootstrap_parts.rs` | **Create** (BootstrapParts only) |
| `services/src/setting_service/error.rs` | Remove `BootstrapServiceError` |
| `services/src/setting_service/mod.rs` | `bootstrap_service` → `bootstrap_parts` |
| `lib_bodhiserver/src/bootstrap_service.rs` | **Create** (BootstrapService struct+impl) |
| `lib_bodhiserver/src/error.rs` | Replace `Bootstrap(#[from])` with `BootstrapBodhiHomeNotFound` |
| `lib_bodhiserver/src/lib.rs` | Add module, move re-export |
| `lib_bodhiserver/src/app_dirs_builder.rs` | Import from `crate` instead of `services` |
| `lib_bodhiserver/src/app_service_builder.rs` | Import from `crate` instead of `services` |

No Cargo.toml changes needed — `lib_bodhiserver` already depends on `services`, `objs`, `serde_yaml`.

## Verification

```bash
cargo test -p services
cargo test -p lib_bodhiserver
make test.backend
```
