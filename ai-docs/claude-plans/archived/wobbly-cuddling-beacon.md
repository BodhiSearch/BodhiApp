# E2E Test Update for Toolset Multi-Instance Architecture

## Summary

Update Playwright E2E tests to work with the new UUID-based multi-instance toolset architecture. Tests currently fail because they use old type-based selectors (`builtin-exa-web-search`) and old API endpoints (`/toolsets/{id}/config`).

## Key Architecture Changes

| Aspect | Old | New |
|--------|-----|-----|
| Toolset ID | Type ID (`builtin-exa-web-search`) | UUID |
| Edit URL | `/ui/toolsets/edit?id={type}` | `/ui/toolsets/edit?id={uuid}` |
| API endpoints | `/toolsets/{id}/config` | `/toolsets/{id}` |
| Admin toggle | On edit page | Separate `/ui/toolsets/admin` page |
| Chat popover | Flat list | Grouped by `toolset_type` |
| Create flow | Auto-created on first config | Explicit via `/ui/toolsets/new` |

## Test Flow Change

**Old Flow**: Navigate to edit page directly using type ID
**New Flow**: Create toolset via `/ui/toolsets/new` → Get UUID from list/API → Navigate to edit

---

## Phase page-objects: Update Page Object Selectors

### File: `crates/lib_bodhiserver_napi/tests-js/pages/ToolsetsPage.mjs`

**Current problematic code:**
```javascript
// Line 59: Uses wrong query param name
async navigateToToolsetEdit(toolsetId) {
  await this.navigate(`/ui/toolsets/edit?toolset_id=${toolsetId}`);  // WRONG
}
```

**Selector updates:**

| Old Selector | New Selector | Notes |
|--------------|--------------|-------|
| `toolset-edit-page` | `edit-toolset-page` | Container changed |
| `toolset-config-form` | Remove | Different form structure |
| `save-toolset-config` | `toolset-save-button` | Button name changed |
| `app-enabled-toggle` | Remove from edit page | Now on admin page |
| `toolset-name-{type}` | `toolset-name-{uuid}` + `data-testid-type="{type}"` | Add type attribute |
| `toolset-edit-button-{type}` | `toolset-edit-button-{uuid}` + `data-testid-type="{type}"` | Add type attribute |

**URL change:**
```javascript
// Old: /ui/toolsets/edit?toolset_id=builtin-exa-web-search
// New: /ui/toolsets/edit?id={uuid}
```

**New methods to add:**
- `navigateToNewToolset()` - Navigate to `/ui/toolsets/new`
- `createToolset(type, name, apiKey)` - Fill and submit new toolset form
- `getToolsetRowByType(type)` - Find row using `data-testid-type`
- `clickEditByType(type)` - Click edit using type selector
- `navigateToAdmin()` - Navigate to `/ui/toolsets/admin`
- `enableToolsetTypeOnAdmin(typeId)` - Navigate to admin, enable type

**Rewrite `configureToolsetWithApiKey` method:**
```javascript
async configureToolsetWithApiKey(toolsetType, apiKey, toolsetName = null) {
  // Step 1: Ensure type is enabled on admin page
  await this.navigateToAdmin();
  const typeToggle = this.page.locator(`[data-testid="type-toggle-${toolsetType}"]`);
  const isEnabled = await typeToggle.getAttribute('data-state');
  if (isEnabled !== 'checked') {
    await typeToggle.click();
    // Confirm enable dialog
    await this.page.click('button:has-text("Enable")');
  }

  // Step 2: Create new toolset
  await this.navigateToNewToolset();
  await this.page.selectOption('[data-testid="toolset-type-select"]', toolsetType);
  const name = toolsetName || toolsetType;
  await this.page.fill('[data-testid="toolset-name-input"]', name);
  await this.page.fill('[data-testid="toolset-api-key-input"]', apiKey);
  await this.page.click('[data-testid="toolset-create-button"]');

  // Wait for redirect to list page
  await this.page.waitForURL(/\/ui\/toolsets(?!\/new)/);
}
```

### File: `crates/lib_bodhiserver_napi/tests-js/pages/SetupToolsetsPage.mjs`

**Selector updates:**

| Old Selector | New Selector | Notes |
|--------------|--------------|-------|
| `toolset-config-form` | `setup-toolset-form` | Form ID changed |
| `save-toolset-config` | `create-toolset-button` | Now creates, not saves |
| `enabledBadge: 'text=Enabled'` | Scope to form | Avoid matching toast |

**Test ID attributes in UI (verified):**
- `toolsets-setup-page` - Page container
- `setup-toolset-form` - Form wrapper
- `app-enabled-toggle` - Server-level toggle
- `toolset-name-input` - Name input
- `toolset-description-input` - Description input
- `toolset-api-key-input` - API key input
- `toolset-enabled-toggle` - User-level toggle
- `create-toolset-button` - Submit button
- `skip-toolsets-setup` - Skip button

---

## Phase ui-testid: Add Type-Based Test IDs

### File: `crates/bodhi/src/app/ui/toolsets/page.tsx`

Add `data-testid-type` attribute to row elements:

```tsx
// On name cell
data-testid-type={toolset.toolset_type}

// On edit button
data-testid-type={toolset.toolset_type}
```

### File: `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`

Add `data-testid-type` attribute to toolset items:

```tsx
// On toolset-item-{id}
data-testid-type={toolset.toolset_type}
```

---

## Phase setup-tests: Fix Setup Toolsets Tests

### File: `crates/lib_bodhiserver_napi/tests-js/specs/setup/setup-toolsets.spec.mjs`

**Test: "displays correctly and can skip"**
- Update selectors to match new form structure
- `setup-toolset-form` instead of `toolset-config-form`

**Test: "configures Exa Web Search with API key"**
- Update form submission to use `create-toolset-button`
- Update success verification (form is for creating, not configuring)

**Verification:**
```bash
npx playwright test tests-js/specs/setup/setup-toolsets.spec.mjs
```

---

## Phase config-tests: Fix Toolsets Config Tests

### File: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-config.spec.mjs`

**Test: "displays toolsets list page"**
- Use `data-testid-type` selector to find Exa toolset row
- No toolset exists initially - test should create one first OR verify empty state

**Test: "navigates to toolset edit page from list"**
- Create toolset via UI first (`/ui/toolsets/new`)
- Use type-based selector to find and click edit button
- Verify `edit-toolset-page` loads

**Test: "displays toolset configuration form"**
- Create toolset first
- Navigate to edit
- Verify form fields: `toolset-name-input`, `toolset-api-key-input`, `toolset-enabled-switch`

**Test: "shows admin toggle for resource_admin users"**
- Navigate to `/ui/toolsets/admin` instead
- Verify `type-toggle-builtin-exa-web-search` visible

**Test: "shows confirmation dialog when toggling app enable"**
- Navigate to admin page
- Toggle `type-toggle-builtin-exa-web-search`
- Verify confirmation dialog appears

**Test: "configures toolset with real API key"**
- Create toolset via new page
- Verify success toast/redirect
- Navigate back to edit, verify form shows configured state

**Verification:**
```bash
npx playwright test tests-js/specs/toolsets/toolsets-config.spec.mjs
```

---

## Phase chat-tests: Fix Chat Toolsets Tests

### File: `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-toolsets.spec.mjs`

**Pre-condition for all tests:**
- Create toolset via API or UI first (toolset with API key configured)
- Navigate to chat page

**Test: "toolsets popover displays Exa Web Search toolset"**
- Update selector: find by `data-testid-type="builtin-exa-web-search"` within popover
- Verify toolset appears in group

**Test: "unconfigured toolset checkbox is disabled with tooltip"**
- Create toolset WITHOUT API key
- Verify checkbox is disabled
- Verify tooltip shows reason

**Test: "toolset selection persists when popover is reopened"**
- Create toolset with API key
- Select toolset
- Close/reopen popover
- Verify selection persisted

**Test: "toolsets badge shows count when toolsets are enabled"**
- Create toolset with API key
- Enable tools
- Verify badge count

**Verification:**
```bash
npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs
```

---

## Phase auth-tests: Fix Auth Restrictions Tests

### File: `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`

**Key changes needed:**

1. **Old `/config` endpoints no longer exist:**
   | Old | New |
   |-----|-----|
   | `GET /toolsets/{id}/config` | `GET /toolsets/{id}` |
   | `PUT /toolsets/{id}/config` | `PUT /toolsets/{id}` |
   | `DELETE /toolsets/{id}/config` | `DELETE /toolsets/{id}` |

2. **TOOLSET_ID now needs to be UUID, not type ID:**
   ```javascript
   // Old: const TOOLSET_ID = 'builtin-exa-web-search';
   // New: Need to create toolset first and capture UUID
   ```

3. **Response field change:**
   ```javascript
   // Old: t.toolset_id === TOOLSET_ID
   // New: t.toolset_type === 'builtin-exa-web-search'
   ```

**Auth model per routes.rs:**
- CRUD (`/toolsets`, `/toolsets/{id}`) → Session only (no API tokens, no OAuth)
- Execute (`/toolsets/{id}/execute/{method}`) → Session + OAuth with `toolset_auth_middleware`
- Types Admin (`/toolset_types/*`) → Admin session only

**Tests to update:**

| Test | Change |
|------|--------|
| GET /toolsets/{id}/config with API token | Change to `/toolsets/{uuid}`, expect 401 |
| PUT /toolsets/{id}/config with API token | Change to `/toolsets/{uuid}`, expect 401 |
| DELETE /toolsets/{id}/config with API token | Change to `/toolsets/{uuid}`, expect 401 |
| GET /toolsets/{id}/config with OAuth token | Change to `/toolsets/{uuid}`, expect 401 (session-only) |
| PUT /toolsets/{id}/config with OAuth token | Change to `/toolsets/{uuid}`, expect 401 |
| App WITH scope + OAuth WITH scope | Create toolset first, use UUID for execute |

**Test: "App WITH toolset scope + OAuth WITH scope returns toolset" (lines 224-351)**

Key changes:
```javascript
// Step 1: configureToolsetWithApiKey now creates toolset via UI
await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);

// Step 2: After list call, find toolset by type and extract UUID
const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {...});
const data = await response.json();
const exaToolset = data.toolsets.find(t => t.toolset_type === 'builtin-exa-web-search');
expect(exaToolset).toBeDefined();
const toolsetUuid = exaToolset.id;

// Step 3: Use UUID for execute
const executeResponse = await fetch(
  `${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}/execute/search`,
  {...}
);
```

**Verification:**
```bash
npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs
```

---

## Phase agentic-test: Fix Agentic Chat Test

### File: `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-agentic.spec.mjs`

**Test: "agentic chat with Exa web search executes tool"**
- Update `configureToolsetWithApiKey` method in ToolsetsPage
- Create toolset via `/ui/toolsets/new` instead of configuring existing
- Navigate to chat
- Use updated popover selectors

**Verification:**
```bash
npx playwright test tests-js/specs/chat/chat-agentic.spec.mjs
```

---

## Phase multi-instance-test: Add Multiple Toolsets Test

### File: `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-toolsets.spec.mjs`

**New Test: "multiple toolsets of same type are grouped in popover"**

```javascript
test('multiple toolsets of same type are grouped in popover', async ({ page }) => {
  // Setup: Create 2 toolsets of same type
  const toolsetsPage = new ToolsetsPage(page);
  await toolsetsPage.navigateToNewToolset();
  await toolsetsPage.createToolset('builtin-exa-web-search', 'exa-work', 'test-api-key-1');

  await toolsetsPage.navigateToNewToolset();
  await toolsetsPage.createToolset('builtin-exa-web-search', 'exa-personal', 'test-api-key-2');

  // Navigate to chat
  const chatPage = new ChatPage(page);
  await chatPage.navigate();

  // Open toolsets popover
  await page.click('[data-testid="toolsets-popover-trigger"]');

  // Verify group header exists for the type
  await expect(page.locator('[data-testid="toolset-group-builtin-exa-web-search"]')).toBeVisible();

  // Verify both toolsets appear in the group
  const group = page.locator('[data-testid="toolset-group-builtin-exa-web-search"]');
  await expect(group.locator('[data-testid-type="builtin-exa-web-search"]')).toHaveCount(2);

  // Verify expand/collapse works
  await page.click('[data-testid="toolset-group-toggle-builtin-exa-web-search"]');
  // Verify toolsets are visible/hidden
});
```

**Verification:**
```bash
npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs --grep "multiple toolsets"
```

---

## Implementation Order

| Phase | Tests | Verification Command |
|-------|-------|---------------------|
| page-objects | Update page object files | N/A (no direct test) |
| ui-testid | Add `data-testid-type` to UI | `make build.ui-clean && make build.ui` |
| setup-tests | setup-toolsets.spec.mjs | `npx playwright test tests-js/specs/setup/setup-toolsets.spec.mjs` |
| config-tests | toolsets-config.spec.mjs | `npx playwright test tests-js/specs/toolsets/toolsets-config.spec.mjs` |
| chat-tests | chat-toolsets.spec.mjs | `npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs` |
| auth-tests | toolsets-auth-restrictions.spec.mjs | `npx playwright test tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` |
| agentic-test | chat-agentic.spec.mjs | `npx playwright test tests-js/specs/chat/chat-agentic.spec.mjs` |
| multi-instance-test | New test in chat-toolsets.spec.mjs | `npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs --grep "multiple toolsets"` |

---

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/bodhi/src/app/ui/toolsets/page.tsx` | Add `data-testid-type` attributes |
| `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` | Add `data-testid-type` attributes |
| `tests-js/pages/ToolsetsPage.mjs` | Update selectors, add new methods |
| `tests-js/pages/SetupToolsetsPage.mjs` | Update selectors |
| `tests-js/specs/setup/setup-toolsets.spec.mjs` | Update for new form structure |
| `tests-js/specs/toolsets/toolsets-config.spec.mjs` | Update test flows |
| `tests-js/specs/chat/chat-toolsets.spec.mjs` | Update selectors, add multi-instance test |
| `tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs` | Update endpoints |
| `tests-js/specs/chat/chat-agentic.spec.mjs` | Update toolset setup flow |

---

## Final Verification

After all phases complete:

```bash
# Run all toolset-related tests
npx playwright test tests-js/specs/toolsets/
npx playwright test tests-js/specs/setup/setup-toolsets.spec.mjs
npx playwright test tests-js/specs/chat/chat-toolsets.spec.mjs
npx playwright test tests-js/specs/chat/chat-agentic.spec.mjs

# Full test suite
make test.ui
```
