# UI + E2E Review: Scope Removal / Access Request Flow Changes

**Date**: 2026-02-27
**Reviewer**: Claude (claude-sonnet-4-6)
**Scope**: Frontend (`bodhi/src`) + E2E/test-oauth-app layer

---

## Summary

The changes introduce a role-selection dropdown on the access request review page, remove the `ScopeDisplay` component, add `requested_role` to the test-oauth-app `ConfigForm`, and update E2E page objects and specs to reflect the scope-removal architecture. The implementation is overall clean and correct, with one important gap (missing E2E spec for role downgrade) and several minor findings noted below.

---

## Findings

### IMPORTANT — Missing E2E Spec for Role Downgrade Flow

**File**: (no file — this is a gap in coverage)

The `AccessRequestReviewPage.mjs` page object adds `selectApprovedRole()` and `approveWithRole()` methods, but no E2E spec exercises the role downgrade path end-to-end. The component tests (`page.test.tsx`) verify the unit-level behavior (2 options for power_user, 1 option for resource_user, correct `approved_role` in the body), but there is no Playwright spec that:

- Submits an access request with `requested_role: scope_user_power_user`
- Visits the review page as an admin with a role that can grant power_user
- Selects `scope_user_user` from the dropdown (downgrade)
- Approves and verifies that the approved token/role is `scope_user_user`

This is a confirmed gap. The page object infrastructure exists and is correct; the spec is simply absent. This should be added as a dedicated test in an appropriate spec file (e.g., a new `specs/oauth/oauth-role-downgrade.spec.mjs` or as an additional test in `oauth2-token-exchange.spec.mjs`).

---

### Minor Findings

#### 1. Duplicate `data-testid` on `SelectTrigger`

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`
**Lines**: 362-364

```tsx
<Select
  value={approvedRole ?? ''}
  onValueChange={setApprovedRole}
  data-testid="review-approved-role-select"  // <-- ignored by Radix Select
>
  <SelectTrigger data-testid="review-approved-role-select">  // <-- the real DOM element
```

The `data-testid` is set on both the `<Select>` wrapper (which does not produce a DOM element) and on `<SelectTrigger>`. The prop on `<Select>` has no effect because Radix's `Select` component does not forward unknown props to a DOM node. This is harmless but misleading. Remove the `data-testid` from the `<Select>` wrapper — only the `<SelectTrigger>` matters.

The component tests (`page.test.tsx` line 1169) correctly target `review-approved-role-select` via `screen.getByTestId('review-approved-role-select')` and this works because it finds the `SelectTrigger`. However, the redundant prop on `<Select>` should be removed for clarity.

---

#### 2. `canApprove` does not guard against `roleOptions.length === 0`

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`
**Lines**: 188-205

```tsx
const canApprove = useMemo(() => {
  if (!reviewData) return false;
  if (!approvedRole) return false;  // guards null but not empty roleOptions
  ...
}, [...]);
```

When `roleOptions.length === 0` (i.e., `requestedRole` is unknown/invalid), `approvedRole` stays `null` (the `useEffect` at line 165 only sets it when `roleOptions.length > 0`), so `canApprove` returns `false`. This is correct behavior. However, the section guard at line 355:

```tsx
{roleOptions.length > 0 && (
  <div data-testid="review-approved-role-section">
```

means the dropdown is hidden for an invalid `requestedRole`. No bug, but there is no fallback UI message if a `requestedRole` comes from the server that is not in `SCOPE_ORDER`. In practice this should not happen if the server validates values, but it is worth noting as a silent failure path — the admin would see no role section and the Approve button would be permanently disabled with no explanation.

**Recommendation**: Add a comment or a small warning UI for the case where `roleOptions` is empty on a draft request (this means the server returned an unrecognised `requested_role`).

---

#### 3. `handleApprove` fallback `?? reviewData.requested_role` is dead code

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`
**Line**: 270

```tsx
const body: ApproveAccessRequestBody = {
  approved_role: approvedRole ?? reviewData.requested_role,
```

Because `canApprove` is `false` when `approvedRole` is `null`, the Approve button is disabled. `handleApprove` can only be called when `canApprove` is true, which requires `approvedRole !== null`. The `?? reviewData.requested_role` fallback is unreachable. This is not a bug but is misleading. Consider removing the fallback and using a non-null assertion or a type-narrowed variable instead.

---

#### 4. Component test `setupHandlers` always uses `resource_user`

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/app/ui/apps/access-requests/review/page.test.tsx`
**Line**: 109

```tsx
const setupHandlers = (reviewData?) => {
  const handlers = [...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' })];
```

The `setupHandlers` helper always sets `resource_user`. All earlier tests (loading/error, approve flow, deny flow, non-draft states, multi-tool, partial approve, MCP review, mixed resources) use `setupHandlers`, which means they all run with a `resource_user`. For a `scope_user_user` request this limits the dropdown to 1 option and `approvedRole` is initialized to `scope_user_user`. This is correct for those tests.

The Role Selection Dropdown describe block correctly does NOT use `setupHandlers` and instead calls `server.use(...)` directly with `resource_power_user` where needed. This is correct.

However, none of the "Approve Flow" tests verify the `approved_role` field in the captured body for the default (resource_user + scope_user_user) case. The partial-approve test at line 771 captures the body but only checks the toolsets/mcps shape — it does not assert `body.approved_role`. The role-downgrade test (line 1220) does verify `approved_role`. This is a gap in the existing "Approve Flow" tests, but it is minor since the role downgrade test covers the field directly.

---

#### 5. `useEffect` for `roleOptions` initialization — stable, no infinite loop

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/app/ui/apps/access-requests/review/page.tsx`
**Lines**: 165-169

```tsx
useEffect(() => {
  if (roleOptions.length > 0) {
    setApprovedRole(roleOptions[0].value);
  }
}, [roleOptions]);
```

`roleOptions` is a `useMemo` derived from `reviewData` and `userData`. It only changes when either of those changes. Setting `approvedRole` via `setApprovedRole` does not affect `reviewData` or `userData`, so there is no feedback loop. This is correct.

One edge case: if `userData` arrives after `reviewData` (two separate fetches), `roleOptions` will recompute and the `useEffect` will reset `approvedRole` to the first option. If the user had already changed the dropdown selection, it will be wiped. This is unlikely in practice (both queries resolve quickly) but worth noting.

---

#### 6. `ApproveAccessRequestBody` is defined locally, not from `@bodhiapp/ts-client`

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/hooks/useAppAccessRequests.ts`
**Lines**: 20-38

```ts
export interface ApproveAccessRequestBody {
  approved_role: string;
  ...
}
```

The `ApproveAccessRequestBody` interface is hand-authored in the hooks file rather than generated from the OpenAPI spec via `@bodhiapp/ts-client`. If the backend changes the approve request shape, the frontend type will silently diverge. This is consistent with how other non-generated request body types are handled in the codebase, but it is worth flagging — ideally this type should be exported by `ts-client` from the OpenAPI spec.

---

### E2E / test-oauth-app Layer Findings

#### 7. `auth-server-client.mjs` — `createResourceClient` returns `scope` in destructuring but scope is unused

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi/tests-js/utils/auth-server-client.mjs`
**Lines**: 247-252

```js
return {
  clientId: data.client_id,
  clientSecret: data.client_secret,
  scope: data.scope,   // <-- scope is part of the return value
};
```

The `scope` field is present in the return value of `createResourceClient`, but `getPreConfiguredResourceClient()` (line 590) does not include a `scope` field. Callers that use `createResourceClient` and destructure `scope` from the result would get `undefined` if the server no longer returns it. Given that scope removal is the stated goal of this work, the `scope` field in `createResourceClient`'s return value may be vestigial. This should be cleaned up if `scope` is no longer expected from the resource client creation endpoint.

---

#### 8. `oauth2-token-exchange.spec.mjs` — `configureOAuthForm` does not pass `requestedRole`

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi/tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`
**Lines**: 49-57

```js
await app.config.configureOAuthForm({
  bodhiServerUrl: SHARED_SERVER_URL,
  authServerUrl: authServerConfig.authUrl,
  realm: authServerConfig.authRealm,
  clientId: appClient.clientId,
  redirectUri,
  scope: testData.scopes,
  requested: null,
  // requestedRole not passed -> uses ConfigForm's default 'scope_user_user'
});
```

`requestedRole` is not passed, so the form uses its default value `scope_user_user`. This is fine for the test's purpose (it only checks that OAuth flow works and `userInfo.role` is `scope_user_user`). However, there is no test that exercises the flow with `requestedRole: scope_user_power_user` submitted through the test app — this aligns with the missing E2E spec for role downgrade noted in the Important section.

---

#### 9. `ConfigSection.mjs` — `getResourceScope` and `getAccessRequestScope` methods are present but `ScopeDisplay` component is deleted

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi/tests-js/pages/sections/ConfigSection.mjs`
**Lines**: 78-88

```js
async getResourceScope() {
  return await this.page
    .locator('[data-test-resource-scope]')
    .getAttribute('data-test-resource-scope');
}

async getAccessRequestScope() {
  return await this.page
    .locator('[data-test-access-request-scope]')
    .getAttribute('data-test-access-request-scope');
}
```

The `ScopeDisplay` component has been deleted, but `ConfigSection.mjs` still has `getResourceScope()` which reads `[data-test-resource-scope]`. This attribute no longer exists in the UI. The `getAccessRequestScope()` method reads `[data-test-access-request-scope]` which is still present in `AccessCallbackPage.tsx` (line 139), so that one is still valid.

`getResourceScope()` should either be removed or have a comment noting the attribute source if it was moved elsewhere. Retaining dead page object methods increases maintenance confusion.

---

#### 10. `AccessCallbackPage.tsx` — uses `data.access_request_scope` but this field may not be in the API contract

**File**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi/test-oauth-app/src/pages/AccessCallbackPage.tsx`
**Lines**: 48-56

```tsx
if (data.status === 'approved') {
  let updatedScope = config.scope;
  if (data.access_request_scope) {
    updatedScope = config.scope + ' ' + data.access_request_scope;
  }
  ...
  setAccessRequestScope(data.access_request_scope || null);
```

This code reads `data.access_request_scope` from the access request status response. Given that the scope removal work is removing scope-based fields, it is worth confirming whether the `access_request_scope` field is still present in the `GET /bodhi/v1/apps/access-requests/{id}` response after the scope removal changes. If it has been removed from the backend, this code will silently treat it as absent (the `if (data.access_request_scope)` guard handles the missing case), which means the scope appending logic becomes a no-op. This is likely intentional, but the test-oauth-app code documents this field usage that may no longer be active.

---

## Checklist Assessment

| Checklist Item | Status | Notes |
|---|---|---|
| Types imported from `@bodhiapp/ts-client` | Pass | `AccessRequestReviewResponse`, `McpServerReviewInfo`, etc. are all from `@bodhiapp/ts-client`. `ApproveAccessRequestBody` is hand-authored (finding 6). |
| `data-testid` attributes on interactive elements | Pass with minor issue | All interactive elements have correct `data-testid`. Duplicate on `<Select>` wrapper (finding 1). |
| `useUser` hook properly handles `auth_status` check | Pass | Line 161: `userData?.auth_status === 'logged_in' ? userData.role : null` is correct. |
| Role computation logic correct (`SCOPE_ORDER` ordering, index math) | Pass | `Math.max(requestedIndex, maxGrantableIndex)` correctly implements "lower bound" on granted scope. `SCOPE_ORDER` is ordered highest-to-lowest privilege, so higher index = lower privilege. `slice(startIndex)` yields all scopes at or below the capped level. |
| `useEffect` for `approvedRole` initialization — no infinite loop | Pass | See finding 5 — stable, but has an edge case with late `userData` arrival. |
| `canApprove` correctly checks `approvedRole !== null` | Pass | `if (!approvedRole) return false` at line 191. |
| `handleApprove` sends `approvedRole` in body | Pass | Line 270 sends `approved_role: approvedRole ?? reviewData.requested_role`. The fallback is dead code (finding 3). |
| Component tests: 2 options for power_user request | Pass | Test at line 1152 verifies both `scope_user_power_user` and `scope_user_user` options present. |
| Component tests: 1 option for resource_user | Pass | Test at line 1176 verifies only `scope_user_user` present, `scope_user_power_user` absent. |
| Component tests: submit sends downgraded role | Pass | Test at line 1220 verifies `body.approved_role === 'scope_user_user'` when downgraded. |
| POM patterns followed | Pass | `AccessRequestReviewPage.mjs` extends `BasePage`, uses selectors object, exposes semantic methods. |
| `auth-server-client.mjs` properly updated for scope removal | Mostly pass | `exchangeToken` no longer includes scope claims in params. `scope` field still returned from `createResourceClient` (finding 7). |
| E2E specs no longer reference `scope_user_*` in OAuth exchange scopes | Pass | Verified in `oauth2-token-exchange.spec.mjs`, `oauth-chat-streaming.spec.mjs`, `mcps-auth-restrictions.spec.mjs`, `mcps-oauth-auth.spec.mjs`. None pass `scope_user_*` in the OAuth exchange scope parameter. |
| `ConfigSection.mjs` has methods for `requested_role` selection | Pass | `setRequestedRole(value)` uses `page.selectOption` on `select-requested-role`. |
| No hardcoded timeouts (except ChatPage) | Pass | No `page.waitForTimeout()` or inline `setTimeout()` found in any reviewed file. |
| Missing E2E spec for role downgrade flow | MISSING | Confirmed important gap — page object infrastructure exists but no spec exercises it. |

---

## Summary of Action Items

| Priority | Item |
|---|---|
| Important | Add E2E spec for role downgrade: submit access request with `scope_user_power_user`, approve with downgraded `scope_user_user`, verify resulting token role. |
| Minor | Remove redundant `data-testid` from `<Select>` wrapper (keep only on `<SelectTrigger>`). |
| Minor | Remove or update dead `getResourceScope()` method in `ConfigSection.mjs` since `ScopeDisplay` was deleted. |
| Minor | Remove fallback `?? reviewData.requested_role` in `handleApprove` — it is unreachable dead code. |
| Minor | Clean up `scope` field from `createResourceClient` return value if scope is no longer part of the resource client creation response. |
| Low | Consider `ApproveAccessRequestBody` type for inclusion in `@bodhiapp/ts-client` OpenAPI-generated types. |
| Low | Add UI message when `roleOptions` is empty on a draft request (silent disable with no explanation). |
