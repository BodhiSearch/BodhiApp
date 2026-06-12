# Plan: Fix Review Findings from Scope Removal Commit

## Context

Commit `8f0af5b` squashed layers 2-5 of the KC scope removal work: replacing `resource_scope` with `requested_role`/`approved_role`, removing auto-approve, deriving external app role from DB instead of JWT. A thorough code review identified 19 findings (0 critical, 7 important, 12 nice-to-have) plus dead code. This plan addresses **all findings** plus user-identified dead code in `services/src/objs.rs`.

## Execution Strategy

Each layer is implemented by a dedicated sub-agent using the Task tool. Agents run **sequentially** (not parallel) because each layer depends on the previous layer compiling cleanly. Each agent:
1. Reads the relevant source files
2. Makes the specified changes
3. Runs the layer's verification command
4. Reports results

Layers 1-4 use `general-purpose` agents for Rust code changes.
Layer 5 uses `Bash` directly (make commands).
Layer 6 uses a `general-purpose` agent for frontend changes.
Layer 7 uses a `general-purpose` agent for E2E changes.
Layer 8 uses direct edits (trivial documentation fixes).

For Layer 6 (services unit tests), invoke the `test-services` skill to get canonical test patterns before writing tests.
For Layer 4 (routes_app test), invoke the `test-routes-app` skill to get canonical test patterns.

## Fix Order (Layered)

### Layer 1: objs crate — Sub-agent #1

**Files:**
- `crates/objs/src/user_scope.rs`
- `crates/objs/src/token_scope.rs`

**Changes:**

1. **Remove `MissingUserScope` dead variant** (Important #3)
   - Delete `MissingUserScope` variant from `UserScopeError` enum
   - Remove any match arms referencing it

2. **Add `has_access_to()` reflexive test** (Important #4)
   - Add explicit test: `assert!(UserScope::User.has_access_to(&UserScope::User))`
   - Add explicit test: `assert!(TokenScope::User.has_access_to(&TokenScope::User))`
   - Same for `PowerUser` variant

3. **Remove `TokenScope::from_scope()` and its tests** (N2H #8)
   - Delete `from_scope()` method from `TokenScope`
   - Delete `MissingTokenScope` variant from `TokenScopeError` if it becomes dead
   - Delete associated test functions

**Verify:** `cargo test -p objs`

---

### Layer 2: services crate — Sub-agent #2

**Files:**
- `crates/services/src/auth_service/tests.rs`
- `crates/services/src/objs.rs`
- `crates/services/src/access_request_service/service.rs` (add test module declaration)
- NEW: `crates/services/src/access_request_service/test_access_request_service.rs`

**Changes:**

4. **Remove duplicate `#[rstest]`** (Important #5)
   - `auth_service/tests.rs` line ~313: remove extra `#[rstest]` on `test_exchange_auth_code_success`

5. **Remove dead types from `services/src/objs.rs`** (User-identified dead code)
   - Delete `AppAccessRequest` struct (lines 45-52)
   - Delete `AppAccessResponse` struct (lines 54-60)
   - Delete `AppAccessRequestDetail` struct (lines 62-85)
   - These have zero usages — routes_app uses its own response DTOs

6. **Add `DefaultAccessRequestService` unit tests** (Important #6)
   - Create `test_access_request_service.rs` using TestDbService (real SQLite)
   - Add `#[cfg(test)] #[path = "test_access_request_service.rs"] mod test_access_request_service;` to `service.rs`
   - Test cases:
     - `create_draft` with valid params → returns draft with correct `requested_role`
     - `create_draft` with invalid `flow_type` → `InvalidFlowType`
     - `create_draft` with `flow_type = "redirect"` + missing `redirect_uri` → `MissingRedirectUri`
     - `approve_request` on non-draft → `AlreadyProcessed`
     - `approve_request` threads `approved_role` correctly into DB
     - `get_request` on expired draft → `Expired`

**Verify:** `cargo test -p objs -p services`

---

### Layer 3: auth_middleware crate — Sub-agent #3

**Files:**
- `crates/auth_middleware/src/token_service/service.rs`

**Changes:**

7. **Inline redundant `access_request_scopes` re-filter** (N2H #10)
   - Remove the second `access_request_scopes` variable that re-filters already-filtered `scopes`
   - Rename `scopes` to `access_request_scopes` since it's already filtered to `scope_access_request:*`
   - Create `exchange_scopes` by cloning and extending with `["openid", "email", "profile", "roles"]`

8. **Change `tracing::error!` to `tracing::warn!`** (N2H #11)
   - Line ~291: missing KC `access_request_id` claim is a KC config issue, not app error

9. **Use in-scope `access_request_scope` binding instead of `[0]` index** (N2H #12)
   - Replace `access_request_scopes[0]` with the already-bound `access_request_scope` variable

10. **Add cache-hit test with `role: Some(...)`** (N2H #13)
    - Add test that calls `validate_bearer_token` twice with same token
    - Assert second call returns `AuthContext::ExternalApp { role: Some(...) }` from cache

**Verify:** `cargo test -p objs -p services -p auth_middleware`

---

### Layer 4: routes_app crate — Sub-agent #4

**Files:**
- `crates/routes_app/src/routes_apps/types.rs`
- `crates/routes_app/src/routes_apps/handlers.rs`
- `crates/routes_app/src/routes_apps/test_access_request.rs`

**Changes:**

11. **Change `ApproveAccessRequestBody.approved_role` from `String` to `UserScope`** (Important #1)
    - `types.rs`: change field type to `UserScope`, update OpenAPI example
    - `handlers.rs`: remove `.parse::<UserScope>()` call, use typed value directly
    - `handlers.rs`: pass `body.approved_role.to_string()` to service call

12. **Add test for `approved_role > requested_role`** (Important #2)
    - Add `test_approve_privilege_escalation_approved_exceeds_requested`
    - Setup: draft with `requested_role = "scope_user_user"`, approver = `ResourceRole::PowerUser`, body `approved_role = "scope_user_power_user"`
    - Assert: 403 with error code `app_access_request_error-privilege_escalation`

13. **Update stale OpenAPI description** (N2H #14)
    - `handlers.rs`: remove "auto-approves" language from `createAccessRequest` utoipa description

**Verify:** `cargo test -p objs -p services -p auth_middleware -p routes_app`

---

### Layer 5: Full backend + ts-client — Direct (no sub-agent)

14. **Full backend validation:** `make test.backend`

15. **Regenerate ts-client:** `make build.ts-client`
    - The `UserScope` type change on `approved_role` will update the generated TypeScript types

---

### Layer 6: Frontend — Sub-agent #5

**Files:**
- `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`

**Changes:**

16. **Remove duplicate `data-testid` from `<Select>` wrapper** (N2H #15)
    - Remove `data-testid="review-approved-role-select"` from `<Select>`, keep only on `<SelectTrigger>`

17. **Remove dead fallback `?? reviewData.requested_role`** (N2H #16)
    - In `handleApprove`, change `approvedRole ?? reviewData.requested_role` to just `approvedRole!` (non-null assertion) since `canApprove` guarantees non-null

**Verify:** `cd crates/bodhi && npm test`

---

### Layer 7: E2E — Sub-agent #6

**Files:**
- `crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs`
- `crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`

**Changes:**

18. **Remove dead `getResourceScope()` method** (N2H #17)
    - Delete `getResourceScope()` from `ConfigSection.mjs` — reads `[data-test-resource-scope]` which no longer exists

19. **Add E2E test for role downgrade flow** (Important #7)
    - Add test case(s) to `oauth2-token-exchange.spec.mjs`
    - Flow: submit access request with `scope_user_power_user` → review page → select `scope_user_user` (downgrade) → approve → verify resulting token role

**Verify:** `make build.ui-rebuild && make test.napi`

---

### Layer 8: Documentation / Cross-cutting — Direct (no sub-agent)

**Files:**
- `CLAUDE.md`
- `crates/server_app/tests/resources/.env.test.example`

**Changes:**

20. **Fix CLAUDE.md typo** (N2H #18)
    - Change `{toolsets:[--]}` to `{toolsets:[...]}`

21. **Remove obsolete env var from `.env.test.example`**
    - Remove `INTEG_TEST_RESOURCE_CLIENT_SCOPE` line

---

## Verification

After all changes:
1. `cargo test -p objs` — layer 1
2. `cargo test -p objs -p services` — layer 2
3. `cargo test -p objs -p services -p auth_middleware` — layer 3
4. `cargo test -p objs -p services -p auth_middleware -p routes_app` — layer 4
5. `make test.backend` — full backend
6. `make build.ts-client` — regenerate TypeScript types
7. `cd crates/bodhi && npm test` — frontend
8. `make build.ui-rebuild && make test.napi` — E2E
