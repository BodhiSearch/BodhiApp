---
name: Session PostgreSQL Migration
overview: Add PostgreSQL as an alternative session store backend alongside SQLite, using sqlx::Any for custom queries and enum dispatch for tower-sessions store types. This serves as the exploratory implementation for the future app database PostgreSQL migration.
todos:
  - id: phase-0-docker
    content: "Phase 0: Create docker-compose-test-deps.yml, add Makefile targets (test.deps.up, test.deps.down), document Docker as dev requirement"
    status: completed
  - id: phase-1-deps
    content: "Phase 1.1: Add postgres+any features to sqlx and tower-sessions-sqlx-store in workspace and services Cargo.toml; add serial_test dev-dep"
    status: completed
  - id: phase-1-module
    content: "Phase 1.2-1.4: Restructure session_service into folder module with AppSessionStoreExt trait, SessionStoreBackend enum, DefaultSessionService with AnyPool"
    status: completed
  - id: phase-1-settings
    content: "Phase 1.5: Add BODHI_SESSION_DB_URL and BODHI_DEPLOYMENT to SettingService, replace session_db_path() with session_db_url()"
    status: completed
  - id: phase-1-exports
    content: "Phase 1.6-1.7: Update lib.rs exports and test utilities (session.rs, app.rs)"
    status: completed
  - id: phase-1-tests
    content: "Phase 1.8: Write parameterized #[values(sqlite, postgres)] session tests with #[serial(pg_session)] for PG; branch migration assertions by backend"
    status: completed
  - id: phase-2-builder
    content: "Phase 2: Update AppServiceBuilder session construction to use URL-based backend selection with AnyPool + typed pool"
    status: completed
  - id: phase-3-auth
    content: "Phase 3: Update auth_middleware imports and session store access patterns (~6 test functions)"
    status: completed
  - id: phase-4-routes
    content: "Phase 4: Update routes_app imports and test utility builder methods (8 files)"
    status: completed
  - id: phase-5-server
    content: "Phase 5: Update server_app live server utils + add parameterized PG integration tests with #[serial(pg_session)]"
    status: completed
  - id: phase-6-napi
    content: "Phase 6: Export BODHI_SESSION_DB_URL and BODHI_DEPLOYMENT via NAPI, rebuild bindings"
    status: completed
  - id: phase-7-ci
    content: "Phase 7: Add PostgreSQL service container to .github/workflows/build.yml"
    status: completed
isProject: false
---

# Session Store PostgreSQL Migration

## Architecture Summary

Add PostgreSQL support for the session store (`tower-sessions`) while keeping SQLite as the default for standalone deployments. The session backend is selected at runtime based on `BODHI_SESSION_DB_URL` (URL scheme detection). This migration is an exploratory pattern for the larger app database migration to come.

### Key Architecture Decisions

- **Enum dispatch** for `SessionStoreBackend` (wrapping typed `SqliteStore` / `PostgresStore` from tower-sessions-sqlx-store)
- **sqlx::AnyPool** for our 5 custom queries (user_id tracking, session clearing, counting) -- write once, run on both backends
- **Two pools per session backend**: typed pool (for tower-sessions store construction) + AnyPool (for custom queries)
- `**BODHI_DEPLOYMENT=standalone|cluster`** added as forward infrastructure
- `**BODHI_SESSION_DB_URL`** replaces `session_db_path()` -- falls back to `sqlite:$BODHI_HOME/session.sqlite` when unset
- **Docker required for development** -- all tests (including PG session) must pass in `make test.backend`
- Backend-specific migration SQL detected from URL scheme, run via AnyPool

### Test Coverage Strategy

**services crate**: Parameterized `#[values("sqlite", "postgres")]` tests for session operations. PG tests use `#[serial(pg_session)]`. Tests **fail** when PG is unavailable (not skipped).

**server_app crate**: Parameterized live server tests exercising session flows against both backends. `#[serial(pg_session)]` for PG variants.

**auth_middleware, routes_app**: Import renames only (no new PG-specific tests here).

**CI (build.yml)**: GitHub Actions service container for PostgreSQL.

Each of the phase will be implmented by specialized sub-agent, launched with detailed instruction on the task, provided reference, and gate check test to ensure there are no failures, implementing agent to return summary of changes, make a local commit, and specialized sub-agent to be launched for next phase follow same process

---

## Phase 0: Development Prerequisites

### docker-compose-test-deps.yml (project root)

- Service: `bodhi_session_db` (postgres:17)
- Port: `54320:5432` (non-standard to avoid conflicts)
- Database: `bodhi_sessions`, user: `bodhi_test`, password: `bodhi_test`
- Health check: `pg_isready`

### Makefile targets

- `test.deps.up` -- `docker compose -f docker-compose-test-deps.yml up -d --wait`
- `test.deps.down` -- `docker compose -f docker-compose-test-deps.yml down -v`
- Update `test.backend` to document Docker dependency

### Verification

- `make test.deps.up` starts PG, `make test.deps.down` stops it

---

## Phase 1: services crate -- Session Store Abstraction

### 1.1 Add Dependencies

**Workspace `Cargo.toml`**: Add `postgres` and `any` features to `sqlx` and `tower-sessions-sqlx-store`.

`**crates/services/Cargo.toml**`: Add `postgres`, `any` features. Add `serial_test` as dev-dependency.

Files: [Cargo.toml](Cargo.toml), [crates/services/Cargo.toml](crates/services/Cargo.toml)

### 1.2 Restructure session_service Module

Convert `crates/services/src/session_service.rs` to folder module:

```
session_service/
  mod.rs       -- trait definition, error type, re-exports
  sqlite.rs    -- SqliteSessionStore (wraps SqliteStore + SqlitePool)
  postgres.rs  -- PgSessionStore (wraps PostgresStore + PgPool)
  service.rs   -- SessionStoreBackend enum, DefaultSessionService, AnyPool custom queries
```

Test file: `crates/services/src/test_session_service.rs` (sibling file declared from parent, following project convention)

### 1.3 AppSessionStoreExt Trait

Extract custom session operations into a trait:

```rust
#[async_trait]
trait AppSessionStoreExt: Send + Sync {
  async fn migrate_custom(&self) -> Result<()>;
  async fn clear_sessions_for_user(&self, user_id: &str) -> Result<usize>;
  async fn clear_all_sessions(&self) -> Result<usize>;
  async fn count_sessions_for_user(&self, user_id: &str) -> Result<i32>;
  async fn get_session_ids_for_user(&self, user_id: &str) -> Result<Vec<String>>;
  async fn dump_all_sessions(&self) -> Result<Vec<(String, Option<String>)>>;
}
```

### 1.4 SessionStoreBackend Enum + DefaultSessionService

```rust
enum SessionStoreBackend {
  Sqlite(SqliteStore),
  Postgres(PostgresStore),
}
impl SessionStore for SessionStoreBackend { /* match dispatch */ }

struct DefaultSessionService {
  store_backend: SessionStoreBackend,
  any_pool: AnyPool,
}
```

- `DefaultSessionService` implements both `SessionService` and `AppSessionStoreExt`
- Custom queries use `any_pool` with `?` bind syntax
- Migration detection: parse URL scheme, run SQLite-specific (`pragma_table_info`) or PG-specific (`ADD COLUMN IF NOT EXISTS`) SQL via `any_pool`
- `sqlx::any::install_default_drivers()` called in factory method

### 1.5 Add BODHI_SESSION_DB_URL and BODHI_DEPLOYMENT to SettingService

- New method `session_db_url()` on `SettingService` trait (default impl): checks `BODHI_SESSION_DB_URL` env var, falls back to `sqlite:{session_db_path}`
- Remove `session_db_path()` usages (replaced by `session_db_url()`)
- New method `deployment_mode()`: reads `BODHI_DEPLOYMENT`, defaults to `"standalone"`

Files: [crates/services/src/setting_service/service.rs](crates/services/src/setting_service/service.rs)

### 1.6 Update Exports

[crates/services/src/lib.rs](crates/services/src/lib.rs): Replace `mod session_service;` with folder module. Export `DefaultSessionService`, `SessionStoreBackend`, `AppSessionStoreExt`. Remove `SqliteSessionService`, `AppSessionStore` exports.

### 1.7 Update Test Utilities

**[crates/services/src/test_utils/session.rs](crates/services/src/test_utils/session.rs)**:

- Rename `SqliteSessionService` -> `DefaultSessionService`
- `build_session_service(dbfile)` creates SQLite backend
- Add `build_pg_session_service(url)` for PG backend
- `SessionTestExt` uses `AppSessionStoreExt` trait methods

**[crates/services/src/test_utils/app.rs](crates/services/src/test_utils/app.rs)**:

- `with_session_service()` -> creates SQLite session (unchanged behavior)
- `with_sqlite_session_service()` -> `with_default_session_service()`
- Update type from `Arc<SqliteSessionService>` to `Arc<DefaultSessionService>`

### 1.8 Write Parameterized Tests

`crates/services/src/test_session_service.rs` with `#[values("sqlite", "postgres")]`:

- **Migration tests**: Branch on backend for assertion queries (SQLite uses `pragma_table_info`, PG uses `information_schema.columns`)
- **Session CRUD tests**: save with user_id, save without user_id, load, delete
- **Custom operation tests**: clear_sessions_for_user, clear_all_sessions, count_sessions_for_user, multi-user isolation
- PG tests annotated with `#[serial(pg_session)]`
- PG tests connect to `postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions`

### Verification

- `cargo check -p services`
- `make test.deps.up && cargo test -p services`
- `cargo fmt`

---

## Phase 2: lib_bodhiserver crate -- AppServiceBuilder

### Changes

**[crates/lib_bodhiserver/src/app_service_builder.rs](crates/lib_bodhiserver/src/app_service_builder.rs)**:

- `build_session_service()`: Read `session_db_url()` from SettingService, call `sqlx::any::install_default_drivers()`, then:
  - Parse URL scheme
  - Create typed pool (SqlitePool or PgPool) for tower-sessions store construction
  - Create `AnyPool` from same URL for custom queries
  - Construct `SessionStoreBackend::Sqlite(SqliteStore)` or `SessionStoreBackend::Postgres(PostgresStore)`
  - Construct `DefaultSessionService::new(backend, any_pool)`
  - Run migration
- Update imports: remove `SqliteSessionService`, add new types

### Verification

- `cargo test -p services -p lib_bodhiserver`
- `cargo fmt`

---

## Phase 3: auth_middleware crate -- Import Updates

### Changes

**[crates/auth_middleware/src/auth_middleware/tests.rs](crates/auth_middleware/src/auth_middleware/tests.rs)**:

- Replace `SqliteSessionService` with `DefaultSessionService` (~6 test functions)
- `session_service.session_store.create()` -> `session_service.get_session_store().create()` (or equivalent via `AppSessionStoreExt` / `SessionStore` trait)
- `session_service.session_store.load()` -> `session_service.get_session_store().load()`

### Verification

- `cargo test -p services -p lib_bodhiserver -p auth_middleware`

---

## Phase 4: routes_app crate -- Import and Test Utility Updates

### Changes

8 files affected (count from grep):

- `test_login_callback.rs`, `test_login_initiate.rs`, `test_login_logout.rs`, `test_login_resource_admin.rs` -- import renames + field-to-method access
- `test_access_request_admin.rs` -- import renames + builder method rename
- `test_utils/router.rs` -- import renames
- `routes_dev.rs` -- import renames
- `test_oauth_flow.rs` -- import renames

Pattern: `SqliteSessionService` -> `DefaultSessionService`, `.session_store.X()` -> `.get_session_store().X()`, `.with_sqlite_session_service()` -> `.with_default_session_service()`

### Verification

- `cargo test -p services -p lib_bodhiserver -p auth_middleware -p routes_app`

---

## Phase 5: server_app crate -- Live Server Test Updates + PG Integration Tests

### 5.1 Update Existing SQLite Setup

**[crates/server_app/tests/utils/live_server_utils.rs](crates/server_app/tests/utils/live_server_utils.rs)**:

- 2 functions (`setup_minimal_app_service`, `setup_test_app_service`) construct `SqliteSessionService` directly
- Refactor to use `DefaultSessionService` with `SessionStoreBackend::Sqlite`
- Extract session service construction into a parameterizable helper

### 5.2 Add PG Integration Tests

Parameterize live server tests that exercise session flows:

- Login/logout flow with PG session backend
- Session persistence across requests
- Session clearing on user update
- `#[serial(pg_session)]` for PG variants
- PG URL: `postgres://bodhi_test:bodhi_test@localhost:54320/bodhi_sessions`

### Verification

- `make test.deps.up && cargo test -p server_app`
- `make test.backend`

---

## Phase 6: lib_bodhiserver_napi crate -- NAPI Config Export

### Changes

**[crates/lib_bodhiserver_napi/src/config.rs](crates/lib_bodhiserver_napi/src/config.rs)**:

```rust
#[napi]
pub const BODHI_SESSION_DB_URL: &str = "BODHI_SESSION_DB_URL";

#[napi]
pub const BODHI_DEPLOYMENT: &str = "BODHI_DEPLOYMENT";
```

Rebuild NAPI bindings: `cd crates/lib_bodhiserver_napi && npm run build:release`

### Verification

- `cargo check -p lib_bodhiserver_napi`
- `make test.backend`

---

## Phase 7: CI Integration

### .github/workflows/build.yml

Add PostgreSQL service container to `build-and-test` job:

```yaml
services:
  postgres:
    image: postgres:17
    env:
      POSTGRES_DB: bodhi_sessions
      POSTGRES_USER: bodhi_test
      POSTGRES_PASSWORD: bodhi_test
    ports:
      - 54320:5432
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

### Verification

- CI pipeline runs all backend tests including PG session tests

---

## Files Changed Summary


| Crate                | Files Modified                                                                                     | Files Created                                                                                                                                 |
| -------------------- | -------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| root                 | `Cargo.toml`, `Makefile`                                                                           | `docker-compose-test-deps.yml`                                                                                                                |
| services             | `Cargo.toml`, `lib.rs`, `setting_service/service.rs`, `test_utils/session.rs`, `test_utils/app.rs` | `session_service/mod.rs`, `session_service/sqlite.rs`, `session_service/postgres.rs`, `session_service/service.rs`, `test_session_service.rs` |
| lib_bodhiserver      | `app_service_builder.rs`                                                                           | --                                                                                                                                            |
| auth_middleware      | `auth_middleware/tests.rs`                                                                         | --                                                                                                                                            |
| routes_app           | 8 test/util files (import renames)                                                                 | --                                                                                                                                            |
| server_app           | `tests/utils/live_server_utils.rs`                                                                 | PG integration test file(s)                                                                                                                   |
| lib_bodhiserver_napi | `src/config.rs`                                                                                    | --                                                                                                                                            |
| .github              | `workflows/build.yml`                                                                              | --                                                                                                                                            |


## Deleted Files

- `crates/services/src/session_service.rs` (replaced by `session_service/` folder module)

