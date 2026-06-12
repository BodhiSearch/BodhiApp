# routes_app Code Review — Scope Removal / Access Request Flow Changes

**Review Date:** 2026-02-27
**Reviewer:** Claude Code (claude-sonnet-4-6)
**Scope:** HEAD commit — routes_app crate changes for scope removal and access request flow

---

## Summary

The changes replace the old enum-based `CreateAccessRequestResponse` (Draft/Approved variants) with a simple struct that always returns "draft", add `requested_role` / `approved_role` fields to status and review responses (replacing `resource_scope`), and add an `approved_role` field to `ApproveAccessRequestBody` with privilege escalation guards. Two new error variants (`InsufficientPrivileges`, `PrivilegeEscalation`) are added. Overall the logic is sound, but there are two confirmed gaps that must be fixed.

---

## Findings

### P1 — Important (User Confirmed)

#### Finding 1: `ApproveAccessRequestBody.approved_role` should be `UserScope`, not `String`

**File:** `crates/routes_app/src/routes_apps/types.rs`, line 152

**Current code:**
```rust
pub struct ApproveAccessRequestBody {
  /// Role to grant for the approved request
  pub approved_role: String,
  /// Approved resources with selections
  pub approved: ApprovedResources,
}
```

**Problem:** `approved_role` is declared as `String`. The handler immediately parses it via `.parse::<UserScope>()` (handlers.rs line 303), meaning any parse failure returns a runtime error instead of a structured serde deserialization error. Using `String` also means the OpenAPI schema for this field emits a plain `string` type instead of a constrained enum, giving API clients no type information about valid values.

**Contrast:** `CreateAccessRequestBody.requested_role` is correctly typed as `UserScope` (types.rs line 26):
```rust
pub requested_role: UserScope,
```

**Fix:** Change `approved_role` field type from `String` to `UserScope`. Update the OpenAPI schema example accordingly. The handler can then remove the `.parse::<UserScope>()` call and use the typed value directly — and pass `approved_scope.to_string()` where a String is needed for the service call.

**Impact:** The parse call at handlers.rs line 303 (`body.approved_role.parse::<UserScope>()`) needs to be removed. The `PrivilegeEscalation` error struct fields (`approved: String`, `max_allowed: String`) can remain as `String` since they are for display in error messages, populated via `.to_string()`.

---

#### Finding 2: Missing test for `approved_role > requested_role` (privilege escalation check #1)

**File:** `crates/routes_app/src/routes_apps/test_access_request.rs`

**Current state:** The existing test `test_approve_privilege_escalation_user_grants_power_user` (line 507) covers check #2 only — the approver's role ceiling (`approved_scope > max_grantable`). In that test the `requested_role` in the DB is `scope_user_power_user` and the approver has `ResourceRole::User`, so it is `max_grantable` that is violated (`UserScope::User < UserScope::PowerUser`), not the `approved > requested` check.

**Missing scenario:** A test where:
- `requested_role` in the DB is `scope_user_user` (app asked for User scope)
- Approver has `ResourceRole::PowerUser` (so `max_grantable = UserScope::PowerUser`)
- `approved_role` in the request body is `scope_user_power_user`

Check #2 would pass (`approved_scope <= max_grantable`), but check #1 should catch it (`approved_scope > requested_scope`).

**Handlers.rs check order (lines 314–327):**
```rust
// Validate: approved can't exceed what was requested
if approved_scope > requested_scope {
    return Err(AppAccessRequestError::PrivilegeEscalation { ... })?;
}
// Validate: approved can't exceed what the approver is allowed to grant
if approved_scope > max_grantable {
    return Err(AppAccessRequestError::PrivilegeEscalation { ... })?;
}
```

Check #1 is the correct first guard — it ensures the approved role can never exceed what the app actually requested, regardless of what the approver could theoretically grant. Without a test for this path, a future regression (e.g., accidentally swapping the two conditions) would not be caught.

**Required test name (suggested):** `test_approve_privilege_escalation_approved_exceeds_requested`

**Test setup:**
- Seed draft request with `requested_role = "scope_user_user"`
- Approver auth context: `ResourceRole::PowerUser` (so check #2 would pass)
- Request body: `approved_role = "scope_user_power_user"`
- Expected: `StatusCode::FORBIDDEN`, error code `"app_access_request_error-privilege_escalation"`

---

### P2 — Minor / Observations

#### Finding 3: Error propagation chain for invalid `approved_role` string is indirect

**File:** `crates/routes_app/src/routes_apps/handlers.rs`, line 303

```rust
let approved_scope: UserScope = body.approved_role.parse()?;
```

`UserScope::from_str` returns `UserScopeError::InvalidUserScope`. The `?` operator requires a `From<UserScopeError>` impl on `ApiError`. This chain works if `AppAccessRequestError` has a `#[from] UserScopeError` variant or if `ApiError` has a blanket impl. Currently `AppAccessRequestError` in `error.rs` has no `UserScopeError` variant — the parse error would need to be handled via a direct `From<UserScopeError>` impl on `ApiError` or similar.

This is a compilation concern that should be verified. If `approved_role` is changed to `UserScope` (Finding 1), this issue disappears entirely, since serde handles deserialization before the handler runs. This is an additional reason to fix Finding 1.

---

#### Finding 4: `AccessRequestStatusResponse.requested_role` and `approved_role` are plain `String` types

**File:** `crates/routes_app/src/routes_apps/types.rs`, lines 61–65

```rust
pub requested_role: String,
pub approved_role: Option<String>,
```

These are response-only types used for status polling. Using `String` is acceptable here (data comes from the DB and the DB stores the string representation). However, `UserScope` would provide stronger OpenAPI documentation. This is a lower-priority consistency improvement.

---

#### Finding 5: `AuthContext::Session` pattern match is correct

**File:** `crates/routes_app/src/routes_apps/handlers.rs`, lines 291–295

```rust
let approver_role = if let AuthContext::Session { role, .. } = &auth_context {
    role.ok_or(AppAccessRequestError::InsufficientPrivileges)?
} else {
    return Err(AppAccessRequestError::InsufficientPrivileges)?;
};
```

This is correct. The pattern matches the `AuthContext::Session` variant and extracts `role: Option<ResourceRole>`. The `.ok_or(...)` correctly handles the case where the session exists but the user has no assigned role yet. Non-session auth contexts (ApiToken, ExternalApp, Anonymous) are all rejected. The `InsufficientPrivileges` error type (`ErrorType::Forbidden`) is appropriate.

---

#### Finding 6: `ResourceRole::PowerUser` as threshold for max grantable scope is correct

**File:** `crates/routes_app/src/routes_apps/handlers.rs`, lines 296–300

```rust
let max_grantable = if approver_role >= ResourceRole::PowerUser {
    UserScope::PowerUser
} else {
    UserScope::User
};
```

`ResourceRole` ordering is `User < PowerUser < Manager < Admin`. The condition `approver_role >= ResourceRole::PowerUser` correctly includes PowerUser, Manager, and Admin. The mapping to `UserScope::PowerUser` (the maximum available `UserScope`) for these roles is correct. `ResourceRole::User` gets `UserScope::User` only. The logic matches the documented role hierarchy.

---

#### Finding 7: `CreateAccessRequestResponse` is now a simple struct — OpenAPI description mismatch

**File:** `crates/routes_app/src/routes_apps/handlers.rs`, line 38

```
description = "Create an access request for an app to access user resources. If no tools requested, auto-approves. Unauthenticated endpoint.",
```

The description still says "If no tools requested, auto-approves." This is no longer true — the handler always returns "draft" status and never auto-approves. The old auto-approve logic has been removed. The description should be updated to reflect the current always-draft behavior.

**Fix:** Update the `description` in the `#[utoipa::path]` annotation to remove the auto-approve language.

---

#### Finding 8: `access_request_service.approve_request(...)` receives `body.approved_role` as `String`

**File:** `crates/routes_app/src/routes_apps/handlers.rs`, line 408

```rust
let updated = access_request_service
    .approve_request(
        &id,
        user_id,
        token,
        body.approved.toolsets,
        body.approved.mcps,
        body.approved_role,   // <-- passes the String directly
    )
    .await?;
```

If `approved_role` is changed to `UserScope` (Finding 1), this call site would need to pass `body.approved_role.to_string()` (or the service signature should also be updated to accept `UserScope`). This is a dependent change that must accompany Finding 1.

---

## Test Coverage Analysis

| Scenario | Covered | Test Name |
|---|---|---|
| Approve success — popup flow | Yes | `test_approve_access_request_success::popup_flow` |
| Approve success — redirect flow | Yes | `test_approve_access_request_success::redirect_flow` |
| Instance not owned by user | Yes | `test_approve_access_request_instance_not_owned` |
| Instance disabled | Yes | `test_approve_access_request_instance_not_enabled` |
| Instance missing API key | Yes | `test_approve_access_request_instance_no_api_key` |
| Deny success — popup + redirect | Yes | `test_deny_access_request_success` |
| Check #2: approved > max_grantable | Yes | `test_approve_privilege_escalation_user_grants_power_user` |
| Check #1: approved > requested | **MISSING** | — |
| Power user downgrades requested scope | Yes | `test_approve_valid_downgrade_power_user_grants_user` |
| No session role (`role: None`) | Not covered | — |
| Non-Session auth context (ApiToken/ExternalApp) | Not covered | — |
| Invalid `approved_role` string (parse error) | Not covered | — |

The "No session role" and "Non-Session auth context" gaps are lower priority since auth middleware upstream should prevent these cases from reaching the handler in production, but dedicated handler-level tests would increase confidence.

---

## Action Items

| Priority | Item | File |
|---|---|---|
| P1 | Change `ApproveAccessRequestBody.approved_role` from `String` to `UserScope` | `types.rs` |
| P1 | Update handler to remove `.parse::<UserScope>()` call; pass `.to_string()` to service | `handlers.rs` |
| P1 | Add test: `approved_role > requested_role` (check #1 in isolation) | `test_access_request.rs` |
| P2 | Fix OpenAPI description: remove "auto-approves" language | `handlers.rs` |
| P2 | Verify `UserScopeError` error propagation compiles (resolves automatically with P1 fix) | `handlers.rs` |
