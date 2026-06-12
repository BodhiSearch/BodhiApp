# Plan: Role Selection Dropdown for Access Request Review + Privilege Escalation Guard

## Context

The access request review page (`/ui/apps/access-requests/review`) currently shows only "Deny" and "Approve All" buttons with no way for the reviewing user to choose what role to grant. The `approved_role` is always echoed back as the `requested_role` from the access request. This has two gaps:

1. **UI gap**: When an app requests `scope_user_power_user`, the approving user (who may only be `resource_user`) cannot downgrade to `scope_user_user`. There is no dropdown to select the approved role.
2. **Backend gap**: No privilege escalation validation — any session user can grant any role string (including roles above their own level).
3. **Test app gap**: `ConfigForm.tsx` hardcodes `requested_role: 'scope_user_user'`, preventing E2E tests from using `scope_user_power_user`.

**Goal**: Add a role-selection dropdown on the review page, add backend validation, extend the test-oauth-app, and add an E2E test for role downgrade.

---

## Role Logic

`approved_role` is a `UserScope` value (`scope_user_user` or `scope_user_power_user`).

The approver's max grantable `UserScope` is derived from their session `ResourceRole`:
- `ResourceRole::User` → max `UserScope::User` (`scope_user_user`)
- `ResourceRole::PowerUser | Manager | Admin` → max `UserScope::PowerUser` (`scope_user_power_user`)

The available options shown in the dropdown = all `UserScope` values where:
```
scope <= min(parse(requested_role), max_grantable_by_user_role)
```

Examples:
- `requested_role = scope_user_power_user` + user is `resource_power_user` → options: `[scope_user_power_user, scope_user_user]`
- `requested_role = scope_user_power_user` + user is `resource_user` → options: `[scope_user_user]`
- `requested_role = scope_user_user` + user is `resource_admin` → options: `[scope_user_user]`

Default selection: the lowest available option that still satisfies `requested_role` without exceeding user's role (i.e., `min(requested_role, user_max)` which is always the effective cap, so default = the first/highest available = `min(requested_role, user_max)`).

---

## Changes by Layer

### Layer 1: `routes_app` — Backend Validation

**File**: `crates/routes_app/src/routes_apps/handlers.rs`

In `approve_access_request_handler`, after extracting `user_id` and before tool/MCP validation:

1. Extract the approver's session `ResourceRole` from `AuthContext::Session { role, .. }`. If role is `None`, return `InsufficientPrivileges` error.
2. Compute max grantable: `if role >= ResourceRole::PowerUser { UserScope::PowerUser } else { UserScope::User }`
3. Parse `body.approved_role` as `UserScope` (return `BadRequest` if invalid string via `UserScopeError`)
4. Fetch the access request to get `request.requested_role`, parse it as `UserScope`
5. Validate: `approved_scope > requested_scope` → return `PrivilegeEscalation`
6. Validate: `approved_scope > max_grantable` → return `PrivilegeEscalation`

Note: The access request must be fetched before tool/MCP validation anyway (it currently is fetched in the review handler but not the approval handler). Add fetch of access request record at the start of `approve_access_request_handler`.

**File**: The error enum for `AppAccessRequestError` (in `crates/routes_app/src/routes_apps/` — check where it's defined: likely `handlers.rs` or a types file):

Add:
```rust
#[error("approved role {approved:?} exceeds allowed maximum for this user")]
#[error_meta(error_type = ErrorType::Forbidden)]
PrivilegeEscalation { approved: String, max_allowed: String },
```

Also add `InvalidUserScope(#[from] UserScopeError)` or handle inline via `map_err`.

**File**: `crates/routes_app/src/routes_apps/test_access_request.rs`

Add test cases:
- `test_approve_privilege_escalation_user_grants_power_user`: user with `resource_user` role attempts to grant `scope_user_power_user` → 403
- `test_approve_valid_downgrade_power_user_grants_user`: user with `resource_power_user` role grants `scope_user_user` on a `scope_user_power_user` request → 200

### Layer 2: `bodhi/src` — Review Page UI

**File**: `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`

Changes to `ReviewContent`:

1. **Import `useUser`** from `@/hooks/useUsers` (existing hook returning `UserResponse`).
2. **Add state**: `const [approvedRole, setApprovedRole] = useState<string | null>(null);`
3. **Derive role options** via `useMemo`:
   ```typescript
   const roleOptions = useMemo(() => {
     if (!reviewData) return [];
     const requestedScope = reviewData.requested_role; // e.g. 'scope_user_power_user'
     const userRole = userData?.auth_status === 'logged_in' ? userData.role : null;
     // compute available options: all UserScope values <= min(requestedScope, maxGrantable)
     // Return array like: [{value: 'scope_user_power_user', label: 'Power User'}, {value: 'scope_user_user', label: 'User'}]
     ...
   }, [reviewData, userData]);
   ```
4. **Initialize `approvedRole`** in `useEffect` when `roleOptions` changes: set to `roleOptions[0]?.value ?? null` (the max grantable option).
5. **Add role dropdown** before the Deny/Approve buttons section:
   ```tsx
   <div className="mb-4">
     <Label>Approved Role</Label>
     <Select value={approvedRole ?? ''} onValueChange={setApprovedRole}
       data-testid="review-approved-role-select">
       {roleOptions.map(opt => (
         <SelectItem key={opt.value} value={opt.value}
           data-testid={`review-approved-role-option-${opt.value}`}>
           {opt.label}
         </SelectItem>
       ))}
     </Select>
   </div>
   ```
6. **Update `handleApprove`**: use `approvedRole ?? reviewData.requested_role` instead of `reviewData.requested_role`.
7. **Update `canApprove`**: also require `approvedRole !== null`.

**File**: `crates/bodhi/src/test-fixtures/app-access-requests.ts`

Add/update fixtures for:
- `mockDraftReviewResponsePowerUser`: `requested_role: 'scope_user_power_user'` variant
- Verify existing fixtures still work (they use `scope_user_user` which shows only one option)

**Component tests** (co-located with the review page or in a test file):
- When `requested_role = scope_user_power_user` + user is `resource_power_user` → 2 options in dropdown
- When `requested_role = scope_user_power_user` + user is `resource_user` → 1 option (scope_user_user only)
- When `requested_role = scope_user_user` → 1 option always
- Approve submission uses selected `approved_role` not `requested_role`

### Layer 3: `test-oauth-app` — Configurable `requested_role`

**File**: `crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx`

1. Add `requestedRole` state: `const [requestedRole, setRequestedRole] = useState('scope_user_user');`
2. Add a dropdown/select for `requested_role` field with options `scope_user_user` and `scope_user_power_user`
3. Add `data-testid="input-requested-role"` (or a select with `data-testid="select-requested-role"`)
4. Use `requestedRole` in the request body: `requested_role: requestedRole`

**File**: `crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs`

1. Add selector: `requestedRole: '[data-testid="select-requested-role"]'` (or appropriate testid)
2. Add method: `async setRequestedRole(value)` to interact with the dropdown
3. Update `configureOAuthForm` to accept `requestedRole` parameter (default: `'scope_user_user'`)

### Layer 4: E2E Tests — Role Downgrade Test

**File**: `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestReviewPage.mjs`

Add:
- Selector: `approvedRoleSelect: '[data-testid="review-approved-role-select"]'`
- Method: `async selectApprovedRole(role)` — clicks the select and chooses the given role option via `data-testid="review-approved-role-option-{role}"`
- Method: `async approveWithRole(role)` — calls `waitForReviewPage()`, `selectApprovedRole(role)`, `clickApprove()`

**New test** (add to `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-auth.spec.mjs` or a new file `mcps-role-selection.spec.mjs`):

**Test: "Approver downgrades scope_user_power_user request to scope_user_user"**

Flow:
1. Login as admin, create MCP instance
2. Configure test-oauth-app with `requested_role: 'scope_user_power_user'`, MCP in `requested`
3. Submit access request → redirected to review page
4. On review page: verify dropdown shows two options (`scope_user_power_user`, `scope_user_user`)
5. Select `scope_user_user` from dropdown, approve with MCP instance
6. Complete OAuth callback → login
7. Call `/bodhi/v1/user` with external app token → verify `role = scope_user_user` (NOT power_user)
8. Call `/bodhi/v1/info` or another endpoint to confirm role level

---

## Critical Files

| File | Change |
|------|--------|
| `crates/routes_app/src/routes_apps/handlers.rs` | Add privilege escalation validation in `approve_access_request_handler` |
| `crates/routes_app/src/routes_apps/` (error type location) | Add `PrivilegeEscalation` error variant to `AppAccessRequestError` |
| `crates/routes_app/src/routes_apps/test_access_request.rs` | Add backend validation tests |
| `crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx` | Add role dropdown, use `useUser`, update approval body |
| `crates/bodhi/src/test-fixtures/app-access-requests.ts` | Add power_user fixture variants |
| `crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx` | Add `requested_role` input field |
| `crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs` | Add `requestedRole` param and setter |
| `crates/lib_bodhiserver_napi/tests-js/pages/AccessRequestReviewPage.mjs` | Add `selectApprovedRole` and `approveWithRole` methods |
| `crates/lib_bodhiserver_napi/tests-js/specs/mcps/` (new or existing spec) | Add role downgrade E2E test |

---

## Verification

1. **Backend unit tests**: `cargo test -p routes_app -- access_request` — new privilege escalation tests pass
2. **Frontend component tests**: `cd crates/bodhi && npm test` — review page tests covering dropdown options and approval body
3. **UI rebuild**: `make build.ui-rebuild` after UI changes
4. **E2E**: Run `mcps-oauth-auth.spec.mjs` (or new spec) via `npm run test:playwright:headed` — verify role downgrade flow works end-to-end
5. **Manual**: Open review page for a `scope_user_power_user` access request → verify dropdown shows correct options based on approver's role; approve with downgraded role → verify token has `scope_user_user` in KC
