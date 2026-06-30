# UI + ts-client Review

Ultracode re-review (Sonnet workflow) of diff range `4dea5ea9..HEAD` — "tokens screen-v2 migration + App Token grants" effort. Findings below survived adversarial verification (refute-by-default); each carries a verdict (`confirmed` = defect traced in committed code; `plausible` = likely real, severity/reachability not fully confirmed). Review only — no source modified.

## Summary
- Findings in this layer: 12 (Critical: 0, Important: 4, Nice-to-have: 8)

## Findings

### F16: grantableModelItems misclassifies ModelRouterResponse aliases as 'local' in the access picker
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: correctness
- **File**: `crates/bodhi/src/lib/grantItems.ts`
- **Location**: grantableModelItems(), lines 12-22
- **Issue**: The structural narrowing else if ('alias' in alias) matches not only UserAliasResponse and ModelAliasResponse (real local GGUF aliases) but also ModelRouterResponse (a composite routing alias), because ModelRouterResponse also carries an alias string field. ModelRouterResponse items are assigned type: 'local' and land in the 'Local Models' group in the picker panel with a 'local' badge.
- **Failure scenario**: A user creates a ModelRouter alias 'my-router'. On the New Token or review-consent screen, the model picker panel shows 'my-router' under 'Local Models' with a 'local' badge alongside real local-file aliases. If the user applies a 'Local only' type filter, 'my-router' incorrectly appears. If they apply 'API only', it is correctly excluded — but it is a valid grant target they would miss. The grant ID (alias.alias) sent in the payload is still correct; enforcement works. The bug is display-only but misleads user intent.
- **Recommendation**: Add a structural guard before the 'alias' branch to detect ModelRouterResponse (e.g. 'targets' in alias && 'strategy' in alias) and emit an item with no type field, so it renders in the untyped/ungrouped bucket rather than 'Local Models'. Alternatively use the source field if it is reliably present on all AliasResponse variants.
- **Rationale**: The models list endpoint includes all four AliasResponse variants. The check 'alias' in alias is too broad — it matches ModelRouterResponse, ModelAliasResponse, and UserAliasResponse alike. Only ApiAliasResponse is correctly discriminated.
- **Evidence**: Traced the full type hierarchy and rendering path. In ts-client/src/types/types.gen.ts line 1227-1235, ModelRouterResponse is defined with fields: source, id, alias: string, targets, strategy, created_at, updated_at. It has alias but NOT models or prefix. In grantItems.ts (new file, entirely in diff), the narrowing at line 12 (`'models' in alias && 'prefix' in alias`) only catches ApiAliasResponse; the fallthrough at line 18 (`else if ('alias' in alias)`) matches ModelRouterResponse, ModelAliasResponse, and UserAliasResponse alike — all receive `type: 'local'`. In AccessPickerPanel.tsx, `const local = filtered.filter((i) => i.type === 'local')` (line 53) lands ModelRouter items in the "Local Models" group (line 56), they receive a `local` badge (line 120), and with the "API only" type filter active, `i.type !== typeFilter` excludes them (line 45), making valid ModelRouter grant targets invisible to users who filter by API.
- **Verify notes**: The defect is real and fully traceable: ModelRouterResponse.alias satisfies 'alias' in alias, so it is misclassified as local. Enforcement is unaffected (the alias.alias value used as grant id is correct), but the picker mis-groups and, more importantly, the type filter silently hides ModelRouter aliases when "API only" is selected, causing users to miss them as valid grant targets. Priority "important" is correctly rated — it is a display+UX bug rather than a security or data-integrity flaw, but it can cause users to misconfigure grants by excluding valid options.
- **Sources**: arch:frontend, bug:frontend, prior:ui-review F7

### F17: Wrong CardTitle 'New App Token' on the API Token creation page
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: correctness
- **File**: `crates/bodhi/src/routes/tokens/new/index.tsx`
- **Location**: NewTokenContent, CardTitle, line 40
- **Issue**: The diff changed <CardTitle>New API Token</CardTitle> to <CardTitle>New App Token</CardTitle> on the /tokens/new/ route which creates a user-issued API Token (via TokenForm/useCreateToken). The breadcrumb on line 22 still correctly reads 'New API Token'. The page has contradictory labels. A genuinely separate 'App Tokens' management screen now exists at /tokens/apps/ for third-party app grants, making the conflation directly harmful to information architecture.
- **Failure scenario**: A user navigating to /tokens/new/ to create an API token sees the breadcrumb 'New API Token' but the card heading 'New App Token', confusing the two fundamentally different token types.
- **Recommendation**: Revert the CardTitle to 'New API Token' to match the breadcrumb and the route's purpose.
- **Rationale**: The title was correct before this diff. The change appears to be an accidental copy-paste or find-replace spillover from Phase 8 App Token UI work.
- **Evidence**: In `crates/bodhi/src/routes/tokens/new/index.tsx`:
- Line 22: `{ label: 'New API Token', current: true }` (breadcrumb, unchanged)
- Line 40: `<CardTitle>New App Token</CardTitle>` (changed in this diff)

The git diff `4dea5ea9..HEAD -- crates/bodhi/src/routes/tokens/new/index.tsx` confirms the CardTitle was deliberately changed from `New API Token` to `New App Token` while the breadcrumb constant was left as `'New API Token'`. The same page now presents two different labels for what is the same route and action.
- **Verify notes**: This is a real defect introduced in this diff. The `/tokens/new/` route creates user-issued API tokens (programmatic access); the new `/tokens/apps/` route manages third-party app grants. Renaming the CardTitle to "New App Token" on the API token creation page creates a direct contradiction with the breadcrumb on the same page and confuses it with the distinct App Tokens feature. Priority "important" is correct — it harms the information architecture of a security-sensitive feature where users must understand which token type they are creating.
- **Sources**: bug:frontend, prior:ui-review F1

### F18: grantItems.ts (grantableModelItems/grantableMcpItems) has no unit tests
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/lib/grantItems.ts`
- **Location**: entire file (28 lines)
- **Issue**: The shared bucketing logic in grantItems.ts — which maps AliasResponse variants to AccessItem grant IDs — has no test file. It is the single source of truth for what models/MCPs appear in the picker and under which type group. Edge cases like ModelRouterResponse classification (see F16), duplicate ID deduplication (Map), ApiAliasResponse with null prefix, and empty arrays are untested.
- **Failure scenario**: The ModelRouterResponse misclassification bug (F16) exists in production with no failing test to catch it. A regression in ApiAliasResponse ID construction (prefix + model.id) silently breaks grants for all users creating API-model-scoped tokens.
- **Recommendation**: Add src/lib/grantItems.test.ts covering: all four AliasResponse variant kinds; ModelRouterResponse mapped to a no-type or neutral-type item; duplicate IDs deduplicated; ApiAliasResponse with null prefix; empty MCP list; MCP item ID = name mapping.
- **Rationale**: grantItems.ts is called from both TokenForm and the review consent screen. A regression here silently breaks grants for all users on both surfaces. It is a high-leverage shared function.
- **Evidence**: grantItems.ts confirmed as a new file in the diff (git diff 4dea5ea9..HEAD shows it at new file mode 100644). grep over all *.test.* files in crates/bodhi/src finds zero references to grantableModelItems or grantableMcpItems. The consumer component test files (TokenForm.test.tsx, review/index.test.tsx) exercise only ModelAliasResponse fixtures via mockModelsDefault() → createMockModelAlias(). ModelRouterResponse type (from ts-client/src/types/types.gen.ts) has an `alias` field but no `models`/`prefix` fields, so it silently falls into the `else if ('alias' in alias)` branch and is classified as type:'local' — a real bug that would be caught by a unit test covering the ModelRouterResponse variant.
- **Verify notes**: The finding is accurate on all counts: the file is new, has no unit tests, and the structural narrowing logic (checking 'models' in alias && 'prefix' in alias vs 'alias' in alias) silently misclassifies ModelRouterResponse as a local model grant. The component-level tests use only ModelAliasResponse mocks, leaving ApiAliasResponse (null-prefix edge case, prefix concatenation), ModelRouterResponse classification, duplicate-ID deduplication, and empty-array paths all untested. Priority "important" is correct — this is new shared utility code on two grant surfaces, not a test gap on legacy untouched code.
- **Sources**: arch:frontend, test:frontend-e2e (grantItems no unit tests)

### F19: Consent approval payload test does not assert the mcps_list field
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/routes/apps/access-requests/review/index.test.tsx`
- **Location**: lines 1056-1069, 'approve payload carries the model + MCP grants' assertion block
- **Issue**: The test asserts models_list (line 1064), models_access (line 1066), mcps[0].status (line 1067), and mcps_access (line 1069) in the captured PUT body. It does NOT assert body.approved.mcps_list. The fixture mockDraftWithGrantFlagsResponse includes mcps_list: true in the requested flags; the review-list-mcps-toggle is rendered and clicked at line 1004. The resulting mcps_list value in the approved payload is never verified.
- **Failure scenario**: If toggleListMcps() in the consent review component had a bug that set mcps_list=false in the approved body even after the toggle is checked on, the test at line 1046 (clicking review-list-models-toggle) would succeed, the MCP select + approve flow would succeed, and the assertion block would pass — because mcps_list is absent from the assertions.
- **Recommendation**: Extend the assertion block to include expect(body.approved.mcps_list).toBe(true). Add a companion test where mcps_list is toggled OFF and verify the payload carries mcps_list: false. Mirror the existing models_list assertion pattern.
- **Rationale**: Both list toggles are symmetric UI controls. Asserting models_list but not mcps_list creates an asymmetric coverage hole where a regression on the MCP listing toggle would slip through the unit test layer entirely.
- **Evidence**: In `crates/bodhi/src/routes/apps/access-requests/review/index.test.tsx`, the "approve payload carries the model + MCP grants" test (line 1023): (1) never clicks `review-list-mcps-toggle` — only `review-list-models-toggle` is clicked at line 1046; (2) the TypeScript cast at lines 1056–1062 omits `mcps_list`; (3) assertions at lines 1064–1069 cover `models_list`, `models_access.type`, `mcps[0].status`, and `mcps_access`, but there is no assertion on `body.approved.mcps_list`. A grep over all `*.test.*` files confirms zero occurrences of `mcps_list` in any unit test. The fixture `mockDraftWithGrantFlagsResponse` (apps.ts:456) has `mcps_list: true` in `requested`. The component at index.tsx:275 computes `mcps_list: req.mcps_list ? listMcps : false`; since the toggle is never clicked, the test silently sends `mcps_list: false` and never verifies it. The E2E test in `app-tokens-grants.spec.mjs:71` exercises `listMcps: true` via `approveWithGrants` but only asserts model-access reflection, not MCP list.
- **Verify notes**: The finding's description of "clicked at line 1004" is inaccurate — line 1004 belongs to a separate test that only asserts the element is present in the DOM, not a click. The payload test never clicks the MCP list toggle at all, meaning the coverage gap is actually worse than described: not only is `mcps_list` unasserted, but the toggled-on case is never exercised in the unit test at all. Priority stays "important" because `models_list` and `mcps_list` are symmetric controls; the missing assertion is a real coverage hole where a regression on `toggleListMcps()` or its state wiring at index.tsx:275 would go undetected at the unit layer.
- **Sources**: test:frontend-e2e (mcps_list assertion)

### F15: Apps mutation hooks use inline error extraction instead of shared extractErrorMessage, dropping network-error details
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: api-consistency
- **File**: `crates/bodhi/src/hooks/apps/useAppAccessRequests.ts`
- **Location**: useRevokeAppAccess line 97, useDenyAppAccessRequest line 121, useApproveAppAccessRequest line 64
- **Issue**: The three new/modified mutations extract error messages inline (error?.response?.data?.error?.message || 'Failed to ...') instead of calling extractErrorMessage(error, 'Failed to ...') as the tokens hooks do. The inline version only reads the Bodhi API envelope. On a network failure (CORS block, timeout, server down) where error.response is undefined, the inline version swallows the underlying AxiosError message and always shows the hardcoded fallback.
- **Failure scenario**: User is on the App Tokens screen and the backend is unreachable. They click Revoke Access and confirm. Network call fails with AxiosError whose .message is 'Network Error'. The hook fires onError('Failed to revoke app access') instead of 'Network Error', hiding the real cause from the toast.
- **Recommendation**: Import and call extractErrorMessage(error, 'Failed to ...') in place of the three inline expressions, matching the pattern in useTokens.ts.
- **Rationale**: useTokens.ts already uses extractErrorMessage consistently. Diverging from the shared utility in newly added hooks increases the surface area where error messages are degraded.
- **Evidence**: 
git diff 4dea5ea9..HEAD confirms useRevokeAppAccess is newly added in this diff and uses inline extraction at line 97:
  `const message = error?.response?.data?.error?.message || 'Failed to revoke app access';`

The other two hooks (useApproveAppAccessRequest line 64, useDenyAppAccessRequest line 121) are pre-existing in the baseline commit — git show 4dea5ea9 shows both already had the inline pattern before this diff.

extractErrorMessage in /crates/bodhi/src/lib/errorUtils.ts lines 20-25:
```
export function extractErrorMessage(error: MaybeAxios, fallback: string): string {
  const enveloped = asBodhiError(error);
  if (enveloped) return enveloped.message || fallback;
  const raw = (error as { message?: string } | undefined)?.message;
  return raw || fallback;
}
```
The test at errorUtils.test.ts line 18-20 explicitly validates: `axiosError(undefined, 'Network Error')` → extractErrorMessage returns 'Network Error', not the fallback. The inline `error?.response?.data?.error?.message` is undefined when `error.response` is absent, so the expression evaluates directly to the hardcoded fallback, swallowing the AxiosError.message.

- **Verify notes**: The finding is confirmed but its scope is partially overstated: only useRevokeAppAccess (line 97) is new in this diff; useApproveAppAccessRequest and useDenyAppAccessRequest carried the same inline pattern before 4dea5ea9 and were not modified here. The behavior gap is real — on network failures the newly added hook returns 'Failed to revoke app access' instead of the underlying 'Network Error'. Priority is downgraded from important to nice-to-have because the error is still shown to the user (just less specifically), and this is a UX/debuggability issue with no correctness or security consequence.
- **Sources**: arch:frontend

### F34: AccessPickerPanel group labels 'Local Models' and 'API Models' hardcoded regardless of noun prop
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: architecture
- **File**: `crates/bodhi/src/components/access-picker/AccessPickerPanel.tsx`
- **Location**: groups memo, lines 51-59
- **Issue**: When items carry a type field, the slide-in panel groups them under the hardcoded labels 'Local Models' and 'API Models', ignoring the noun prop. The component is designed for sharing between models and MCPs (comment: 'Shared by models and MCPs'), but contains model-specific strings. Currently MCP items have no type so hasTypes is false and the labels are never shown for MCPs. Risk materialises when typed non-model items are added.
- **Recommendation**: Derive group labels from noun: 'Local ${noun}s' / 'API ${noun}s', or accept optional localGroupLabel/apiGroupLabel props defaulting to the current strings.
- **Rationale**: The component is explicitly designed for sharing but contains model-specific strings, creating a maintenance trap for future resource types.
- **Evidence**: AccessPickerPanel.tsx is a new file in this diff (git diff 4dea5ea9..HEAD shows `new file mode 100644`). Lines 56-57 read: `if (local.length) out.push({ label: 'Local Models', items: local });` and `if (api.length) out.push({ label: 'API Models', items: api });` — both hardcoded, ignoring the `noun` prop. Every other user-visible string in the component uses `noun`: line 52 (`${noun}s`), line 76 (`Search ${noun}s…`), line 99 (`No {noun}s match…`), line 132-133 (`{noun} selected`). The parent component AccessPicker.tsx line 29-30 has the JSDoc: "Shared by models and MCPs." TokenForm.tsx confirms both callers: `noun="model"` (line 151) and `noun="MCP"` (line 174). The latent-only safeguard is confirmed: `hasTypes = items.some((i) => i.type)` (line 40) is false for MCP items today (they carry no type field), so the hardcoded labels are never rendered currently.
- **Verify notes**: The finding's description of the failure mechanism is accurate. The component is explicitly designed for sharing between models and MCPs (parent's JSDoc) but bakes in model-specific group label strings rather than deriving from the noun prop. No current user-visible impact since MCP AccessItem objects have no type, keeping hasTypes false and the grouped code path unreachable for MCPs. Risk is latent and surfaces only when typed non-model items are introduced. Priority nice-to-have is correctly rated.
- **Sources**: arch:frontend

### F35: ReviewContent fetches models and MCPs unconditionally even when the app requested neither grant type
- **Priority**: nice-to-have  ·  **Verdict**: plausible  ·  **Category**: architecture
- **File**: `crates/bodhi/src/routes/apps/access-requests/review/index.tsx`
- **Location**: ReviewContent, lines 138-139
- **Issue**: useListModels and useListMcps are always called at component mount regardless of whether the access request includes models_access, models_list, mcps_access, or mcps_list flags. React-Query rules prevent conditional hook calls, but the enabled option could be set to false when the flags are absent. Two extra API calls fire for every consent review even if the GrantBlock sections are never rendered.
- **Recommendation**: Set enabled based on reviewData flags: useListModels(1, 100, 'alias', 'asc', undefined, { enabled: !!(reviewData && (req?.models_access || req?.models_list)) }). This requires waiting for reviewData before enabling, which React-Query supports natively.
- **Rationale**: Aligns data fetching with actual usage — only load the picker datasets when the review screen will actually show the pickers.
- **Sources**: arch:frontend

### F36: AppDetailPanel hardcodes 'active' status chip instead of reading app.status
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: correctness
- **File**: `crates/bodhi/src/routes/tokens/apps/index.tsx`
- **Location**: AppDetailPanel, line 254
- **Issue**: AppDetailPanel renders <span className="status-chip status-active">active</span> unconditionally. The AppAccessSummary type has a status field with possible values 'draft' | 'approved' | 'denied' | 'failed' | 'expired' | 'revoked'. The hardcoded text and CSS class ignore the actual value. The analogous TokenDetailPanel in routes/tokens/index.tsx correctly reads token.status from data.
- **Failure scenario**: Currently no runtime impact because list_approved_for_user filters to Status=Approved only. However if a revoked app appears in the list — e.g. through a cache-staleness window after a local state update, or if the backend adds support for showing recently-revoked items — the chip will always display 'active' regardless of app.status.
- **Recommendation**: Replace the hardcoded chip with: const isApproved = app.status === 'approved'; <span className={'status-chip ' + (isApproved ? 'status-active' : 'status-inactive')}>{app.status}</span>
- **Rationale**: AppAccessSummary.status is available and should be used. Hardcoding 'active' is a latent bug if the invariant that the list only contains approved items is ever relaxed.
- **Evidence**: git diff 4dea5ea9..HEAD shows the line was introduced in this diff: `+        <span className="status-chip status-active">active</span>` at AppDetailPanel in crates/bodhi/src/routes/tokens/apps/index.tsx line 254. AppAccessSummary.status is typed as 'draft' | 'approved' | 'denied' | 'failed' | 'expired' | 'revoked' (ts-client/src/types/types.gen.ts line 245) and is on the app prop. The backend list_approved_for_user (services/src/app_access_requests/access_request_repository.rs line 365) filters to Status=Approved only, so no runtime user impact today. The analogous TokenDetailPanel in routes/tokens/index.tsx lines 351/359 correctly reads token.status dynamically.
- **Verify notes**: Confirmed real defect introduced in this diff. The backend invariant (only Approved items returned) prevents user-visible impact currently, keeping priority at nice-to-have. The fix is trivial: replace the hardcoded chip with `const isApproved = app.status === 'approved'` and `<span className={'status-chip ' + (isApproved ? 'status-active' : 'status-inactive')}>{app.status}</span>`, mirroring TokenDetailPanel. Test fixtures already include mockAppAccessRevoked (test-fixtures/apps.ts line 38) demonstrating the type supports non-approved statuses in frontend test data.
- **Sources**: bug:frontend, prior:ui-review F2

### F37: ListingToggle keyboard activation (Space/Enter) not unit-tested
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/components/access-picker/ListingToggle.test.tsx`
- **Location**: line 25 — only click toggling is tested
- **Issue**: ListingToggle.tsx lines 41-47 implement an onKeyDown handler that fires onToggle for Space and Enter keys. The component is rendered as a div with role=checkbox and tabIndex=0, so keyboard interaction is the primary accessible path. The test file has 4 tests (render, click-toggle, redundant-hint, disabled-click-no-fire) but none covers keyboard activation.
- **Recommendation**: Add two unit tests: userEvent.keyboard(' ') on the toggle element → onToggle called; userEvent.keyboard('{Enter}') → onToggle called. Also add a disabled variant asserting neither key fires onToggle.
- **Rationale**: The component explicitly advertises keyboard accessibility (role=checkbox, tabIndex=0). Testing only click paths leaves the keyboard handler untested.
- **Evidence**: Both files are new in the diff (4dea5ea9..HEAD). ListingToggle.tsx lines 41-47 implement onKeyDown that fires onToggle for e.key === ' ' or e.key === 'Enter' when not disabled. The test file has exactly 4 tests (render, click-toggle, redundant-hint, disabled-click-no-fire); all interaction tests use userEvent.click only. There is no userEvent.keyboard or fireEvent.keyDown call anywhere in the test file.
- **Verify notes**: The keyboard handler code is correct as written (e.key === ' ' is the right DOM value for Space). The gap is purely test coverage: a future refactor of the key check (e.g. to 'Space' or 'Spacebar') would silently break accessibility without any test catching it. Priority nice-to-have is appropriate — this is a coverage gap for an a11y code path, not a functional bug in the shipped code.
- **Sources**: test:frontend-e2e, prior:ui-review F4

### F38: AccessPickerPanel type filter (Local/API dropdown) not unit-tested
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/components/access-picker/AccessPicker.test.tsx`
- **Location**: line 74 — search filter is tested; type filter select is not
- **Issue**: AccessPickerPanel.tsx lines 38-48 implement a typeFilter state and a <select> (data-testid='{prefix}-panel-type') that appears when any item has a type property. The ITEMS fixture in the test has mixed types (local: llama3:8b, mistral:7b; api: gpt-4o). The panel-type select is rendered but never exercised in the tests.
- **Recommendation**: Add a test that opens the panel, selects 'Local' from the type filter select, and verifies only local items appear (llama3:8b and mistral:7b visible; gpt-4o hidden). Add a companion test for the 'API' filter.
- **Rationale**: The type filter is the primary UX differentiator between the model picker (mixes local+API models) and the MCP picker (no types). It is new code in this diff with no coverage.
- **Evidence**: AccessPicker.test.tsx is a new file in this diff (git diff 4dea5ea9..HEAD confirms new file). It has 5 tests covering mode rendering, panel open, item select/remove, search filter, and count display. None of the tests interact with `model-access-panel-type`. AccessPickerPanel.tsx lines 38-48 implement `typeFilter` state and lines 82-93 render `<select data-testid="{prefix}-panel-type">` with All/Local/API options, conditional on `hasTypes`. The ITEMS fixture in the test (llama3:8b type=local, gpt-4o type=api, mistral:7b type=local) makes `hasTypes=true` for every test that opens the panel, so the select IS rendered — it's just never exercised. The filter predicate at line 45 (`if (hasTypes && typeFilter !== 'all' && i.type !== typeFilter) return false`) has no test coverage.
- **Verify notes**: The gap is real and in new code. The type filter is a primary UX differentiator for the model token form (mixing local+API models) vs. the MCP picker (no types). A regression swapping 'local'/'api' labels or breaking the filter predicate would pass the entire unit suite. Priority nice-to-have is correct — this is a missing coverage case on working code, not a production defect.
- **Sources**: test:frontend-e2e, prior:ui-review F4

### F39: GrantBlock component has no dedicated unit test
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/components/access-picker/GrantBlock.tsx`
- **Location**: line 40 — no corresponding .test.tsx
- **Issue**: GrantBlock.tsx (new in this diff) is a composition wrapper around ListingToggle + AccessPicker that supports showListing=false and showAccess=false to selectively render each half — the consent screen uses these to show only the controls the app requested. There is no GrantBlock.test.tsx. The showListing/showAccess props are exercised only indirectly through the consent screen unit test (review/index.test.tsx).
- **Recommendation**: Add GrantBlock.test.tsx covering: renders both halves by default; showListing=false hides the listing toggle but shows the picker; showAccess=false shows the toggle but hides the picker; disabled=true passes disabled to both child components. Keep it shallow (no MSW needed).
- **Rationale**: GrantBlock is the shared seam between two critical screens (token form and consent review). Testing its conditional rendering contract directly makes the component resilient to future props changes.
- **Evidence**: No GrantBlock.test.tsx exists in crates/bodhi/src/components/access-picker/ (only AccessPicker.test.tsx and ListingToggle.test.tsx are present). The consent review test (index.test.tsx) uses mockDraftWithGrantFlagsResponse which sets all four flags (models_list, models_access, mcps_list, mcps_access) to true, so only the showListing=true + showAccess=true combination is exercised through GrantBlock. The 'omits the grant sections' test uses mockDraftReviewResponse which prevents GrantBlock from rendering at all. No test exercises showListing=false with showAccess=true or vice versa — the mixed partial combinations that are the component's primary design purpose (consent screen shows only the controls the app requested).
- **Verify notes**: The conditional rendering logic in GrantBlock (lines 66 and 79) is simple JSX short-circuit guards. An inversion of the all-true path would be caught by the existing consent screen test (line 1001 asserts review-list-models-toggle is present). The uncaught gap is the partial combinations (showListing=false, showAccess=true etc.) which the consent screen always passes as derived from req.models_list/req.models_access — and no fixture supplies a mixed partial case. Priority nice-to-have is correctly rated: the bug risk is low given the trivial logic, but the direct unit test for the component's conditional-render contract is missing.
- **Sources**: test:frontend-e2e

### F40: TokenForm scope_token_power_user card disabled state not tested for resource_user role
- **Priority**: nice-to-have  ·  **Verdict**: confirmed  ·  **Category**: test-coverage
- **File**: `crates/bodhi/src/routes/tokens/-components/TokenForm.test.tsx`
- **Location**: line 99 — TokenForm describe block
- **Issue**: TokenForm.tsx:83 computes canPowerUser from userInfo.role and sets the Power User role card disabled when the user is resource_user (line 202). The test's beforeEach mounts mockUserLoggedIn({}, { stub: true }) which returns the default role. There is no test verifying that when userInfo.role is 'resource_user', the scope_token_power_user card has disabled=true and cannot be clicked to change scope.
- **Recommendation**: Add a test using mockUserLoggedIn({ role: 'resource_user' }) that asserts scope-card-scope_token_power_user has attribute disabled and clicking it does not change the selected scope. Add a complementary test for resource_power_user where both cards are enabled.
- **Rationale**: The canPowerUser guard is a client-side enforcement of a role boundary. Unit-testing the client guard ensures regressions are caught at the fastest feedback level.
- **Evidence**: TokenForm.tsx:83 computes `canPowerUser = userInfo?.auth_status === 'logged_in' && userInfo.role !== 'resource_user'`. TokenForm.tsx:202 uses `disabledCard = card.scope === 'scope_token_power_user' && !canPowerUser` and sets `disabled={disabledCard}` on the button (line 211). In TokenForm.test.tsx:106, the shared `beforeEach` calls `mockUserLoggedIn({}, { stub: true })`, which via user.ts:35 defaults `role` to `null` — making `canPowerUser` true in every rendered test. Searching the entire test file for `resource_user`, `resource_power_user`, `canPowerUser`, and `disabled` finds zero matches related to the scope card. The only appearance of `scope_token_power_user` in the test file is line 81, inside a `toCreateTokenRequest` data-transform unit test that does not render the component. No test mounts the form with `role: 'resource_user'` to assert the power-user card carries `disabled=true` and ignores clicks.
- **Verify notes**: The production guard is correctly implemented; this is purely a test coverage gap. The client-side guard is UX-only — the backend enforces the role boundary (the server rejects a `scope_token_power_user` token created by a `resource_user` caller), so a regression in this guard would not be a security issue. Priority "nice-to-have" is correctly rated.
- **Sources**: test:frontend-e2e
