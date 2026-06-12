# Phase 5: server_app crate -- Live Server Test Utility Updates

## Functional Goal

Update the server_app crate's live server test utilities to construct session services using the new backend-agnostic types. This crate contains integration tests that spin up real HTTP servers with fully-wired services.

## Prerequisites

- Phases 1-4 completed
- Discover main's current `crates/server_app/tests/utils/` structure

## Changes

### 1. Update Imports

In `crates/server_app/tests/utils/live_server_utils.rs` (or wherever the live server test utilities live on main):

- Replace `SqliteSessionService` import with `DefaultSessionService`
- Add imports for `SessionStoreBackend` and `SqliteSessionStore`
- Remove `SqliteSessionService` from the imports list

### 2. Update Session Service Construction

The live server test utilities construct a full `AppService` including session service for integration testing. The pattern changes from:

**Before**:
1. Create `SqlitePool` from session DB path
2. Construct `SqliteSessionService::new(pool)`
3. Run `session_service.migrate()`

**After**:
1. Create `SqlitePool` from session DB path
2. Construct `SqliteSessionStore::new(pool)`
3. Run `store.migrate()`
4. Wrap in `DefaultSessionService::new(SessionStoreBackend::Sqlite(store))`

In the worktree, this pattern appeared in two functions within `live_server_utils.rs`:
- `setup_minimal_app_service()` -- minimal service for basic integration tests
- `setup_test_app_service()` -- full service for comprehensive integration tests

The implementer should search for all `SqliteSessionService` references in the `server_app` crate.

### 3. Module Ordering

The worktree also had a minor change in `crates/server_app/tests/utils/mod.rs` -- reordering module declarations and adding a trailing newline. This is a formatting artifact and will happen naturally with `cargo fmt`.

## Verification

1. `cargo check -p server_app` -- compiles with updated utilities
2. `cargo test -p server_app` -- all server_app tests pass (these are slower integration tests)
3. **Cumulative**: `cargo test -p services -p lib_bodhiserver -p auth_middleware -p routes_app -p server_app`
4. **Full backend**: `make test.backend` -- complete backend test suite passes
5. `cargo fmt` -- clean formatting
