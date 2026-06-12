# Plan: Post-Implementation Feedback for Session-to-PG Migration

## Context

This plan addresses feedback on the completed session-to-PostgreSQL migration (plan: `20260224-session-to-pg.plan.md`). The feedback identifies six areas needing refinement: module structure violations, error handling, settings architecture, test quality, and documentation gaps.

---

## Step 1: Restructure session_service module (services crate)

**Problem**: `session_service/mod.rs` contains trait definitions and error types, violating the project convention that mod.rs should only contain module declarations and re-exports.

**Files to modify:**
- `crates/services/src/session_service/mod.rs` — strip to declarations + re-exports only
- `crates/services/src/session_service/service.rs` — rename to `session_service.rs`
- New: `crates/services/src/session_service/error.rs`
- New: `crates/services/src/session_service/session_store.rs`

**Changes:**

### 1a. Create `error.rs`
Move from `mod.rs`:
- `SessionServiceError` enum (lines 12-20)
- `impl From<tower_sessions::session_store::Error>` (lines 22-26)
- `impl_error_from!` bridge (lines 28-32)
- `SessionResult<T>` type alias (line 34)

Add new variant:
```rust
#[error("Session DB setup error: {0}")]
#[error_meta(error_type = ErrorType::InternalServer)]
DbSetup(String),
```

### 1b. Create `session_store.rs`
Move from current `service.rs`:
- `InnerStoreShared` struct + impl
- `SessionStoreBackend` struct + `impl SessionStore for SessionStoreBackend`
- `is_postgres_url()` function
- Constructors: `SessionStoreBackend::new_sqlite()`, `new_postgres()`, `any_pool()`

### 1c. Rename `service.rs` → `session_service.rs`
Keep:
- `SessionService` trait (moved from mod.rs lines 36-44)
- `AppSessionStoreExt` trait (moved from mod.rs lines 46-54)
- `DefaultSessionService` struct + all impls
- `SessionService` impl for `DefaultSessionService`
- `AppSessionStoreExt` impl for `DefaultSessionService`

### 1d. Update `mod.rs`
```rust
mod error;
mod postgres;
mod session_service;
mod session_store;
mod sqlite;

pub use error::*;
pub use session_service::*;
pub use session_store::*;
```

### 1e. Update `postgres.rs` — remove `expect`
Change `create_postgres_store` to return `SessionResult<PostgresStore>`:
```rust
pub(crate) fn create_postgres_store(pool: PgPool) -> SessionResult<PostgresStore> {
  Ok(PostgresStore::new(pool)
    .with_schema_name("public")
    .map_err(|e| SessionServiceError::DbSetup(format!("invalid schema name: {e}")))?
    .with_table_name("tower_sessions")
    .map_err(|e| SessionServiceError::DbSetup(format!("invalid table name: {e}")))?)
}
```
Update call site in `session_service.rs` `connect_postgres()` to use `?` instead of direct call.

---

## Step 2: Settings architecture cleanup (services crate)

**Problem**: `session_db_url()` and `deployment_mode()` have inline fallbacks instead of using the settings hierarchy defaults system.

**Files to modify:**
- `crates/services/src/setting_service/default_service.rs` — extend `build_all_defaults`
- `crates/services/src/setting_service/service.rs` — simplify trait methods
- `crates/services/src/setting_service/constants.rs` — add to `SETTING_VARS`
- `crates/services/src/test_utils/envs.rs` — seed stub defaults

### 2a. Extend `build_all_defaults` signature
File: `crates/services/src/setting_service/default_service.rs`

Change signature:
```rust
fn build_all_defaults(
  env_wrapper: &dyn EnvWrapper,
  file_defaults: &HashMap<String, Value>,
  bodhi_home: &Path,  // NEW
) -> HashMap<String, Value> {
```

Add at end of function (before `defaults` return):
```rust
ensure_default!(
  BODHI_SESSION_DB_URL,
  Value::String(format!("sqlite:{}", bodhi_home.join(SESSION_DB).display()))
);
ensure_default!(BODHI_DEPLOYMENT, Value::String("standalone".to_string()));
```

Update call site in `from_parts()` (line 94):
```rust
let defaults = build_all_defaults(parts.env_wrapper.as_ref(), &parts.file_defaults, &parts.bodhi_home);
```

Add imports: `SESSION_DB`, `BODHI_SESSION_DB_URL`, `BODHI_DEPLOYMENT`, `Path`.

### 2b. Add to SETTING_VARS
File: `crates/services/src/setting_service/constants.rs`

Add `BODHI_SESSION_DB_URL` and `BODHI_DEPLOYMENT` to the `SETTING_VARS` array. These will be visible in the settings list API but not editable (gated by `EDIT_SETTINGS_ALLOWED` in routes_app).

### 2c. Simplify `session_db_url()` and `deployment_mode()`
File: `crates/services/src/setting_service/service.rs`

Remove `session_db_path()` method from the trait entirely.

Replace `session_db_url()`:
```rust
async fn session_db_url(&self) -> String {
  self.get_setting(BODHI_SESSION_DB_URL)
    .await
    .expect("BODHI_SESSION_DB_URL should have a default")
}
```

Replace `deployment_mode()`:
```rust
async fn deployment_mode(&self) -> String {
  self.get_setting(BODHI_DEPLOYMENT)
    .await
    .expect("BODHI_DEPLOYMENT should have a default")
}
```

### 2d. Update SettingServiceStub
File: `crates/services/src/test_utils/envs.rs`

Add to the `setup()` HashMap (around line 157):
```rust
(BODHI_SESSION_DB_URL.to_string(), format!("sqlite:{}", bodhi.join("session.sqlite").display())),
(BODHI_DEPLOYMENT.to_string(), "standalone".to_string()),
```

---

## Step 3: Test improvements (services crate)

**Problem**: Tests use hardcoded PG URL, test raw SQL instead of service methods, and don't reuse existing fixtures.

**Files to modify:**
- `crates/services/src/test_session_service.rs` — refactor tests
- New: `crates/services/.env.test` — PG test URL
- `crates/services/src/test_utils/session.rs` — verify fixture reuse

### 3a. Create `.env.test`
File: `crates/services/.env.test`
```
INTEG_TEST_SESSION_PG_URL=postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions
```

### 3b. Replace hardcoded PG_URL
File: `crates/services/src/test_session_service.rs`

Remove: `const PG_URL: &str = "postgres://...";`

Add a fixture that loads from env (following auth_middleware pattern at `crates/auth_middleware/tests/test_live_auth_middleware.rs:120-136`):
```rust
fn pg_url() -> String {
  let env_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.test"));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }
  std::env::var("INTEG_TEST_SESSION_PG_URL")
    .expect("INTEG_TEST_SESSION_PG_URL must be set")
}
```

### 3c. Remove `test_session_service_migration`
This test directly queries schema metadata (pragma_table_info, information_schema) — it tests SQL queries, not production service methods. The other tests (`test_session_service_save_with_user_id`, `test_session_service_clear_sessions_for_user`, etc.) implicitly verify migration succeeded. Remove it entirely.

### 3d. Fix test fixture: reuse `build_session_service`
In `create_session_service` for the `"sqlite"` branch, replace manual `TempDir::new()` + `std::mem::forget` with the existing factory from `test_utils/session.rs`:
```rust
// Use existing factory
"sqlite" => {
  let temp_dir = tempfile::TempDir::new().unwrap();
  let dbfile = temp_dir.path().join("test_sessions.sqlite");
  let service = DefaultSessionService::build_session_service(dbfile).await;
  // Keep temp_dir alive by returning it alongside service, or use rstest fixture
  std::mem::forget(temp_dir); // temp_dir ownership handled by test lifetime
  service
}
```

Note: `build_session_service` (from `test_utils/session.rs`) already handles file creation and `connect_sqlite`. The `std::mem::forget` pattern remains necessary here since the tests are parameterized and the temp dir must outlive the test. If a better pattern exists (e.g., returning a tuple), apply it.

---

## Step 4: Documentation

### 4a. Create `crates/CLAUDE.md`
New shared conventions file for all Rust crates.

Contents:
```markdown
# crates/CLAUDE.md - Shared Rust Conventions

## Module Organization
- `mod.rs` files must contain ONLY module declarations (`mod xxx;`) and re-exports (`pub use xxx::*;`).
  No trait definitions, error enums, structs, or implementation code in mod.rs.
- For service modules with multiple concerns, split into: `error.rs`, `service.rs` (or domain-named files), and `mod.rs` for wiring.
```

### 4b. Create `crates/routes_app/TECHDEBT.md`
```markdown
# routes_app Technical Debt

## Move EDIT_SETTINGS_ALLOWED to settings_service
- Currently: `EDIT_SETTINGS_ALLOWED` is defined in `crates/routes_app/src/routes_settings/route_settings.rs`
- Should be: Moved to `crates/services/src/setting_service/` so the editability allowlist lives alongside SETTING_VARS
- Reason: Setting visibility (SETTING_VARS) and editability (EDIT_SETTINGS_ALLOWED) are related concerns that should be co-located in the settings service
```

### 4c. Update `crates/services/CLAUDE.md`
Update the "Session Security" section to reflect the new module structure:
- Session service now split into `error.rs`, `session_store.rs`, `session_service.rs`, `postgres.rs`, `sqlite.rs`
- `SessionStoreBackend` wraps both SQLite and Postgres backends
- `AppSessionStoreExt` provides custom user_id tracking operations
- `SessionServiceError::DbSetup` variant for store configuration errors

---

## Verification

1. `cargo check -p services` — verify services crate compiles
2. `cargo test -p services` — run all services tests (requires docker-compose PG running)
3. `cargo check -p lib_bodhiserver` — verify downstream compiles
4. `cargo check -p routes_app` — verify routes compile with SETTING_VARS change
5. `make test.backend` — full backend regression

---

## Files Changed Summary

| File | Action |
|---|---|
| `crates/services/src/session_service/mod.rs` | Strip to declarations + re-exports |
| `crates/services/src/session_service/error.rs` | NEW — error enum, result alias |
| `crates/services/src/session_service/session_store.rs` | NEW — SessionStoreBackend, is_postgres_url |
| `crates/services/src/session_service/session_service.rs` | RENAME from service.rs — traits + DefaultSessionService |
| `crates/services/src/session_service/postgres.rs` | Return `SessionResult` instead of `expect` |
| `crates/services/src/setting_service/default_service.rs` | Extend `build_all_defaults` with `bodhi_home` param |
| `crates/services/src/setting_service/service.rs` | Remove `session_db_path()`, simplify `session_db_url()` and `deployment_mode()` |
| `crates/services/src/setting_service/constants.rs` | Add to `SETTING_VARS` |
| `crates/services/src/test_utils/envs.rs` | Seed BODHI_SESSION_DB_URL + BODHI_DEPLOYMENT in stub |
| `crates/services/src/test_session_service.rs` | Remove migration test, load PG URL from env |
| `crates/services/.env.test` | NEW — INTEG_TEST_SESSION_PG_URL |
| `crates/CLAUDE.md` | NEW — shared Rust conventions |
| `crates/routes_app/TECHDEBT.md` | NEW — EDIT_SETTINGS_ALLOWED tech debt |
| `crates/services/CLAUDE.md` | Update session service docs |
