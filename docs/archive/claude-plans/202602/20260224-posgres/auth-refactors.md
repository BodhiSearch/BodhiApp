# Auth Refactors Archive (Not Part of Session PG Flow)

This document archives two auth-related refactors that were implemented in the `multi-tenant` worktree as multi-tenancy prerequisites. They are **not required** for the session PostgreSQL migration but represent useful architectural improvements that could be done independently.

## 1. Make ExternalApp Role Optional

### Problem

When an external app token lacks `scope_user_*` claims, the token exchange flow rejects the request entirely (via `TokenError::ScopeEmpty`), falling back to `AuthContext::Anonymous`. This loses all identity information -- the user's ID, app client ID, and access request ID are all discarded.

### Desired Behavior

Preserve the full `AuthContext::ExternalApp` with `role: None` when user scope claims are absent. This allows:
- `/bodhi/v1/user` to show "request access" UI (identity known, role not yet assigned)
- Downstream middleware to handle authorization decisions
- Consistent pattern with `AuthContext::Session { role: None }` which already exists

### What Was Changed (in worktree)

- `AuthContext::ExternalApp { role: UserScope }` changed to `role: Option<UserScope>`
- `app_role()` method returns `None` when `role: None`
- `ScopeEmpty` guard removed from token exchange path
- `UserScope::from_scope()` called with `.ok()` instead of `?` (converts to `Option`)
- `api_auth_middleware` split ExternalApp match into `role: Some(role)` and `role: None` arms
- Added `test_external_app_no_role()` test factory

### Affected Crates

`objs`, `auth_middleware`, `routes_app`

### Full Details

See `ai-docs/claude-plans/20260210-access-request/20260217-scope-role-optional.md` for the complete plan with implementation summary.

---

## 2. Move ResourceScope from objs to auth_middleware

### Problem

`ResourceScope` and `ResourceScopeError` are only used within the `auth_middleware` crate, but they live in the `objs` crate. This creates unnecessary cross-crate coupling.

### Desired Behavior

Move `ResourceScope` to its only consumer (`auth_middleware`), reducing the public API surface of `objs`.

### What Was Changed (in worktree)

- Created `crates/auth_middleware/src/resource_scope.rs` (moved from `crates/objs/src/resource_scope.rs`)
- Changed `ResourceScope::User(UserScope)` to `ResourceScope::User(Option<UserScope>)` (combined with refactor #1)
- Deleted `crates/objs/src/resource_scope.rs`
- Updated imports in `token_service.rs`, `auth_middleware.rs`, `api_auth_middleware.rs`

### Additional: Test Utility Reorganization

Test factories were extracted from `auth_context.rs` inline `#[cfg(feature = "test-utils")]` blocks into `crates/auth_middleware/src/test_utils/auth_context.rs`, following project convention.

### Affected Crates

`objs`, `auth_middleware`, `routes_app` (6 test files updated for import changes)

---

## 3. Store External App Token

### Problem

When processing an external app's bearer token, the original token value was not preserved in `AuthContext`. Downstream handlers that need to forward the token (e.g., for proxying to upstream services) had no access to it.

### Desired Behavior

Store the original bearer token in `AuthContext::ExternalApp` as `external_app_token: Option<String>` so it can be forwarded by downstream handlers.

### What Was Changed (in worktree)

- Added `external_app_token: Option<String>` field to `AuthContext::ExternalApp`
- `auth_middleware` stores the bearer token when `ResourceScope::User(_)` is matched
- `optional_auth_middleware` follows the same pattern

### Affected Crates

`auth_middleware`

---

## Implementation Note

These refactors were done as commits `13e83c10` and `b545d427` in the `multi-tenant` worktree. If re-implementing on main, check whether:

1. Main already has equivalent changes (due to parallel development)
2. The types/fields referenced still exist in their current form
3. The `ResourceScope` type still lives in `objs` on main

Each refactor stands alone and can be implemented independently of the session PG migration.
