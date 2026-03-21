# Fix e2e test failures: editLocalModel uses alias instead of UUID

## Context

After migrating UserAlias from YAML files to SQLite, the edit page URL changed from `?alias=<name>` to `?id=<uuid>`. The UI (`page.tsx` line 163) already routes with `id=${model.id}`, and the edit page (`edit/page.tsx` line 13) already reads `id` from search params. However, the e2e test's `editLocalModel` method still asserts `searchParams.get('alias')` causing 2 test failures.

## Changes

### 1. Add `data-test-model-id` to table rows
**File**: `crates/bodhi/src/app/ui/models/page.tsx`

Add `getRowProps` to the `DataTable` component call (around line 558) to emit a `data-test-model-id` attribute on each `<tr>`:

```tsx
getRowProps={(model: AliasResponse) => ({
  'data-test-model-id': isUserAlias(model) ? model.id : isApiAlias(model) ? model.id : undefined,
})}
```

Only `UserAliasResponse` and `ApiAliasResponse` have `id` (UUID). `ModelAliasResponse` (source='model') does not.

### 2. Update `editLocalModel` in test page object
**File**: `crates/lib_bodhiserver_napi/tests-js/pages/ModelsListPage.mjs`

Update `editLocalModel(alias)` (line 168-178) to:
1. Find the alias cell using existing selector `localAliasCell(alias)`
2. Navigate to parent `<tr>` to get `data-test-model-id` attribute (the UUID)
3. Click the edit button (unchanged)
4. Assert `searchParams.get('id')` equals the extracted UUID (instead of `searchParams.get('alias')`)

```js
async editLocalModel(alias) {
    // Get model UUID from the row's data-test-model-id attribute
    const aliasCell = this.page.locator(this.selectors.localAliasCell(alias));
    await expect(aliasCell).toBeVisible();
    const row = aliasCell.locator('xpath=ancestor::tr');
    const modelId = await row.getAttribute('data-test-model-id');

    const editBtn = this.page.locator(this.selectors.editButton(alias));
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await this.waitForUrl('/ui/models/edit/');
    await this.waitForSPAReady();

    // Verify we're on the edit page with correct model UUID
    const currentUrl = new URL(this.page.url());
    expect(currentUrl.searchParams.get('id')).toBe(modelId);
}
```

### 3. Build UI
```bash
make build.ui-rebuild
```

### 4. Run failing tests
```bash
cd crates/lib_bodhiserver_napi && npx playwright test tests-js/specs/models/model-alias.spec.mjs
```

## Verification
- Both e2e tests in `model-alias.spec.mjs` should pass
- The edit page loads correctly with the UUID-based URL
