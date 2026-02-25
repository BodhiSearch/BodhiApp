# Phase 2: lib_bodhiserver crate -- AppServiceBuilder

## Functional Goal

Update the application service builder to select between SQLite and PostgreSQL session backends at startup based on the `BODHI_SESSION_DB_URL` configuration, using the new types from Phase 1.

## Prerequisites

- Phase 1 (services crate) completed and passing tests
- Discover main's current `AppServiceBuilder` structure (file: `crates/lib_bodhiserver/src/app_service_builder.rs`)

## Changes

### 1. Update Session Service Construction

The `get_or_build_session_service()` method in `AppServiceBuilder` currently:
- Reads `session_db_path()` from `SettingService`
- Creates a `SqlitePool` connection
- Constructs `SqliteSessionService::new(pool)`
- Runs migration

It needs to:
- Read `session_db_url()` from `SettingService` (the new method from Phase 1)
- Check if the URL starts with `postgres://` or `postgresql://`
- For PostgreSQL: construct `PgSessionStore::connect(url)`, run migration, wrap in `SessionStoreBackend::Postgres`
- For SQLite: construct `SqlitePool::connect(url)`, create `SqliteSessionStore::new(pool)`, run migration, wrap in `SessionStoreBackend::Sqlite`
- Construct `DefaultSessionService::new(backend)` for either path

### 2. Update Imports

Replace the `SqliteSessionService` import with the new types:
- `DefaultSessionService`
- `SessionStoreBackend`
- `SqliteSessionStore`
- `PgSessionStore`

Remove any direct `SqlitePool` usage for session construction if it was previously imported solely for that purpose.

## Verification

1. `cargo check -p lib_bodhiserver` -- compiles with updated builder
2. `cargo test -p lib_bodhiserver` -- existing tests pass
3. **Cumulative**: `cargo test -p services -p lib_bodhiserver` -- both crates pass
4. `cargo fmt` -- clean formatting
