# Frontend Tests Review

## Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx` | 260 | MCP server creation with optional OAuth auto-DCR |
| `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx` | 563 | Server view page with auth config CRUD management |
| `crates/bodhi/src/app/ui/mcps/oauth/callback/page.test.tsx` | 220 | OAuth callback page: state validation, code exchange, redirect |
| `crates/bodhi/src/app/ui/mcps/new/page.test.tsx` | 945 | MCP create/edit with auth config dropdown, OAuth connect/disconnect, session restore |
| `crates/bodhi/src/lib/urlUtils.test.ts` | 40 | `extractSecondLevelDomain` utility for auto-naming |
| `crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts` | 457 | MSW v2 handler factories and mock data for MCP endpoints |

## Test Coverage Summary

| Critical Flow | Tested? | Test Location | Notes |
|---|---|---|---|
| Auto-DCR success on OAuth selection (new server page) | Yes | `new/page.test.tsx` line 36-105: "auto-triggers DCR and populates registration type dropdown on OAuth selection" | Good: verifies endpoint auto-population and Dynamic Registration dropdown selection |
| Auto-DCR success on OAuth selection (view page) | Yes | `view/page.test.tsx` line 415-473: "auto-triggers DCR and populates fields when OAuth is selected" | Same flow tested on the server view inline form |
| Silent fallback to Pre-Registered on DCR failure (new page) | Yes | `new/page.test.tsx` line 213-259: "silently switches to pre-registered on auto-DCR failure" | Uses `setTimeout` delay -- see finding #1 |
| Silent fallback to Pre-Registered on DCR failure (view page) | Yes | `view/page.test.tsx` line 475-518: "silently falls back to pre-registered on auto-DCR failure" | Uses `setTimeout` delay -- see finding #1 |
| Config name auto-update on type switch | Yes | `view/page.test.tsx` line 520-562: "updates name field when switching from header to oauth auth type" | Verifies `header-default` changes to `oauth-default` |
| Config name preserved on custom edit | **No** | -- | **Missing**: No test verifies that a user-typed custom name survives a type switch. See finding #2 |
| OAuth callback: success with code exchange + redirect | Yes | `callback/page.test.tsx` line 58-70: "exchanges token and redirects on success" | Covers create mode redirect to `/ui/mcps/new/` |
| OAuth callback: edit mode redirect (return_url) | Yes | `callback/page.test.tsx` line 72-96: "redirects to return_url when present (edit mode)" | |
| OAuth callback: error from provider | Yes | `callback/page.test.tsx` line 108-118: "shows error from OAuth provider" | |
| OAuth callback: missing code | Yes | `callback/page.test.tsx` line 130-139: "shows error when no authorization code provided" | |
| OAuth callback: missing state | Yes | `callback/page.test.tsx` line 151-160: "shows error when state parameter is missing" | |
| OAuth callback: corrupt session data | Yes | `callback/page.test.tsx` line 173-182: "shows error when session data is corrupt" | |
| OAuth callback: token exchange failure | Yes | `callback/page.test.tsx` line 210-219: "shows error when token exchange fails" | |
| Auth config CRUD on view page (list header) | Yes | `view/page.test.tsx` line 124-141 | |
| Auth config CRUD on view page (list OAuth) | Yes | `view/page.test.tsx` line 143-166 | |
| Auth config CRUD on view page (create header) | Yes | `view/page.test.tsx` line 237-266 | |
| Auth config CRUD on view page (delete with dialog) | Yes | `view/page.test.tsx` line 297-325 | |
| Auth config CRUD on view page (create error) | Yes | `view/page.test.tsx` line 268-295 | |
| Auth config CRUD on view page (delete error) | **No** | -- | `mockDeleteAuthConfigError` exists in handlers but never used in tests. See finding #3 |
| Auth config dropdown on MCP create page (public default) | Yes | `mcps/new/page.test.tsx` line 278-291 | |
| Auth config dropdown with type badges | Yes | `mcps/new/page.test.tsx` line 293-333 | Header and OAuth badges both tested |
| Auth config auto-select on server selection | Yes | `mcps/new/page.test.tsx` line 378-395 | |
| OAuth Connect button triggers login redirect | Yes | `mcps/new/page.test.tsx` line 868-891 | |
| OAuth Connected card displays in edit mode | Yes | `mcps/new/page.test.tsx` line 605-618 | |
| OAuth lazy disconnect flow | Yes | `mcps/new/page.test.tsx` line 833-852 | |
| OAuth disconnect + update deletes token | Yes | `mcps/new/page.test.tsx` line 894-944 | |
| Session restore after OAuth callback | Yes | `mcps/new/page.test.tsx` line 683-702 | |
| Session data priority over API data in edit mode | Yes | `mcps/new/page.test.tsx` line 706-746 | |
| Post-callback MCP creation with OAuth | Yes | `mcps/new/page.test.tsx` line 750-805 | |
| Submit with header auth_type and auth_uuid | Yes | `mcps/new/page.test.tsx` line 444-485 | Captures and validates request body |
| Admin "New Auth Config" option visibility | Yes | `mcps/new/page.test.tsx` line 397-418 | |
| Non-admin cannot see "New Auth Config" | Yes | `mcps/new/page.test.tsx` line 420-442 | |
| Create server with OAuth DCR config (save) | **Skipped** | `new/page.test.tsx` line 107-211 | `.skip` with TODO comment: timeout issue. See finding #4 |
| OAuth create flow on view page (OAuth config) | **No** | -- | Creating an OAuth auth config via the inline form on view page not tested. See finding #5 |

## Findings

### Finding 1: Inline setTimeout in test assertions violates project rule

- **Priority**: Critical
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx`
- **Location**: Line 249: `await new Promise((resolve) => setTimeout(resolve, 500));`
- **Also**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx` line 504
- **Issue**: The project rules in CLAUDE.md explicitly state: "do not add inline timeouts in component tests... this fix hides the actual issue." Both DCR-failure fallback tests use a 500ms `setTimeout` to wait for the auto-DCR mutation to fail before checking the Pre-Registered fallback state.
- **Recommendation**: Replace the `setTimeout` with a `waitFor` assertion that waits for the observable UI change (e.g., `await waitFor(() => { expect(registrationTypeSelect...).toContain('Pre-Registered'); })`). The view page test already partially does this by following the setTimeout with a `waitFor`, making the setTimeout redundant. The new page test uses `textContent` directly after the setTimeout without `waitFor`, compounding the issue.
- **Rationale**: Inline timeouts are flaky -- they may pass on fast machines and fail on slow CI. They also mask the real issue: the test should be waiting for a state change, not an arbitrary duration.

### Finding 2: Missing test for custom name preservation on type switch

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
- **Location**: After line 562 (the "updates name field when switching from header to oauth auth type" test)
- **Issue**: The source code in `AuthConfigForm.tsx` (lines 100-103) only auto-updates the name when it matches the default values (`header-default` or `oauth-default`) or is empty. If a user types a custom name like "My Custom Auth", switching types should preserve it. There is no test verifying this behavior.
- **Recommendation**: Add a test that: (1) opens the auth config form, (2) clears the auto-populated name and types a custom name, (3) switches from Header to OAuth, (4) asserts the custom name is preserved (not overwritten to `oauth-default`).
- **Rationale**: This is a core UX guarantee -- user edits should not be silently discarded. Without a test, a regression could easily slip in.

### Finding 3: Delete auth config error scenario not tested

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
- **Location**: After the "deletes an auth config via confirmation dialog" test (line 325)
- **Issue**: The MSW handler `mockDeleteAuthConfigError` is defined in `mcps.ts` (line 353-362) but never used in any test file. The create error case is tested (line 268-295), but the delete error case is not.
- **Recommendation**: Add a test that: (1) attempts to delete an auth config, (2) uses `mockDeleteAuthConfigError`, (3) verifies the dialog stays open or shows an error message on failure.
- **Rationale**: Error flows should be tested for all CRUD operations. The create error test already follows this pattern; delete should be consistent.

### Finding 4: Skipped test with TODO and timeout issues

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx`
- **Location**: Line 107-211: `it.skip('creates MCP server with OAuth DCR config on save', ...)`
- **Issue**: This test is skipped with a TODO comment: "This test is timing out - the DCR endpoint is being called successfully but createMcpServer is never called." Additionally, the test has two problems: (a) it uses `{ timeout: 5000 }` in a `waitFor` call (line 196), violating the inline timeout rule, and (b) it passes a callback function to `mockCreateMcpServer()` (line 132), but the handler factory only accepts a `McpServerResponse` object, not a callback. This means the handler would not work as intended.
- **Recommendation**: Fix the skipped test: (1) use the raw `http.post()` handler pattern (as done elsewhere, e.g., `mcps/new/page.test.tsx` line 448) instead of trying to pass a callback to `mockCreateMcpServer`, (2) remove the `{ timeout: 5000 }` from waitFor, (3) investigate why the mutation chain (DCR then create) does not complete.
- **Rationale**: This is the only test that covers the end-to-end "create server with OAuth DCR config" flow. Without it, the full save-with-DCR path has no automated coverage.

### Finding 5: No test for creating OAuth auth config via view page inline form

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.test.tsx`
- **Location**: After the "creates a header auth config via inline form" test (line 266)
- **Issue**: The test suite covers creating a header auth config via the inline form (line 237-266) but does not cover creating an OAuth auth config. The view page inline form has different fields and behavior for OAuth (registration type sub-dropdown, endpoint fields, optional DCR call). This is a distinct code path that is untested.
- **Recommendation**: Add a test that: (1) opens the inline form, (2) selects OAuth type, (3) fills in OAuth-specific fields (client_id, authorization_endpoint, token_endpoint, scopes), (4) clicks save, (5) verifies the form closes or the config appears in the list. Optionally add a variant for the DCR registration type that triggers standalone dynamic registration before save.
- **Rationale**: The OAuth auth config creation flow involves different form fields, different validation, and a different request payload (discriminated union with `type: "oauth"`). Without test coverage, regressions in the OAuth creation path would go undetected.

### Finding 6: Unused import in new/page.test.tsx

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx`
- **Location**: Line 5: `mockCreateMcpServerError`
- **Issue**: `mockCreateMcpServerError` is imported but (a) never used in any test and (b) does not exist as an export in `mcps.ts`. This will cause a build/import error.
- **Recommendation**: Remove the unused import. If a server creation error test is needed, define `mockCreateMcpServerError` in the handlers file first.
- **Rationale**: Dead imports that reference non-existent exports may cause compilation errors or confusion during maintenance.

### Finding 7: textContent assertion instead of testing-library matcher

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.test.tsx`
- **Location**: Line 254: `expect(registrationTypeSelect.textContent).toContain('Pre-Registered');`
- **Issue**: The test accesses `textContent` directly on the DOM element instead of using testing-library matchers like `toHaveTextContent('Pre-Registered')`. The project convention favors `getByTestId` with testing-library assertions over raw DOM access.
- **Recommendation**: Replace with `expect(registrationTypeSelect).toHaveTextContent('Pre-Registered');` which is more readable and consistent with the rest of the test suite.
- **Rationale**: Consistency with project testing conventions improves maintainability.

### Finding 8: MSW default handler ordering discrepancy with documentation

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts`
- **Location**: Lines 444-449 in `mcpsHandlers` array
- **Issue**: The `05-frontend.md` documentation explicitly states: "MUST be registered before generic auth-configs handlers" for `mockOAuthLogin` and `mockOAuthTokenExchange`. However, in the default `mcpsHandlers` array, `mockListAuthConfigs()` (line 444) and `mockCreateAuthConfig()` (line 445) are registered BEFORE `mockOAuthLogin()` (line 448) and `mockOAuthTokenExchange()` (line 449). While this may not cause issues because MSW v2 differentiates paths by full pattern matching (the sub-paths `/mcps/auth-configs/:id/login` and `/mcps/auth-configs/:id/token` are distinct from `/mcps/auth-configs`), the ordering contradicts the documented requirement and the comment on lines 428-430 about sub-path handlers first.
- **Recommendation**: Move `mockOAuthLogin()` and `mockOAuthTokenExchange()` before `mockListAuthConfigs()` and `mockCreateAuthConfig()` in the default handler array to match the documented requirement and eliminate any risk of MSW matching ambiguity.
- **Rationale**: Code should match its documentation. Even if the current MSW behavior handles it correctly, future MSW updates or refactors could introduce subtle matching bugs.

### Finding 9: No error handler for MCP server creation in handler factories

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts`
- **Location**: After `mockCreateMcpServer` (line 198)
- **Issue**: The handlers file provides error variants for MCPs (`mockCreateMcpError`), auth configs (`mockCreateAuthConfigError`, `mockDeleteAuthConfigError`), tools (`mockFetchMcpToolsError`), discovery (`mockDiscoverMcpError`), and tokens (`mockDeleteOAuthTokenError`), but there is no `mockCreateMcpServerError` function. The import in `new/page.test.tsx` references it, suggesting it was planned but never implemented.
- **Recommendation**: Add `mockCreateMcpServerError` to the handler factories for completeness. Also add a corresponding test in `new/page.test.tsx` for server creation failure.
- **Rationale**: Consistency in error handler coverage across all CRUD operations.

### Finding 10: Comprehensive OAuth callback test coverage is strong

- **Priority**: Positive observation
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.test.tsx`
- **Location**: Entire file (220 lines, 7 test cases across 5 describe blocks)
- **Issue**: No issue. The callback page tests thoroughly cover all edge cases: success with redirect, edit mode return_url, provider error, missing code, missing state, corrupt session data, and token exchange failure. Each scenario has proper session setup and MSW handler configuration.
- **Recommendation**: None.
- **Rationale**: This is well-structured test coverage for a critical security flow.

### Finding 11: MCP create/edit page has thorough OAuth flow coverage

- **Priority**: Positive observation
- **File**: `crates/bodhi/src/app/ui/mcps/new/page.test.tsx`
- **Location**: Entire file (945 lines, 20+ test cases across 10 describe blocks)
- **Issue**: No issue. The test file comprehensively covers: create flow, edit flow, auth config dropdown (badges, auto-select, admin vs non-admin), edit with public/header/OAuth/DCR auth types, OAuth session restore, session data priority, post-callback creation, lazy disconnect, OAuth connect redirect, and disconnect + update with token deletion.
- **Recommendation**: None.
- **Rationale**: This is thorough coverage of a complex multi-step feature.

## Summary

**Overall quality**: Good. The test suite covers the vast majority of critical OAuth flows with proper use of `data-testid`, `getByTestId`, `waitFor`, and MSW v2 handlers. The mock data factories and handler factories are well-organized and follow consistent patterns.

**Critical issues** (2):
1. Two inline `setTimeout` calls violate project rules and introduce flakiness
2. Skipped test for end-to-end DCR server creation with broken handler call

**Important gaps** (2):
1. No test for custom name preservation on type switch
2. No test for creating OAuth auth config via view page inline form

**Nice-to-have improvements** (5):
1. Missing delete auth config error test
2. Unused/broken import of non-existent `mockCreateMcpServerError`
3. `textContent` assertion instead of `toHaveTextContent`
4. Default handler ordering inconsistency with documentation
5. Missing `mockCreateMcpServerError` handler factory
