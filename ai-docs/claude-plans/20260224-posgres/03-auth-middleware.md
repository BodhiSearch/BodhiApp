# Phase 3: auth_middleware crate -- Import Updates

## Functional Goal

Update the auth_middleware crate to use the renamed `DefaultSessionService` type and access session store through the `SessionService` trait method instead of direct field access.

## Prerequisites

- Phase 1 (services) and Phase 2 (lib_bodhiserver) completed
- Discover main's current auth_middleware test structure

## Changes

### 1. Update Imports

In `crates/auth_middleware/src/auth_middleware.rs` (and its test module):
- Replace `SqliteSessionService` import with `DefaultSessionService`
- Add `SessionService` trait import (needed for `get_session_store()` method)

### 2. Update Test Session Construction

The auth_middleware tests construct session services directly for test setup. All places that use `SqliteSessionService::build_session_service(dbfile)` need to use `DefaultSessionService::build_session_service(dbfile)` instead.

In the worktree, approximately 6 test functions in `auth_middleware.rs` were affected. The implementer should search for all `SqliteSessionService` references within this crate.

### 3. Update Session Store Access Pattern

Tests that access the session store for creating/loading records need to change from direct field access to method access:

- `session_service.session_store.create(&mut record)` becomes `session_service.get_session_store().create(&mut record)`
- `session_service.session_store.load(&id)` becomes `session_service.get_session_store().load(&id)`

This change is necessary because `DefaultSessionService` exposes the store through a trait method rather than a public field, enabling backend-agnostic access.

### 4. Formatting-Only Changes

The worktree diff also contained formatting adjustments (line wrapping for long function arguments) in `auth_middleware.rs`, `resource_scope.rs`, and `toolset_auth_middleware.rs`. These are `cargo fmt` artifacts and will happen naturally when running `cargo fmt` after the substantive changes.

## Verification

1. `cargo check -p auth_middleware` -- compiles with updated imports
2. `cargo test -p auth_middleware` -- all auth middleware tests pass
3. **Cumulative**: `cargo test -p services -p lib_bodhiserver -p auth_middleware`
4. `cargo fmt` -- clean formatting
