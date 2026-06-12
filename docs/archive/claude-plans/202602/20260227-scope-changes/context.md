# KC Scope Removal — Context

> **Scope**: This document covers the removal of Keycloak-managed scopes from BodhiApp's external app authorization flow, replacing JWT-scope-based role derivation with DB-based `approved_role` on access request records. It spans all 8 layers of the codebase: `objs`, `services`, `auth_middleware`, `routes_app`, `server_app`, frontend, E2E tests, and documentation.

## File Loading Guide

No files are mandatory — this document is self-contained.

**Tier 1 — Recommended for related implementation work:**
- `crates/auth_middleware/CLAUDE.md` — AuthContext enum, token service, role derivation
- `crates/routes_app/CLAUDE.md` — App access request handlers, privilege escalation checks
- `crates/services/CLAUDE.md` — AccessRequestService, DB schema, AuthService changes

**Tier 2 — Plan and review history:**
- `20260227-kc_scope_removal_plan_4be81c98.plan.md` — Original 8-layer implementation plan
- `20260227-scope-change-reviews.md` — Post-implementation review findings and fix plan
- `20260226-role-dropdown.md` — Role dropdown + privilege escalation guard plan
- `reviews/index.md` — Code review index (19 findings, 0 critical)

---

## Problem Statement

BodhiApp's external app authorization previously relied on Keycloak-managed scopes for role derivation:

1. **`scope_resource_*` scopes**: Per-resource scopes loaded by Keycloak at token exchange time. Performance degraded at O(N) with the number of resources. Keycloak's `keycloak-bodhi-ext` extension removed these entirely.
2. **`scope_user_*` / `scope_token_*` scopes with Admin/Manager tiers**: Redundant with `resource_access` JWT roles already present in tokens. The four-tier hierarchy (User, PowerUser, Manager, Admin) for `UserScope` and `TokenScope` was unnecessary — only User and PowerUser tiers are meaningful for external apps and API tokens.
3. **Auto-approve flow**: When an external app requested access without specifying tools/MCPs, BodhiApp auto-approved by calling `register_resource_access()` on Keycloak's server-to-server endpoint. This bypassed user review entirely.

The net result was that external app roles were derived by parsing `scope_user_*` strings from JWT scope claims via `UserScope::from_scope()`, and the `ResourceScope` enum bridged between `TokenScope` and `UserScope` at the middleware layer. This was fragile, KC-dependent, and gave users no control over what role an external app received.

**No backwards compatibility required** — no production deployment exists.

---

## Design Decisions

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | External app role source | DB `approved_role` on access request record | Decouples from KC scope parsing; user explicitly grants role |
| 2 | UserScope variants | Remove `Admin`, `Manager`; keep `User`, `PowerUser` | Only two tiers meaningful for external apps |
| 3 | TokenScope variants | Remove `Admin`, `Manager`; keep `User`, `PowerUser` | Only two tiers meaningful for API tokens |
| 4 | ResourceScope enum | Remove entirely | No longer needed — role derivation moved to DB |
| 5 | `UserScope::from_scope()` | Remove | No longer parsing role from JWT scope strings |
| 6 | `TokenScope::from_scope()` | Remove | No production callers after scope simplification |
| 7 | Auto-approve flow | Remove entirely | All access requests go through user review (draft-first) |
| 8 | `AuthService::register_resource_access()` | Remove | Only used by the removed auto-approve flow |
| 9 | DB `resource_scope` column | Remove from schema | Replaced by `requested_role` / `approved_role` |
| 10 | DB `AppInstance.scope` column | Remove from schema | KC no longer returns scope for resource clients |
| 11 | Access request role columns | `requested_role TEXT NOT NULL` + `approved_role TEXT` (nullable) | Separates what the app asked for from what the user granted |
| 12 | `AuthContext::ExternalApp.scope` field | Rename to `role: Option<UserScope>` | Reflects DB-derived role, `None` when no approved access request |
| 13 | Token exchange scopes | Only `scope_access_request:*` + standard OIDC scopes; no `scope_user_*` | KC no longer processes user-tier scopes |
| 14 | Privilege escalation guards | Two-layer: routes_app handler check + auth_middleware token exchange check | Defense-in-depth at both API entry and token validation |
| 15 | `CreateAccessRequestResponse` | Single struct (not tagged enum) with `status: "draft"` | No `Approved` variant needed after auto-approve removal |
| 16 | `ApproveAccessRequestBody.approved_role` | Typed as `UserScope` (not `String`) | Serde-level validation; no manual `.parse()` needed |

---

## Before/After Architecture

### External App Role Derivation — Before

```
External App Token
  → auth_middleware strips "Bearer " prefix
  → token_service.validate_bearer_token()
      → handle_external_client_token()
          → extract scope_user_* + scope_access_request:* from JWT claims
          → include scope_user_* in exchange scope list
          → exchange token with KC
          → parse role from exchanged token via UserScope::from_scope(scope_claims.scope)
          → return (access_token, ResourceScope::User(user_scope), app_client_id)
  → build_auth_context_from_bearer() matches ResourceScope::User(role)
  → constructs AuthContext::ExternalApp { scope: role, ... }
```

### External App Role Derivation — After

```
External App Token
  → auth_middleware strips "Bearer " prefix
  → token_service.validate_bearer_token()
      → handle_external_client_token()
          → extract ONLY scope_access_request:* from JWT claims
          → look up access request record in DB by access_request_scope
          → validate: status=approved, azp match, user_id match
          → exchange token with KC (only scope_access_request:* + OIDC standard scopes)
          → post-exchange: verify access_request_id claim matches DB record
          → derive role from DB record's approved_role column
          → verify role doesn't exceed user's resource_role (privilege escalation guard)
          → return AuthContext::ExternalApp { role: Some(approved_role), ... }
  → middleware inserts AuthContext directly into request extensions
```

Key differences:
- `scope_user_*` scopes are **not** included in exchange scope list
- Role derived from **DB `approved_role`**, not JWT scope parsing
- `ResourceScope` enum eliminated — `validate_bearer_token()` returns `AuthContext` directly
- `build_auth_context_from_bearer()` helper function eliminated
- Privilege escalation check added at token exchange time

---

## Data Model Changes

### `app_access_requests` Table (migration 0009)

| Column | Before | After |
|--------|--------|-------|
| `resource_scope` | `TEXT` (KC-returned `scope_resource-xyz`) | **Removed** |
| `requested_role` | -- | `TEXT NOT NULL` (e.g. `scope_user_user`) |
| `approved_role` | -- | `TEXT` nullable (set on approval, e.g. `scope_user_user`) |
| `access_request_scope` | `TEXT` (nullable for auto-approve) | `TEXT` (nullable, set on user approval) |

### `apps` Table (migration 0014)

| Column | Before | After |
|--------|--------|-------|
| `scope` | `TEXT NOT NULL` (KC scope string) | **Removed** |

### `AppAccessRequestRow` Struct

```rust
// Before:
pub struct AppAccessRequestRow {
  // ...
  pub resource_scope: Option<String>,
  pub access_request_scope: Option<String>,
  // ...
}

// After:
pub struct AppAccessRequestRow {
  // ...
  pub requested_role: String,
  pub approved_role: Option<String>,
  pub access_request_scope: Option<String>,
  // ...
}
```

### `AppInstance` / `AppInstanceRow` Structs

The `scope` field was removed from both structs. `create_instance()` no longer accepts a `scope` parameter.

---

## Access Request Workflow

All access requests now follow a single draft-first flow (no auto-approve):

```
1. External app → POST /bodhi/v1/apps/request-access
   Body: { app_client_id, flow_type, requested_role: "scope_user_user", requested: {...} }
   Response: { id, status: "draft", review_url }

2. External app polls → GET /bodhi/v1/apps/access-requests/{id}?app_client_id=xxx
   Response: { id, status, requested_role, approved_role, access_request_scope }

3. User reviews → GET /bodhi/v1/access-requests/{id}/review  (session auth)
   Response: { id, app_client_id, requested_role, status, requested, tools_info, mcps_info }

4a. User approves → PUT /bodhi/v1/access-requests/{id}/approve  (session auth)
    Body: { approved_role: "scope_user_user", approved: { toolsets: [...], mcps: [...] } }
    → validates privilege escalation (approved <= requested, approved <= approver's max)
    → registers consent with KC
    → updates DB: status=approved, approved_role, access_request_scope

4b. User denies → POST /bodhi/v1/access-requests/{id}/deny  (session auth)
    → updates DB: status=denied
```

### What Changed from Before

- **No auto-approve branch**: Previously, `create_draft()` checked `is_auto_approve` (true when no tools/MCPs requested) and called `register_resource_access()` to auto-approve. Now all requests create a draft.
- **`requested_role` parameter**: `create_draft()` accepts `requested_role` from the external app.
- **`approved_role` parameter**: `approve_request()` accepts `approved_role` from the reviewing user.
- **`DefaultAccessRequestService` no longer depends on `AppInstanceService`**: The auto-approve flow was the only consumer.
- **`RegisterResourceAccessResponse` removed**: Only used by auto-approve.

---

## Token Exchange Flow

`handle_external_client_token()` in `DefaultTokenService` performs the following steps:

### Pre-Exchange Validation

1. Look up `AppInstance` (client credentials) from DB
2. Extract claims from external token: issuer, azp, audience, scope
3. Validate issuer matches configured auth issuer
4. Validate audience matches our client_id
5. **Filter scopes**: extract only `scope_access_request:*` prefixed scopes (no `scope_user_*`)
6. If `scope_access_request:*` found, validate against DB:
   - Look up access request record by scope
   - Verify `status == "approved"`
   - Verify `app_client_id` matches token's `azp` claim
   - Verify `user_id` matches token's `sub` claim

### Token Exchange

7. Build exchange scope list: `scope_access_request:*` + standard OIDC scopes (`openid`, `email`, `profile`, `roles`) — only if present in original token
8. Call `auth_service.exchange_app_token()` with KC

### Post-Exchange Validation

9. Extract claims from exchanged token
10. If a validated access request record exists, verify `access_request_id` claim matches the record's primary key
11. **Derive role**: Parse `approved_role` from the validated DB record as `UserScope`

### Privilege Escalation Check (Token Service Layer)

12. Extract user's `resource_role` from exchanged token's `resource_access` claim
13. Compute `max_scope = resource_role.max_user_scope()`
14. If `approved_role > max_scope`, reject with `AccessRequestValidationError::PrivilegeEscalation`

### Build Result

15. Construct `AuthContext::ExternalApp` with `role: Some(approved_role)` (or `None` if no access request)
16. Construct `CachedExchangeResult` with `role` and `access_request_id` for caching
17. Cache the result keyed by token digest

---

## Privilege Escalation Guards

Privilege escalation is prevented at two layers:

### Layer 1: Route Handler (`routes_app` — `approve_access_request_handler`)

When a user approves an access request, the handler validates:

1. **Approver has a session role**: `AuthContext::Session { role: Some(role), .. }` required
2. **Compute max grantable**: `ResourceRole::PowerUser+ → UserScope::PowerUser`, `ResourceRole::User → UserScope::User`
3. **Check #1**: `approved_role > requested_role` → 403 `PrivilegeEscalation`
4. **Check #2**: `approved_role > max_grantable` → 403 `PrivilegeEscalation`

This prevents a `resource_user` from granting `scope_user_power_user`, and prevents granting more than what was requested.

### Layer 2: Token Exchange (`auth_middleware` — `handle_external_client_token`)

At runtime, when an external app presents a token for exchange:

1. Read `approved_role` from the access request DB record
2. Extract the user's `ResourceRole` from the exchanged token's `resource_access` claim
3. Compute `max_scope = resource_role.max_user_scope()`
4. If `approved_role > max_scope`, reject with `PrivilegeEscalation`

This catches cases where the DB record was tampered with or the user's role was downgraded after approval.

### UI Guard (`computeRoleOptions` in review page)

The frontend review page computes available role options based on both the `requested_role` and the approver's `ResourceRole`:

```typescript
function computeRoleOptions(requestedRole, userRole) {
  // Available: all scopes at or below min(requestedScope, maxGrantable)
  // resource_power_user/manager/admin → can grant scope_user_power_user
  // resource_user → can only grant scope_user_user
}
```

The dropdown only shows roles the approver is allowed to grant, preventing the user from even attempting an escalation.

---

## UI Changes

### Review Page (`crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`)

**Added:**
- **Role dropdown**: `<Select>` component showing available `approved_role` options based on `computeRoleOptions()`
- **`useUser()` hook**: Fetches current user's role for computing grantable options
- **`approvedRole` state**: Tracks selected role, initialized to highest available option
- **`canApprove` guard**: Requires `approvedRole !== null` in addition to tool/MCP validation
- **`handleApprove` body**: Includes `approved_role` in the `ApproveAccessRequestBody`

**Removed:**
- No `ScopeDisplay` component removal in this page (that was in the test-oauth-app)

### Test OAuth App (`crates/lib_bodhiserver_napi/test-oauth-app/`)

**Removed:**
- `ScopeDisplay.tsx` component (displayed KC-returned scope — no longer exists)
- `resource_scope` references from `ConfigForm.tsx`
- Scope-related fields from API responses

**Changed:**
- `ConfigForm.tsx`: Hardcoded scopes removed; `requested_role` field added
- `AccessCallbackPage.tsx`: Simplified to not display resource_scope

### E2E Page Objects

**Added:**
- `AccessRequestReviewPage.mjs`: Page object for the review page with `selectApprovedRole()` and `approveWithRole()` methods

**Changed:**
- `ConfigSection.mjs`: Removed `getResourceScope()` dead method

---

## Type Changes Summary

### `objs` Crate

| Type | Change |
|------|--------|
| `UserScope` | Removed `Admin`, `Manager` variants; kept `User`, `PowerUser` |
| `UserScope::from_scope()` | Removed (parsed role from JWT scope string) |
| `UserScopeError::MissingUserScope` | Removed (dead after `from_scope()` removal) |
| `TokenScope` | Removed `Admin`, `Manager` variants; kept `User`, `PowerUser` |
| `TokenScope::from_scope()` | Removed |
| `TokenScopeError::MissingOfflineAccess` | Removed |
| `TokenScopeError::MissingTokenScope` | Removed |
| `ResourceRole::max_user_scope()` | Added — returns max `UserScope` grantable by this role |

### `services` Crate

| Type / Method | Change |
|---------------|--------|
| `AppAccessRequestRow.resource_scope` | Removed |
| `AppAccessRequestRow.requested_role` | Added (`String`) |
| `AppAccessRequestRow.approved_role` | Added (`Option<String>`) |
| `AppInstanceRow.scope` / `AppInstance.scope` | Removed |
| `AccessRequestService::create_draft()` | Added `requested_role` parameter |
| `AccessRequestService::approve_request()` | Added `approved_role` parameter |
| `AccessRequestRepository::update_approval()` | Changed: `resource_scope` param → `approved_role` param; `access_request_scope` changed from `Option<String>` to `&str` |
| `AuthService::register_resource_access()` | Removed entirely |
| `RegisterResourceAccessResponse` | Removed |
| `RegisterAccessRequestConsentResponse.scope` | Removed |
| `ClientRegistrationResponse.scope` | Removed |
| `DefaultAccessRequestService` constructor | No longer takes `AppInstanceService` |
| Dead types in `services/src/objs.rs` | `AppAccessRequest`, `AppAccessResponse`, `AppAccessRequestDetail` removed |

### `auth_middleware` Crate

| Type / Method | Change |
|---------------|--------|
| `ResourceScope` enum | Removed entirely (file deleted) |
| `ResourceScopeError` | Removed |
| `AuthContext::ExternalApp.scope` | Renamed to `role: Option<UserScope>` |
| `CachedExchangeResult` | Added `role: Option<String>`, `access_request_id: Option<String>` |
| `DefaultTokenService::validate_bearer_token()` | Returns `AuthContext` directly (was `(String, ResourceScope, Option<String>)`) |
| `DefaultTokenService::handle_external_client_token()` | Returns `(AuthContext, CachedExchangeResult)` (was `(String, ResourceScope, String)`) |
| `build_auth_context_from_bearer()` | Removed (inlined into token service) |
| `ApiAuthError::InvalidResourceScope` | Removed |

### `routes_app` Crate

| Type | Change |
|------|--------|
| `CreateAccessRequestBody.requested_role` | Added (typed as `UserScope`) |
| `CreateAccessRequestResponse` | Changed from tagged enum (`Draft`/`Approved`) to flat struct |
| `AccessRequestStatusResponse.resource_scope` | Removed |
| `AccessRequestStatusResponse.requested_role` | Added |
| `AccessRequestStatusResponse.approved_role` | Added (`Option<String>`) |
| `AccessRequestReviewResponse.requested_role` | Added |
| `ApproveAccessRequestBody.approved_role` | Added (typed as `UserScope`) |
| `AppAccessRequestError::InsufficientPrivileges` | Added (403) |
| `AppAccessRequestError::PrivilegeEscalation` | Added (403, with `approved` and `max_allowed` fields) |

### `services` Crate — Error Types

| Error | Change |
|-------|--------|
| `AccessRequestValidationError::PrivilegeEscalation` | Added (`approved_role`, `max_scope` fields) — used by token service |

---

## Key Files Summary

| Layer | File | Role in Feature |
|-------|------|-----------------|
| objs | `crates/objs/src/user_scope.rs` | `UserScope` simplified to User/PowerUser, `from_scope()` removed |
| objs | `crates/objs/src/token_scope.rs` | `TokenScope` simplified to User/PowerUser, `from_scope()` removed |
| objs | `crates/objs/src/resource_role.rs` | Added `max_user_scope()` method |
| services | `crates/services/migrations/0009_app_access_requests.up.sql` | Schema: `resource_scope` → `requested_role` + `approved_role` |
| services | `crates/services/migrations/0014_apps.up.sql` | Schema: removed `scope` column |
| services | `crates/services/src/db/objs.rs` | `AppAccessRequestRow` fields updated |
| services | `crates/services/src/db/service_access_request.rs` | SQL queries updated for new columns |
| services | `crates/services/src/access_request_service/service.rs` | Auto-approve removed, role params added |
| services | `crates/services/src/access_request_service/test_access_request_service.rs` | **New** — unit tests for service |
| services | `crates/services/src/auth_service/service.rs` | `register_resource_access()` removed |
| services | `crates/services/src/app_instance_service.rs` | `scope` field removed |
| auth_middleware | `crates/auth_middleware/src/resource_scope.rs` | **Deleted** — `ResourceScope` enum removed |
| auth_middleware | `crates/auth_middleware/src/token_service/service.rs` | Core change: returns `AuthContext` directly, DB-based role derivation, privilege escalation check |
| auth_middleware | `crates/auth_middleware/src/token_service/tests.rs` | Updated for new return types and role derivation |
| auth_middleware | `crates/auth_middleware/src/auth_middleware/middleware.rs` | Simplified: no `ResourceScope` matching, no `build_auth_context_from_bearer()` |
| auth_middleware | `crates/auth_middleware/src/api_auth_middleware.rs` | `scope` → `role` pattern match; removed `InvalidResourceScope` |
| auth_middleware | `crates/auth_middleware/src/test_utils/auth_server_test_client.rs` | Removed `resource_scope_name`, `request_audience_access()`, KC audience access step |
| auth_middleware | `crates/auth_middleware/tests/test_live_auth_middleware.rs` | Updated for new scope patterns and DB-based flow |
| routes_app | `crates/routes_app/src/routes_apps/types.rs` | Request/response types with `requested_role`, `approved_role` |
| routes_app | `crates/routes_app/src/routes_apps/error.rs` | Added `InsufficientPrivileges`, `PrivilegeEscalation` |
| routes_app | `crates/routes_app/src/routes_apps/handlers.rs` | Privilege escalation validation in approve handler |
| routes_app | `crates/routes_app/src/routes_apps/test_access_request.rs` | Tests for new role-based flow |
| server_app | `crates/server_app/tests/test_oauth_external_token.rs` | Integration tests updated for new scope patterns |
| server_app | `crates/server_app/tests/utils/external_token.rs` | Token construction updated |
| frontend | `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx` | Role dropdown, `computeRoleOptions()`, `useUser()` |
| frontend | `crates/bodhi/src/app/ui/apps/access-requests/review/page.test.tsx` | **New** — component tests for review page |
| frontend | `crates/bodhi/src/hooks/useAppAccessRequests.ts` | Updated types |
| frontend | `crates/bodhi/src/test-fixtures/app-access-requests.ts` | **New** — test fixtures |
| test-oauth-app | `crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx` | Simplified, scope fields removed |
| test-oauth-app | `crates/lib_bodhiserver_napi/test-oauth-app/src/components/ScopeDisplay.tsx` | **Deleted** |
| E2E | `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestReviewPage.mjs` | **New** — page object for review page |
| E2E | `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` | Updated scope assertions |
| E2E | `crates/lib_bodhiserver_napi/tests-js/utils/auth-server-client.mjs` | Removed scope-related setup steps |
| OpenAPI | `openapi.json` | Updated schemas for new request/response types |
| ts-client | `ts-client/src/types/types.gen.ts` | Regenerated TypeScript types |

---

## Caching Behavior

`CachedExchangeResult` stores the outcome of a token exchange to avoid repeated KC calls:

```rust
pub struct CachedExchangeResult {
  pub token: String,           // Exchanged access token
  pub app_client_id: String,   // External app's client ID
  pub role: Option<String>,    // DB-derived approved_role (serialized UserScope)
  pub access_request_id: Option<String>,  // Access request record ID
}
```

On cache hit, the cached result is used to reconstruct `AuthContext::ExternalApp` directly, parsing `role` back to `UserScope`. Token expiration is checked before using the cached result.

---

## Security Verification

The following security properties were verified during code review:

1. External app role derived exclusively from DB `approved_role`, never from JWT scope claims
2. `scope_user_*` scopes NOT forwarded during token exchange (only `scope_access_request:*`)
3. Constant-time comparison used for API token hash verification (unchanged)
4. Pre-exchange validation: status=approved, azp match, user_id match
5. Post-exchange validation: access_request_id claim matches DB record
6. `ExternalApp.role = None` when no validated access request → rejected by `api_auth_middleware`
7. Privilege escalation guard has two independent checks (approved > requested, approved > max_grantable) at the route handler level
8. Privilege escalation guard at token exchange level (approved_role > user's resource_role max_user_scope)
9. CSRF state parameter handling intact in OAuth flows

---

## Commit History

| Commit | Description |
|--------|-------------|
| (layer 1) | `objs` — UserScope/TokenScope simplified, ResourceRole.max_user_scope() added |
| `8f0af5b` | Layers 2-5 squashed — services, auth_middleware, routes_app, server_app |
| (layer 6) | Frontend — review page role dropdown, component tests |
| (layer 7) | E2E tests — updated OAuth fixtures, auth server client, specs |
| (layer 8) | Documentation — CLAUDE.md/PACKAGE.md updates, architecture docs |
| (reviews) | Post-review fixes — dead code removal, privilege escalation tests, service unit tests |

---

## Impact Summary

- **107 files changed** across 9 crates + frontend + E2E + docs
- **4,816 insertions, 2,309 deletions**
- Core type removals: `ResourceScope`, `UserScope::Admin/Manager`, `TokenScope::Admin/Manager`, `UserScope::from_scope()`, `TokenScope::from_scope()`
- Core type additions: `ResourceRole::max_user_scope()`, `AppAccessRequestError::PrivilegeEscalation`
- DB schema: 2 migrations modified (0009, 0014)
- API contract: 4 endpoints affected (create, status, review, approve)
- Token exchange return type fundamentally changed: `(String, ResourceScope, String)` → `AuthContext` directly
