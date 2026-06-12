# Plan: Rename `/ui/mcp-servers/` to `/ui/mcps/servers/` + Fix AuthConfigForm Inconsistency

## Context

The backend API was refactored to nest all MCP sub-resources under `/bodhi/v1/mcps/` (e.g., `/bodhi/v1/mcps/servers`, `/bodhi/v1/mcps/auth-configs`). The frontend routes still use the old flat structure (`/ui/mcp-servers/`), creating an inconsistency.

Additionally, the `AuthConfigForm` component renders differently on the "new server" page vs the "view server" page — the new page shows a Registration Type dropdown (Pre-Registered / Dynamic Registration) and auto-attempts DCR, while the view page hides the dropdown and shows all OAuth fields simultaneously. Both should behave identically.

**Intended outcome:** Frontend routes match backend nesting (`/ui/mcps/servers/`), and both auth config form locations show the same fields and behavior.

---

## Phase 1: Fix AuthConfigForm Inconsistency (do first, before directory move)

### 1.1 Remove `enableAutoDcr` prop from `AuthConfigForm`

**File:** `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx`

Remove the `enableAutoDcr` prop entirely. Make the component always use unified behavior:

| Line | Current | Change |
|------|---------|--------|
| 52 | `enableAutoDcr?: boolean;` | Remove from interface |
| 73 | `if (props.enableAutoDcr && data.registration_endpoint)` | `if (data.registration_endpoint)` |
| 85-93 | `if (props.enableAutoDcr && !autoDcrFailed)` | `if (!autoDcrFailed)` — always silent fallback on first failure |
| 107-116 | Auto-DCR effect guarded by `props.enableAutoDcr` | Remove `props.enableAutoDcr` guard — always auto-DCR |
| 118-125 | Separate "view page only" auto-discover effect | Remove entirely (merged into the auto-DCR effect above) |
| 128-141 | Manual retry guarded by `props.enableAutoDcr &&` | Remove `props.enableAutoDcr &&` from guard |
| 245 | `{props.enableAutoDcr && (` Registration Type selector | Always render — remove guard |
| 292 | `{(!props.enableAutoDcr \|\| props.registrationType === 'pre_registered') && (` | `{props.registrationType === 'pre_registered' && (` |
| 337 | `{(!props.enableAutoDcr \|\| props.registrationType === 'dynamic_registration') && (` | `{props.registrationType === 'dynamic_registration' && (` |

### 1.2 Update call sites

- `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx` line 275: Remove `enableAutoDcr={true}`
- `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx` line 263: Remove `enableAutoDcr={false}`

### 1.3 Update view/page.test.tsx

**File:** `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`

Tests to update for unified behavior:
- Tests that expect Client ID to appear immediately after selecting OAuth — now auto-DCR fires first, then silently falls back
- Tests that check OAuth field visibility — Registration Type dropdown is now always visible
- Tests that assert discovery error display — first failure is now silent (switches to Pre-Registered)
- Add test verifying Registration Type dropdown visible when OAuth selected on view page

### 1.4 Verify new/page.test.tsx

Run existing tests — should pass unchanged since they already tested with `enableAutoDcr={true}` behavior.

### 1.5 Add E2E test: OAuth DCR auth config via view page UI

**Gap:** All existing E2E tests create OAuth auth configs via API helpers (`createOAuthConfigViaApi`). No E2E test exercises the "Add Auth Config" form on the server view page. This is the exact UI that was inconsistent.

**Add page object methods to `McpsPage.mjs`:**

New selectors for server view page's inline auth config form (all already have `data-testid` in AuthConfigForm):
```js
// Server view page - auth config inline form
addAuthConfigButton: '[data-testid="add-auth-config-button"]',
authConfigForm: '[data-testid="auth-config-form"]',
authConfigTypeSelect: '[data-testid="auth-config-type-select"]',
authConfigNameInput: '[data-testid="auth-config-name-input"]',
oauthRegistrationTypeSelect: '[data-testid="oauth-registration-type-select"]',
authConfigAuthEndpointInput: '[data-testid="auth-config-auth-endpoint-input"]',
authConfigTokenEndpointInput: '[data-testid="auth-config-token-endpoint-input"]',
authConfigRegistrationEndpointInput: '[data-testid="auth-config-registration-endpoint-input"]',
authConfigScopesInput: '[data-testid="auth-config-scopes-input"]',
authConfigClientIdInput: '[data-testid="auth-config-client-id-input"]',
authConfigClientSecretInput: '[data-testid="auth-config-client-secret-input"]',
authConfigSaveButton: '[data-testid="auth-config-save-button"]',
authConfigCancelButton: '[data-testid="auth-config-cancel-button"]',
authConfigDiscoverStatus: '[data-testid="auth-config-discover-status"]',
```

New methods:
- `clickAddAuthConfig()` — click the "Add Auth Config" button on view page
- `clickViewServerById(id)` — click the view button on server list row
- `selectInlineAuthConfigType(type)` — select Header/OAuth from the inline form type dropdown
- `waitForDiscoveryComplete()` — wait for discovery spinner to disappear
- `expectRegistrationType(type)` — verify the Registration Type dropdown value
- `clickInlineAuthConfigSave()` — click Save on the inline form
- `expectAuthConfigRow(configName)` — verify auth config appears in the list

**Add E2E test in `mcps-oauth-dcr.spec.mjs`:**

New test: `'Add OAuth DCR auth config via server view page UI'`

Steps:
1. Login, create MCP server pointing to DCR test server (via `createMcpServer`)
2. Discover + DCR via API, create first OAuth auth config via API (reuse existing `setupDcrMcpInstance` helper partially)
3. Create MCP instance with that auth config, verify in playground (tools work)
4. Navigate back to MCP Servers page → click view on the server
5. Click "Add Auth Config" on view page
6. Select "OAuth" from type dropdown
7. Verify auto-discovery triggers (loading spinner, then endpoints auto-populated)
8. Verify Registration Type dropdown shows "Dynamic Registration"
9. Click Save → triggers DCR → auth config created
10. Verify new auth config appears in the auth configs list

### 1.6 Run unit tests
```bash
cd crates/bodhi && npm test -- --run src/app/ui/mcp-servers/
```

### 1.7 Build UI and run E2E tests
```bash
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright -- tests-js/specs/mcps/
```

### 1.8 Local commit
Commit Phase 1 changes before starting Phase 2 route rename.

---

## Phase 2: Route Rename (`/ui/mcp-servers/` → `/ui/mcps/servers/`)

### 2.1 Move directory
```bash
cd crates/bodhi/src/app/ui && git mv mcp-servers mcps/servers
```

Moves all files into `mcps/servers/` — internal relative imports (e.g., `../components/AuthConfigForm`) remain valid.

### 2.2 Update route constant

**File:** `crates/bodhi/src/lib/constants.ts` line 30
```
ROUTE_MCP_SERVERS = '/ui/mcp-servers'  →  ROUTE_MCP_SERVERS = '/ui/mcps/servers'
```
Keep constant name unchanged.

### 2.3 Fix McpManagementTabs

**File:** `crates/bodhi/src/components/McpManagementTabs.tsx`

**Critical:** After the change, `ROUTE_MCPS = '/ui/mcps'` and `ROUTE_MCP_SERVERS = '/ui/mcps/servers'`. The `isActive` function uses `pathname?.startsWith(href)`. When on `/ui/mcps/servers/new`, `startsWith('/ui/mcps')` is also true → both tabs would appear active.

Fix `isActive`:
```tsx
const isActive = (href: string) => {
  if (href === ROUTE_MCPS) {
    return pathname?.startsWith(href) && !pathname?.startsWith(ROUTE_MCP_SERVERS);
  }
  return pathname?.startsWith(href);
};
```

**data-testid preservation:** Currently `tab.href.split('/').pop()` → `'mcp-servers'` → `mcp-tab-mcp-servers`. After change: `'servers'` → `mcp-tab-servers` (BREAKS E2E). Fix by adding explicit `testId` field to tab config:
```tsx
const tabs = [
  { href: ROUTE_MCPS, label: 'My MCPs', ..., testId: 'mcp-tab-mcps' },
  { href: ROUTE_MCP_SERVERS, label: 'MCP Servers', ..., testId: 'mcp-tab-mcp-servers' },
];
// Then: data-testid={tab.testId}
```

### 2.4 Update navigation items

**File:** `crates/bodhi/src/hooks/use-navigation.tsx`

| Line | Current | New |
|------|---------|-----|
| 142 | `href: '/ui/mcp-servers/'` | `href: '/ui/mcps/servers/'` |
| 156 | `href: '/ui/mcp-servers/new/'` | `href: '/ui/mcps/servers/new/'` |
| 163 | `href: '/ui/mcp-servers/edit/'` | `href: '/ui/mcps/servers/edit/'` |
| 244 | `pathname?.startsWith('/ui/mcp-servers/') \|\| pathname?.startsWith('/ui/mcps/')` | `pathname?.startsWith('/ui/mcps/')` (subsumes mcp-servers check) |

### 2.5 Update hardcoded routes in moved page files

Replace hardcoded `/ui/mcp-servers` with `ROUTE_MCP_SERVERS` constant import.

**`mcps/servers/page.tsx`** (was `mcp-servers/page.tsx`):
- Links to `/ui/mcp-servers/view?id=...`, `/ui/mcp-servers/edit?id=...`, `/ui/mcp-servers/new` → use `ROUTE_MCP_SERVERS`

**`mcps/servers/new/page.tsx`** (was `mcp-servers/new/page.tsx`):
- Line 55: `router.push('/ui/mcp-servers')` → `router.push(ROUTE_MCP_SERVERS)`
- Line 290: Same

**`mcps/servers/view/page.tsx`** (was `mcp-servers/view/page.tsx`):
- Line 199: `href="/ui/mcp-servers/edit?id=..."` → use `ROUTE_MCP_SERVERS`

**`mcps/servers/edit/page.tsx`** (was `mcp-servers/edit/page.tsx`):
- Line 59: `router.push('/ui/mcp-servers/view?id=...')` → use `ROUTE_MCP_SERVERS`
- Line 261: `router.push('/ui/mcp-servers')` → `router.push(ROUTE_MCP_SERVERS)`

### 2.6 Update cross-references from existing mcps/ pages

**`crates/bodhi/src/app/ui/mcps/new/McpServerSelector.tsx`:**
- Links to `/ui/mcp-servers/new` → use `ROUTE_MCP_SERVERS`

### 2.7 Update unit test files (moved paths + expectations)

**`mcps/servers/new/page.test.tsx`:**
- Import path: `@/app/ui/mcp-servers/new/page` → `@/app/ui/mcps/servers/new/page`
- `usePathname`: `/ui/mcp-servers/new` → `/ui/mcps/servers/new`
- Route assertions: `/ui/mcp-servers` → `/ui/mcps/servers`

**`mcps/servers/view/page.test.tsx`:**
- Import path: `@/app/ui/mcp-servers/view/page` → `@/app/ui/mcps/servers/view/page`
- `usePathname`: `/ui/mcp-servers/view` → `/ui/mcps/servers/view`
- Edit link assertion: `/ui/mcp-servers/edit?id=...` → `/ui/mcps/servers/edit?id=...`

### 2.8 Update E2E page object

**File:** `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs`

| Line | Current | New |
|------|---------|-----|
| 5 | Comment: `/ui/mcp-servers` | `/ui/mcps/servers` |
| 118 | `navigate('/ui/mcp-servers/')` | `navigate('/ui/mcps/servers/')` |
| 123 | `waitForURL(/\/ui\/mcp-servers/)` | `waitForURL(/\/ui\/mcps\/servers/)` |
| 129 | `waitForURL(/\/ui\/mcp-servers\/new/)` | `waitForURL(/\/ui\/mcps\/servers\/new/)` |
| 151 | `waitForURL(/\/ui\/mcp-servers(?!\/new)/)` | `waitForURL(/\/ui\/mcps\/servers(?!\/new)/)` |

### 2.9 Create TECHDEBT.md

**File:** `crates/lib_bodhiserver_napi/tests-js/TECHDEBT.md`

Add note: Split `McpsPage.mjs` into separate `McpServersPage` and `McpInstancesPage` page objects.

### 2.10 Run all tests
```bash
cd crates/bodhi && npm test
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

---

## Files Summary

### Phase 1 (AuthConfigForm fix + E2E) — 6 files
1. `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx`
2. `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx`
3. `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx`
4. `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
5. `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs` — add server view auth config form methods
6. `crates/lib_bodhiserver_napi/tests-js/specs/mcps/mcps-oauth-dcr.spec.mjs` — add view page DCR test

### Phase 2 (Route rename) — 12 files + 1 new
5. `crates/bodhi/src/app/ui/mcp-servers/` → `crates/bodhi/src/app/ui/mcps/servers/` (directory move)
6. `crates/bodhi/src/lib/constants.ts`
7. `crates/bodhi/src/components/McpManagementTabs.tsx`
8. `crates/bodhi/src/hooks/use-navigation.tsx`
9. `crates/bodhi/src/app/ui/mcps/servers/page.tsx` (post-move)
10. `crates/bodhi/src/app/ui/mcps/servers/new/page.tsx` (post-move)
11. `crates/bodhi/src/app/ui/mcps/servers/view/page.tsx` (post-move)
12. `crates/bodhi/src/app/ui/mcps/servers/edit/page.tsx` (post-move)
13. `crates/bodhi/src/app/ui/mcps/new/McpServerSelector.tsx`
14. `crates/bodhi/src/app/ui/mcps/servers/new/page.test.tsx` (post-move)
15. `crates/bodhi/src/app/ui/mcps/servers/view/page.test.tsx` (post-move)
16. `crates/lib_bodhiserver_napi/tests-js/pages/McpsPage.mjs`
17. `crates/lib_bodhiserver_napi/tests-js/TECHDEBT.md` (NEW)

---

## Verification

1. `cd crates/bodhi && npm test` — all component tests pass
2. `make build.ui-rebuild` — UI builds successfully
3. `cd crates/lib_bodhiserver_napi && npm run test:playwright` — E2E tests pass
4. Manual smoke: `cd crates/bodhi && npm run dev` — navigate to `/ui/mcps/servers/`, verify:
   - Tabs work (My MCPs / MCP Servers)
   - Server list → new → view → edit flow
   - AuthConfigForm shows Registration Type on both new and view pages
   - OAuth auto-DCR works identically on both pages
