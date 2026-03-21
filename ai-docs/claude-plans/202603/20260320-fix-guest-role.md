# Plan: Post-Refactor Cleanup — Guest Endpoints, Type Dedup, Input Validation

## Context

After adding `ResourceRole::Anonymous` and `ResourceRole::Guest` variants, three follow-up issues remain:

1. **Frontend type duplication**: `roles.ts` manually defines `Role` type identical to `ResourceRole` from `@bodhiapp/ts-client`
2. **Route group misplacement**: `users_request_access` and `users_request_status` sit in `optional_auth` (no auth required), but request-access now correctly requires authentication (Guest minimum). These should be behind `auth_middleware` + `api_auth_middleware(Guest)`
3. **Input validation gap**: `ChangeRoleRequest.role` and `ApproveUserAccessRequest.role` accept `ResourceRole::Anonymous`/`Guest` via deserialization — admins could assign Guest/Anonymous as a role
4. **Tech debt**: Tenant endpoints (`tenants_index`, `tenants_create`, `tenants_activate`) lack proper multi-tenant authorization middleware

## Changes

### 1. Frontend: Re-export `Role` from ts-client

**File: `crates/bodhi/src/lib/roles.ts`**

- Remove manual `Role` type definition (lines 7-13)
- Add: `export type { ResourceRole as Role } from '@bodhiapp/ts-client';`
- Keep `roleHierarchy`, `ROLE_OPTIONS`, and all helper functions unchanged
- No other frontend files change — they all import `Role` from `@/lib/roles`

### 2. Backend: New `guest_endpoints` route group

**File: `crates/routes_app/src/routes.rs`**

Remove from `optional_auth` (lines 116-117):
```
.route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
.route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
```

Add new route group after `optional_auth` setup:
```rust
let guest_endpoints = Router::new()
    .route(ENDPOINT_USER_REQUEST_ACCESS, post(users_request_access))
    .route(ENDPOINT_USER_REQUEST_STATUS, get(users_request_status))
    .route_layer(from_fn_with_state(
      state.clone(),
      move |state, req, next| {
        api_auth_middleware(ResourceRole::Guest, None, None, state, req, next)
      },
    ))
    .route_layer(from_fn_with_state(state.clone(), auth_middleware));
```

Merge into `session_protected` (line 535):
```rust
let session_protected = Router::new()
    .merge(guest_endpoints)
    .merge(user_session_apis)
    // ... rest unchanged
```

**File: `crates/routes_app/src/users/routes_users_access_request.rs`**

Simplify `users_request_access` handler — remove explicit `Anonymous` match arm and `AuthenticationRequired` error (middleware now handles this):
```rust
let (user_id, username, role) = match auth_scope.auth_context() {
    AuthContext::Session { user_id, username, role, .. }
    | AuthContext::MultiTenantSession { user_id, username, role, .. }
      => (user_id, username, role),
    AuthContext::Anonymous { .. }
    | AuthContext::ApiToken { .. }
    | AuthContext::ExternalApp { .. } => {
      return Err(UsersRouteError::AlreadyHasAccess)?;
    }
};
```

**File: `crates/routes_app/src/users/error.rs`**

Remove `AuthenticationRequired` variant (middleware handles 401 now).

### 3. Backend: Input validation for role assignment

**File: `crates/routes_app/src/users/routes_users.rs` (users_change_role)**

Add after `caller_role` extraction (before existing `has_access_to` check):
```rust
// Reject Anonymous/Guest as assignment targets
if !request.role.has_access_to(&ResourceRole::User) {
    return Err(UsersRouteError::InsufficientPrivileges)?;
}
```

**File: `crates/routes_app/src/users/routes_users_access_request.rs` (users_access_request_approve)**

Add after approver role extraction (before existing hierarchy check):
```rust
// Reject Anonymous/Guest as assignment targets
if !request.role.has_access_to(&ResourceRole::User) {
    return Err(UsersRouteError::InsufficientPrivileges)?;
}
```

### 4. Backend: Unit tests

**File: `crates/routes_app/src/users/test_access_request_user.rs`** (or new sibling test file)

- Test `users_request_access` with `AuthContext::test_session("u1", "user", ResourceRole::Guest)` → 201 (handler proceeds to create request)
- Test `users_request_status` with `AuthContext::test_session("u1", "user", ResourceRole::Guest)` → 404 (PendingRequestNotFound, not 401)

**File: `crates/routes_app/src/users/test_management_crud.rs`**

- Test `users_change_role` with `request.role = ResourceRole::Guest` → 400 InsufficientPrivileges
- Test `users_change_role` with `request.role = ResourceRole::Anonymous` → 400 InsufficientPrivileges

**File: `crates/routes_app/src/users/test_access_request_admin.rs`**

- Test `users_access_request_approve` with `request.role = ResourceRole::Guest` → 400 InsufficientPrivileges

### 5. TECHDEBT.md entry

**File: `crates/routes_app/TECHDEBT.md`**

Add:
```markdown
## Multi-tenant endpoint authorization

- **Currently**: `tenants_index`, `tenants_create`, and `tenants_activate` are in `optional_auth` with no role enforcement
- **Should be**: Behind middleware requiring multi-tenant deployment mode + dashboard session (at minimum Guest role)
- **Why deferred**: Requires new middleware for deployment mode check + dashboard token validation
```

## Critical Files

| File | Change |
|------|--------|
| `crates/bodhi/src/lib/roles.ts` | Re-export `Role` from ts-client |
| `crates/routes_app/src/routes.rs` | New `guest_endpoints` group |
| `crates/routes_app/src/users/routes_users_access_request.rs` | Simplify handler, add input validation |
| `crates/routes_app/src/users/routes_users.rs` | Add input validation |
| `crates/routes_app/src/users/error.rs` | Remove `AuthenticationRequired` |
| `crates/routes_app/TECHDEBT.md` | Add tenant endpoints entry |

## Verification

1. `cargo test -p routes_app --lib` — all tests pass
2. `cd crates/bodhi && npm run build && npm run test:all` — frontend build + tests pass
3. `cargo test` — full backend passes
