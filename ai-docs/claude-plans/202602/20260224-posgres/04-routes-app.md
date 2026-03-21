# Phase 4: routes_app crate -- Import and Test Utility Updates

## Functional Goal

Update the routes_app crate to use the renamed `DefaultSessionService` type and the updated test utility builder methods from Phase 1.

## Prerequisites

- Phases 1-3 completed
- Discover main's current routes_app test structure (especially login tests and access request tests)

## Changes

### 1. Update Test Imports

In routes_app test files, replace `SqliteSessionService` imports with `DefaultSessionService`. Also add `SessionService` trait import where `get_session_store()` is called.

In the worktree, the affected test files were:
- `crates/routes_app/src/routes_auth/tests/login_test.rs`
- `crates/routes_app/src/routes_users/tests/access_request_test.rs`

The implementer should search for all `SqliteSessionService` references within this crate to find the current set.

### 2. Update Session Service Construction in Tests

All test code that constructs `SqliteSessionService` directly needs to use the new pattern:

- `SqliteSessionService::build_session_service(dbfile)` becomes `DefaultSessionService::build_session_service(dbfile)`

### 3. Update Builder Method Calls

Tests that use `AppServiceStubBuilder` to inject session services:

- `.with_sqlite_session_service(session_service)` becomes `.with_default_session_service(session_service)`

This matches the builder method rename done in Phase 1's test utility updates.

### 4. Update Session Store Access in Tests

Tests that directly access the session store for setup or assertions:

- `session_service.session_store.create(&mut record)` becomes `session_service.get_session_store().create(&mut record)`
- `SessionStore::save(&session_service.session_store, &record)` becomes `SessionStore::save(session_service.get_session_store(), &record)`
- `session_service.session_store.count_sessions_for_user(user_id)` becomes `session_service.get_session_store().count_sessions_for_user(user_id)`

### 5. Login Test Details

The login tests (`login_test.rs`) are the most heavily affected in this crate. They:
- Build session services for session-aware auth flow testing
- Create session records with tokens directly via the store
- Assert session state after auth callback and logout flows
- Use `set_token_in_session()` helper that accesses the store directly

All of these need the import rename and field-to-method access pattern change.

### 6. Access Request Test Details

The access request tests (`access_request_test.rs`) test that approving an access request clears the user's sessions. They:
- Construct a `DefaultSessionService` with a temporary SQLite database
- Create multiple sessions for a user via `SessionStore::save`
- Verify session count before and after approval
- Use `.with_default_session_service()` on the builder

## Verification

1. `cargo check -p routes_app` -- compiles with updated imports
2. `cargo test -p routes_app` -- all route tests pass
3. **Cumulative**: `cargo test -p services -p lib_bodhiserver -p auth_middleware -p routes_app`
4. `cargo fmt` -- clean formatting
