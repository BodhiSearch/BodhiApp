# Phase 1: services crate -- Session Store Abstraction

## Functional Goal

Transform the session service from a SQLite-only implementation to a backend-agnostic service that supports both SQLite and PostgreSQL, selected at runtime based on configuration.

## Prerequisites

- Discover main's current `session_service.rs` structure (it was a single file in the worktree; main may have reorganized)
- Discover main's current `SettingService` trait location and pattern
- Discover main's current `Cargo.toml` workspace dependency versions for `sqlx` and `tower-sessions-sqlx-store`

## Changes

### 1. Add PostgreSQL Dependencies

Add `postgres` and `any` features to both `sqlx` and `tower-sessions-sqlx-store` in the workspace and services crate Cargo.toml files. This enables compilation of PostgreSQL session store support alongside the existing SQLite support.

Also add `serial_test` as a dev-dependency for the services crate (needed for PostgreSQL tests that share a single database).

**Verification**: `cargo check -p services` passes.

### 2. Restructure Session Service Module

Convert the single `session_service.rs` file into a folder module with separate files per backend. Suggested structure:

```
session_service/
  mod.rs       -- module declarations, trait definition, re-exports
  error.rs     -- SessionServiceError type
  sqlite.rs    -- SqliteSessionStore wrapper
  postgres.rs  -- PgSessionStore wrapper
  service.rs   -- DefaultSessionService, SessionStoreBackend enum
  tests.rs     -- parameterized tests for both backends
```

The implementer may choose a different organization if main's patterns have evolved.

### 3. Create Backend-Specific Session Stores

Each backend wraps the corresponding `tower-sessions-sqlx-store` type and adds custom functionality (user_id column tracking, session clearing by user).

**SqliteSessionStore** wraps `tower_sessions_sqlx_store::SqliteStore` + `SqlitePool`:
- `new(pool: SqlitePool)` constructor
- `migrate()` -- runs tower-sessions migration, adds `user_id` TEXT column if missing (via `pragma_table_info` check), creates index
- `clear_sessions_for_user(user_id)` -- deletes sessions by user_id
- `clear_all_sessions()` -- deletes all sessions
- `count_sessions_for_user(user_id)` -- counts sessions for a user
- `get_session_ids_for_user(user_id)` -- lists session IDs for a user
- `dump_all_sessions()` -- lists all sessions (debugging)
- Implements `SessionStore` trait (save/load/delete delegation to inner store, with user_id column update on save)

**PgSessionStore** wraps `tower_sessions_sqlx_store::PostgresStore` + `PgPool`:
- `connect(url: &str)` constructor (creates PgPool internally)
- `migrate()` -- runs tower-sessions migration, adds `user_id` column with `ADD COLUMN IF NOT EXISTS` (PG syntax), creates index
- Same custom methods as SQLite variant but with PostgreSQL bind syntax (`$1`, `$2` instead of `?`)
- Implements `SessionStore` trait (same delegation pattern)

### 4. Create SessionStoreBackend Enum and DefaultSessionService

**SessionStoreBackend** is an enum dispatching to the appropriate backend:

```
SessionStoreBackend::Sqlite(SqliteSessionStore)
SessionStoreBackend::Postgres(PgSessionStore)
```

It implements `SessionStore` by delegating to the inner store, and exposes the custom session management methods via match dispatch.

**DefaultSessionService** (renamed from `SqliteSessionService`) wraps `SessionStoreBackend`:
- `new(backend: SessionStoreBackend)` constructor
- Implements the `SessionService` trait:
  - `session_layer()` -- creates `SessionManagerLayer` (SameSite::Strict, cookie name `bodhiapp_session_id`)
  - `clear_sessions_for_user(user_id)` -- delegates to backend
  - `clear_all_sessions()` -- delegates to backend
  - `get_session_store()` -- returns reference to backend (for test utilities that need direct store access)

### 5. Add BODHI_SESSION_DB_URL to SettingService

Add a new environment variable constant `BODHI_SESSION_DB_URL` to the setting service module.

Add a `session_db_url()` method to the `SettingService` trait with a default implementation:
- If `BODHI_SESSION_DB_URL` is set, return its value
- Otherwise, fall back to `sqlite:{session_db_path}` (preserving existing behavior)

Keep the existing `session_db_path()` method for backwards compatibility.

### 6. Add BODHI_DEPLOYMENT Configuration

Add a new environment variable constant `BODHI_DEPLOYMENT` to the setting service module.

Add a `deployment_mode()` method to the `SettingService` trait:
- Returns `"standalone"` (default) or `"cluster"`
- When `cluster`, `BODHI_SESSION_DB_URL` is expected to be a `postgres://` URL

This is a foundational config that will be used by later phases (app DB, org threading, etc.).

### 7. Update Exports and Imports

Update `crates/services/src/lib.rs` to:
- Replace `mod session_service;` with the folder module (if restructured)
- Export new public types: `DefaultSessionService`, `SessionStoreBackend`, `SqliteSessionStore`, `PgSessionStore`
- Keep `SessionService` trait and `SessionServiceError` exports

### 8. Create docker-compose-test-deps.yml

Create a docker-compose file in the project root for test PostgreSQL:

- Service name: `bodhi_session_db` (not generic `postgres` -- future phases will add app DB)
- Image: `postgres:17` (or latest stable)
- Port: `54320:5432` (non-standard to avoid conflicts with local PostgreSQL)
- Database: `bodhi_sessions`
- User/password: `bodhi_test` / `bodhi_test`
- Health check on `pg_isready`

### 9. Add Makefile Targets

Add these targets to the root Makefile:

- `test.deps.up` -- start test dependencies via docker-compose
- `test.deps.down` -- stop test dependencies and remove volumes
- `test.session.all` -- start deps, run session tests, stop deps

### 10. Write Parameterized Tests

Write tests that validate both backends produce identical behavior. Suggested approach using rstest `#[values]`:

- A fixture function takes a backend type parameter (e.g., `"sqlite"` or `"postgres"`)
- For SQLite: creates temp file, connects SqlitePool, builds SqliteSessionStore
- For PostgreSQL: connects to docker-compose PG at `localhost:54320`, builds PgSessionStore
- PostgreSQL tests should gracefully skip or fail clearly if docker-compose is not running

Tests to cover:
- Session migration runs without error
- Save session with user_id tracking
- Load session returns correct data
- Delete session removes it
- Clear sessions for a specific user
- Clear all sessions
- Count sessions for a user
- User_id index is created
- Multiple sessions for same user
- Sessions for different users are isolated

### 11. Update Test Utilities

Update `crates/services/src/test_utils/session.rs`:
- Rename `SqliteSessionService` references to `DefaultSessionService`
- Update `build_session_service(dbfile)` to construct `SqliteSessionStore` first, then wrap in `SessionStoreBackend::Sqlite`, then `DefaultSessionService::new`
- Update `SessionTestExt` implementation to use `get_session_store()` method instead of direct field access

Update `crates/services/src/test_utils/app.rs`:
- Rename `SqliteSessionService` import to `DefaultSessionService`
- Rename `with_sqlite_session_service()` method to `with_default_session_service()`
- Update the method parameter type from `Arc<SqliteSessionService>` to `Arc<DefaultSessionService>`

## Verification

1. `cargo check -p services` -- compiles with new types
2. `cargo test -p services` -- all existing tests pass (SQLite backend)
3. `cargo test -p services session_service` -- session-specific tests pass
4. With docker-compose up: PostgreSQL-parameterized tests also pass
5. `cargo fmt` -- clean formatting
