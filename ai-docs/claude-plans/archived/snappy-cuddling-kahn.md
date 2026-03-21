# Plan: Refactor chat-toolsets.spec.mjs - Focus on Core Logic

## Problem Analysis

**Root Cause**: 3 tests call `configureToolsetWithApiKey('builtin-exa-web-search', ...)` within same `test.describe` block, creating duplicate toolset instances → name conflicts/selector ambiguity.

**Reference Pattern**: `model-metadata.spec.mjs` - single comprehensive test with `test.step()`.

## Final Test Structure: 1 Test

Replace all 8 tests with single comprehensive flow test:

```javascript
test('complete flow: configure toolset → verify in popover → enable → check persistence @integration', async ({ page }) => {
  const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
  expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();

  await loginPage.performOAuthLogin();

  await test.step('Configure Exa Web Search toolset', async () => {
    await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);
  });

  await test.step('Verify toolset in popover and enable', async () => {
    await chatPage.navigateToChat();
    await chatPage.openToolsetsPopover();
    await chatPage.waitForToolsetsToLoad();
    await chatPage.expectToolsetInPopover('builtin-exa-web-search');
    await chatPage.enableToolset('builtin-exa-web-search');
    await chatPage.closeToolsetsPopover();
    await chatPage.expectToolsetBadgeVisible(4);
  });

  await test.step('Verify selection persists after reopening popover', async () => {
    await chatPage.openToolsetsPopover();
    await chatPage.expectToolsetCheckboxChecked('builtin-exa-web-search');
  });
});
```

## Summary

| Before | After |
|--------|-------|
| 8 tests | 1 test |
| 3x duplicate toolset creation | 1x toolset creation |
| Flaky loading/intermediate tests | Removed |

## Files to Modify

- `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-toolsets.spec.mjs`

## Verification

```bash
cd crates/lib_bodhiserver_napi && npm run test -- --grep "Chat Interface - Toolsets Integration"
```
