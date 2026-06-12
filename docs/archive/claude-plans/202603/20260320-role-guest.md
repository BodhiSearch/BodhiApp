# Plan: Proper Role Hierarchy with Guest and Anonymous Variants

## Context

Many places in the codebase model roles as `Option<Role>` (e.g., `role: Option<ResourceRole>`), indicating incomplete domain modeling. Additionally, the codebase conflates two distinct states:
1. **Unauthenticated** — no credentials at all (`AuthContext::Anonymous`)
2. **Authenticated but no role** — user has identity (JWT) but `roles: []`

This causes bugs like `/user/request-access` returning `AlreadyHasAccess` (422) for unauthenticated users instead of a proper auth error.

**Solution**: Add two new variants to `ResourceRole` only:
- `ResourceRole::Anonymous` — unauthenticated access level
- `ResourceRole::Guest` — authenticated but no assigned role

**Hierarchy**: `Anonymous < Guest < User < PowerUser < Manager < Admin`

**Deferred**: `TokenScope` and `UserScope` stay unchanged (User, PowerUser only). Guest variants will be added when needed. `ExternalApp.role` stays `Option<UserScope>`. `UserInfo.role` stays `Option<AppRole>`.

## Design Decisions

1. **Two new ResourceRole variants**: `Anonymous` (no auth) and `Guest` (authenticated, no role)
2. **TokenScope and UserScope unchanged** — Guest deferred until needed
3. **ExternalApp.role stays `Option<UserScope>`** — no Guest variant yet
4. **No `AppRole::Anonymous`** — `AuthContext::Anonymous.app_role()` returns `Some(AppRole::Session(ResourceRole::Anonymous))`
5. **`app_role()` stays `Option<AppRole>`** — ExternalApp { role: None } still returns None
6. **`UserInfo.role` stays `Option<AppRole>`** — full non-optional deferred until UserScope gets Guest
7. **`AuthContext::Anonymous` simplified** — remove `client_id`/`tenant_id` fields (dead code), keep only `deployment`
8. **Session.role and MultiTenantSession.role become non-optional** — `Option<ResourceRole>` → `ResourceRole`
9. **Wildcard `_` patterns audited** — replace with explicit arms for defense-in-depth
10. **Dead code removed**: `included_roles()`, `included_scopes()`, `resource_role()` (on enum), `scope_token()`, `scope_user()`
11. **Middleware error for Guest role**: `Forbidden` (403) not `MissingAuth` (401) — user IS authenticated
12. **Input validation**: Reject `Anonymous`/`Guest` as targets in `ChangeRoleRequest.role` and `CreateTokenRequest.scope`
13. **`AnonymousNotAllowed` error stays** — semantically correct
14. **Serialization**: `"resource_anonymous"`, `"resource_guest"`

---

## Milestone 1: Backend

**Gate**: `make test.backend` passes

### Layer 1.1: services — ResourceRole enum changes

**File: `crates/services/src/auth/auth_objs.rs`**

Add `Anonymous` and `Guest` variants FIRST for correct `PartialOrd` derive ordering:

```rust
pub enum ResourceRole {
  #[serde(rename = "resource_anonymous")]
  #[strum(serialize = "resource_anonymous")]
  Anonymous,       // No auth
  #[serde(rename = "resource_guest")]
  #[strum(serialize = "resource_guest")]
  Guest,           // Authenticated, no assigned role
  // ... existing User, PowerUser, Manager, Admin
}
```

**AppRole, TokenScope, UserScope stay unchanged.**

Update `FromStr`: add `"resource_anonymous"` and `"resource_guest"`.

Update `ResourceRole::max_user_scope()` — explicit match arms:
```rust
pub fn max_user_scope(&self) -> UserScope {
  match self {
    ResourceRole::Anonymous | ResourceRole::Guest | ResourceRole::User => UserScope::User,
    ResourceRole::PowerUser | ResourceRole::Manager | ResourceRole::Admin => UserScope::PowerUser,
  }
}
```

**Remove dead methods**:
- `ResourceRole::included_roles()` (line 65), `ResourceRole::resource_role()` (line 60)
- `TokenScope::included_scopes()` (line 157), `TokenScope::scope_token()` (line 153)
- `UserScope::included_scopes()` (line 220), `UserScope::scope_user()` (line 216)

**Update test files**:
- `test_auth_objs_role.rs` — remove dead-method tests, add Anonymous/Guest ordering and `has_access_to` tests
- `test_auth_objs_token_scope.rs` — remove dead-method tests
- `test_auth_objs_user_scope.rs` — remove dead-method tests

### Layer 1.2: services — AuthContext changes

**File: `crates/services/src/auth/auth_context.rs`**

Simplify `AuthContext::Anonymous` — remove dead fields:
```rust
Anonymous {
  deployment: DeploymentMode,
}
```

Remove `Option` from Session/MultiTenantSession role fields (NOT ExternalApp):
- `Session.role`: `Option<ResourceRole>` → `ResourceRole`
- `MultiTenantSession.role`: `Option<ResourceRole>` → `ResourceRole`
- `ExternalApp.role`: stays `Option<UserScope>` (no change)

Update convenience methods:
- `client_id()` → Anonymous arm returns `None` (no field)
- `tenant_id()` → Anonymous arm returns `None` (no field)
- `resource_role()` → `Some(role)` for Session/MultiTenantSession (still `Option<&ResourceRole>`)
- `app_role()` → stays `Option<AppRole>`, but simplified:
  ```rust
  pub fn app_role(&self) -> Option<AppRole> {
    match self {
      AuthContext::Anonymous { .. } => Some(AppRole::Session(ResourceRole::Anonymous)),
      AuthContext::Session { role, .. } => Some(AppRole::Session(*role)),
      AuthContext::MultiTenantSession { role, .. } => Some(AppRole::Session(*role)),
      AuthContext::ApiToken { role, .. } => Some(AppRole::ApiToken(*role)),
      AuthContext::ExternalApp { role: Some(role), .. } => Some(AppRole::ExchangedToken(*role)),
      AuthContext::ExternalApp { role: None, .. } => None,
    }
  }
  ```

Update inline tests: remove `client_id`/`tenant_id` from Anonymous, `role: None` → `role: ResourceRole::Guest`, `role: Some(x)` → `role: x` (Session/MultiTenantSession only).

### Layer 1.3: services — Test utilities

**File: `crates/services/src/test_utils/auth_context.rs`**

- `test_anonymous()` → remove `client_id`/`tenant_id` fields
- Remove `test_anonymous_with_client_id()` — field no longer exists
- `test_session()`: `role: Some(role)` → `role: role`
- `test_session_no_role()` → rename/replace with `test_session(id, name, ResourceRole::Guest)`
- `test_session_with_token()`: `role: Some(role)` → `role: role`
- `test_multi_tenant_session()`: `role: None` → `role: ResourceRole::Guest`
- `test_multi_tenant_session_no_role()`: `role: None` → `role: ResourceRole::Guest`
- `test_multi_tenant_session_full()`: `role: Some(role)` → `role: role`
- `with_deployment()` → simplify Anonymous destructuring
- `with_tenant_id()` → remove Anonymous arm
- ExternalApp factories: unchanged (still use `Option<UserScope>`)

### Layer 1.4: routes_app — Middleware

**File: `crates/routes_app/src/middleware/apis/api_middleware.rs`**

Simplify Session/MultiTenantSession match arms (no more `Some(role)`/`None`):
```rust
AuthContext::Session { role, .. }
| AuthContext::MultiTenantSession { role, .. } => {
  if !role.has_access_to(&required_role) {
    return Err(ApiAuthError::Forbidden);
  }
}
// ExternalApp arms unchanged (still uses Option)
```

**File: `crates/routes_app/src/middleware/auth/auth_middleware.rs`**

- `anon()` closure: `AuthContext::Anonymous { deployment: deployment.clone() }`
- Where `role: Some(role)` → `role: role` (Session/MultiTenantSession only)
- Dashboard-only MultiTenantSession: `role: None` → `role: ResourceRole::Guest`
- Line 171: remove `.ok_or(AuthError::MissingRoles)?`

**File: `crates/routes_app/src/shared/auth_scope_extractor.rs`**

Simplify fallback: `AuthContext::Anonymous { deployment: DeploymentMode::Standalone }`

**File: `crates/routes_app/src/middleware/token_service/token_service.rs`**

- `get_valid_session_token` return type: `Result<(String, Option<ResourceRole>)>` → `Result<(String, ResourceRole)>`
- No roles in JWT → `ResourceRole::Guest`:
  ```rust
  let role = claims
    .resource_access
    .get(&instance_client_id)
    .and_then(|roles| ResourceRole::from_resource_role(&roles.roles).ok())
    .unwrap_or(ResourceRole::Guest);
  ```
- `handle_external_client_token`: unchanged (ExternalApp role stays `Option<UserScope>`)

**Files: `crates/routes_app/src/middleware/utils.rs`, `crates/routes_app/src/routes_dev.rs`**

Update Anonymous constructions.

### Layer 1.5: routes_app — Route handlers

**File: `crates/routes_app/src/users/routes_users_access_request.rs`** (users_request_access)

Fix wildcard bug:
```rust
let (user_id, username, role) = match auth_scope.auth_context() {
  AuthContext::Session { user_id, username, role, .. }
  | AuthContext::MultiTenantSession { user_id, username, role, .. }
    => (user_id, username, role),
  AuthContext::Anonymous { .. } => {
    return Err(UsersRouteError::AuthenticationRequired)?;
  }
  AuthContext::ApiToken { .. } | AuthContext::ExternalApp { .. } => {
    return Err(UsersRouteError::AlreadyHasAccess)?;
  }
};

if role.has_access_to(&ResourceRole::User) {
  return Err(UsersRouteError::AlreadyHasAccess)?;
}
```

**File: `crates/routes_app/src/users/error.rs`**

Add `AuthenticationRequired` variant:
```rust
#[error("Authentication required.")]
#[error_meta(error_type = ErrorType::Authentication)]
AuthenticationRequired,
```

**File: `crates/routes_app/src/tokens/routes_tokens.rs`** (tokens_create)

Defense-in-depth for Guest/Anonymous:
```rust
if !user_role.has_access_to(&ResourceRole::User) {
  return Err(TokenRouteError::AccessTokenMissing.into());
}
```

**File: `crates/routes_app/src/users/routes_users.rs`** (users_change_role)

Reject Anonymous/Guest as caller and target:
```rust
if !caller_role.has_access_to(&ResourceRole::User) || !caller_role.has_access_to(&request.role) {
  return Err(UsersRouteError::InsufficientPrivileges.into());
}
if !request.role.has_access_to(&ResourceRole::User) {
  return Err(UsersRouteError::InsufficientPrivileges.into());
}
```

**File: `crates/routes_app/src/users/routes_users_access_request.rs`** (approve handler)

Fix wildcard — add explicit Anonymous arm.

**File: `crates/routes_app/src/apps/routes_apps.rs`** (approve handler)

Fix wildcard at line 302.

**File: `crates/routes_app/src/users/routes_users_info.rs`** (users_info)

Session/MultiTenantSession: `role.map(AppRole::Session)` → `Some(AppRole::Session(*role))`
ExternalApp: unchanged (still uses `role.as_ref().map(...)`)

**File: `crates/routes_app/src/setup/routes_setup.rs`**

Update Anonymous match arms.

### Layer 1.6: Backend tests

Update all test files that construct `AuthContext::Anonymous` or use `role: None` / `role: Some(...)` for Session/MultiTenantSession:

- `crates/services/src/auth/auth_context.rs` (inline tests)
- `crates/routes_app/src/middleware/apis/test_api_middleware.rs`
- `crates/routes_app/src/middleware/access_requests/test_access_request_middleware.rs`
- `crates/routes_app/src/users/test_user_info.rs`
- `crates/routes_app/src/setup/test_setup.rs`
- `crates/routes_app/src/auth/test_login_initiate.rs`
- `crates/routes_app/src/tenants/test_dashboard_auth.rs`
- `crates/routes_app/src/tenants/test_tenants.rs`
- `crates/routes_app/src/tokens/test_tokens_crud.rs`
- `crates/routes_app/tests/test_live_multi_tenant.rs`
- `crates/server_app/tests/test_live_multi_tenant.rs`

### Gate: `make test.backend`

---

## Milestone 2: UI

**Gate**: `cd crates/bodhi && npm run build && npm run test:all`

### Layer 2.1: Regenerate TypeScript types

```bash
cargo run --package xtask openapi
make build.ts-client
```

### Layer 2.2: Frontend code changes

**File: `crates/bodhi/src/lib/roles.ts`**

```typescript
export type Role = 'resource_anonymous' | 'resource_guest' | 'resource_user' | 'resource_power_user' | 'resource_manager' | 'resource_admin';

export const roleHierarchy: Record<Role, number> = {
  resource_anonymous: 0,
  resource_guest: 1,
  resource_user: 2,
  resource_power_user: 3,
  resource_manager: 4,
  resource_admin: 5,
};

// ROLE_OPTIONS: do NOT include resource_anonymous or resource_guest (not assignable)
```

**File: `crates/bodhi/src/components/AppInitializer.tsx`**

```typescript
// Before: if (!userInfo.role)
// After:
if (!userInfo.role || userInfo.role === 'resource_guest' || userInfo.role === 'resource_anonymous') {
  router.push(ROUTE_REQUEST_ACCESS);
  return;
}
```

**Other frontend files**:
- `src/test-fixtures/access-requests.ts` — update `createMockUserInfo`
- `src/test-utils/msw-v2/handlers/user.ts` — default role handler
- `src/components/users/UserRow.tsx` — role handling
- `src/app/ui/toolsets/page.tsx`, `src/app/ui/mcps/servers/page.tsx` — role checks
- `src/app/ui/request-access/page.tsx` — guest/anonymous check
- `src/app/ui/users/pending/page.tsx`, `src/app/ui/users/page.tsx` — role guards
- `src/components/AppInitializer.test.tsx` — test assertions

### Gate: `cd crates/bodhi && npm run build && npm run test:all`

---

## Milestone 3: E2E

**Gate**: E2E tests pass

### Layer 3.1: Rebuild UI for E2E

```bash
make build.ui-rebuild
```

### Layer 3.2: Update E2E tests

Update Playwright tests in `crates/lib_bodhiserver_napi/tests-js/` if any test scenarios involve role-related assertions or mock data.

### Gate: `make test.napi`

---

## Critical Files

| File | Changes |
|------|---------|
| `crates/services/src/auth/auth_objs.rs` | Add Anonymous+Guest to ResourceRole, remove dead methods |
| `crates/services/src/auth/auth_context.rs` | Simplify Anonymous, remove Option from Session/MTS role |
| `crates/services/src/test_utils/auth_context.rs` | Update factory methods |
| `crates/routes_app/src/middleware/apis/api_middleware.rs` | Simplify match arms |
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | Simplify anon(), remove .ok_or() |
| `crates/routes_app/src/shared/auth_scope_extractor.rs` | Simplify fallback |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | Return ResourceRole instead of Option |
| `crates/routes_app/src/users/routes_users_access_request.rs` | Fix wildcard bug |
| `crates/routes_app/src/users/error.rs` | Add AuthenticationRequired |
| `crates/routes_app/src/tokens/routes_tokens.rs` | Audit wildcards |
| `crates/routes_app/src/users/routes_users.rs` | Audit wildcards |
| `crates/routes_app/src/users/routes_users_info.rs` | Simplify role mapping |
| `crates/routes_app/src/apps/routes_apps.rs` | Fix wildcard |
| `crates/routes_app/src/setup/routes_setup.rs` | Update Anonymous arms |
| `crates/bodhi/src/lib/roles.ts` | Add resource_anonymous + resource_guest |
| `crates/bodhi/src/components/AppInitializer.tsx` | Update role checks |
| `openapi.json` | Regenerated |
| `ts-client/src/types/types.gen.ts` | Regenerated |
