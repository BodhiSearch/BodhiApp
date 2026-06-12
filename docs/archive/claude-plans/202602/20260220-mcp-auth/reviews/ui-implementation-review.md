# Frontend Implementation Review

## Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `crates/bodhi/src/hooks/useMcps.ts` | 521 | API hooks for MCP CRUD, auth configs, OAuth flow, token operations |
| `crates/bodhi/src/app/ui/mcps/new/page.tsx` | 816 | MCP instance create/edit page with OAuth connect/disconnect |
| `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx` | 357 | MCP server view with inline auth config management |
| `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx` | 320 | MCP server create with optional auth config |
| `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx` | 337 | Shared auth config form with auto-DCR and discovery |
| `crates/bodhi/src/stores/mcpFormStore.ts` | 106 | Zustand store with sessionStorage persistence for OAuth flow |
| `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx` | 156 | OAuth callback page: state validation, code exchange, redirect |
| `crates/bodhi/src/lib/mcpUtils.ts` | 24 | Auth config display utilities (badge text, variant, detail) |
| `crates/bodhi/src/lib/urlUtils.ts` | 38 | URL domain extraction utility |
| `ts-client/src/types/types.gen.ts` | (generated) | Verified: all MCP/OAuth types present and used by hooks |

## Findings

### Finding 1: `useDiscoverAs` hook is exported but never called from any component

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/hooks/useMcps.ts`
- **Location**: Lines 355-370 (`useDiscoverAs` function), also endpoint at line 78
- **Issue**: The `useDiscoverAs()` hook and `MCPS_OAUTH_DISCOVER_AS_ENDPOINT` are defined but grep across the entire `crates/bodhi/src/` directory shows the hook is only referenced in its own definition file. No component imports or calls it. The `OAuthDiscoverAsRequest` and `OAuthDiscoverAsResponse` types are imported from `@bodhiapp/ts-client` and re-exported but unused.
- **Recommendation**: Remove `useDiscoverAs`, `MCPS_OAUTH_DISCOVER_AS_ENDPOINT`, and the re-exports of `OAuthDiscoverAsRequest`/`OAuthDiscoverAsResponse` from `useMcps.ts`. This is dead code that increases maintenance burden. If a future feature needs AS-level discovery (separate from MCP discovery), it can be re-added.
- **Rationale**: Dead code obscures the actual API surface used by the frontend and can mislead developers into thinking this endpoint is actively consumed.

### Finding 2: `OAUTH_FORM_STORAGE_KEY` duplicated across files

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx` (line 13) and `crates/bodhi/src/stores/mcpFormStore.ts` (line 31)
- **Location**: Top-level constant definition
- **Issue**: The storage key `'mcp_oauth_form_state'` is defined as a separate `const` in both files. If one is changed without the other, sessionStorage state will silently fail to round-trip, breaking the OAuth redirect flow with no visible error.
- **Recommendation**: Export `OAUTH_FORM_STORAGE_KEY` from `mcpFormStore.ts` and import it in `callback/page.tsx`. This ensures a single source of truth.
- **Rationale**: DRY principle for a value where divergence causes a silent, hard-to-diagnose bug in the OAuth flow.

### Finding 3: `restoreFromSession` lacks try/catch around `JSON.parse`

- **Priority**: Important
- **File**: `crates/bodhi/src/stores/mcpFormStore.ts`
- **Location**: Lines 85-90 (`restoreFromSession` method)
- **Issue**: The `restoreFromSession` method calls `JSON.parse(saved)` without a try/catch block. If sessionStorage contains corrupt data (e.g., partial write from a crash, or manual tampering), this will throw an unhandled exception that propagates to the calling component. The callback page (lines 57-64) has its own try/catch around `JSON.parse`, but the `mcps/new/page.tsx` page calls `store.restoreFromSession()` at line 267 without any try/catch protection.
- **Recommendation**: Wrap `JSON.parse(saved)` in a try/catch within `restoreFromSession`, returning `null` on failure and removing the corrupt entry from sessionStorage. This provides defense-in-depth since the store is the authoritative session accessor.
- **Rationale**: Unhandled JSON parse errors in `restoreFromSession` will crash the MCP new/edit page after a corrupt session, leaving the user stuck without understanding why.

### Finding 4: `useStandaloneDynamicRegister` imported but unused in AuthConfigForm

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx`
- **Location**: Line 11
- **Issue**: `useStandaloneDynamicRegister` is imported from `@/hooks/useMcps` but never used within the component. The DCR calls are handled by the parent pages (`new/page.tsx` and `view/page.tsx`), not by the form component.
- **Recommendation**: Remove the unused import.
- **Rationale**: Unused imports increase bundle size (though tree-shaking may remove it) and create confusion about where DCR logic lives.

### Finding 5: View page passes `enableAutoDcr={true}`, contradicting design spec

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx`
- **Location**: Line 267
- **Issue**: The design spec states "new page = silent fallback; view page = error + retry." However, the view page passes `enableAutoDcr={true}` to `AuthConfigForm`, which gives it the same silent-fallback-on-first-failure behavior as the new page. The `AuthConfigForm` component has a separate code path for `!props.enableAutoDcr` (lines 120-126, labeled "Auto-discover on OAuth type selection (view page only)") that never executes because the view page sets `enableAutoDcr={true}`.
- **Recommendation**: Change the view page to pass `enableAutoDcr={false}` so it gets the "show error + retry" behavior. Verify the `!enableAutoDcr` code path in `AuthConfigForm` works correctly for the view page use case, particularly that discovery errors are displayed and the "Switch to Pre-Registered" button appears.
- **Rationale**: The view page is for server administrators who need to see and understand failures. Silent fallback hides actionable error information from admins.

### Finding 6: Duplicate `authConfigTypeBadge` function in `mcp-servers/page.tsx`

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/page.tsx`
- **Location**: Lines 39-44
- **Issue**: `authConfigTypeBadge` is defined locally in `mcp-servers/page.tsx` despite an identical exported version in `@/lib/mcpUtils.ts`. The view page and edit page correctly import from `mcpUtils.ts`, but the list page does not.
- **Recommendation**: Replace the local definition with `import { authConfigTypeBadge } from '@/lib/mcpUtils'`.
- **Rationale**: Code duplication; if behavior changes, the list page would diverge.

### Finding 7: Duplicate `getAuthConfigTypeBadge` in `mcps/new/page.tsx`

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcps/new/page.tsx`
- **Location**: Lines 143-152
- **Issue**: `getAuthConfigTypeBadge` is functionally identical to `authConfigTypeBadge` from `@/lib/mcpUtils.ts`, except it takes a `string` parameter instead of `McpAuthConfigResponse`. While the signature differs slightly (string vs discriminated union), the switch logic is the same. This is the third copy of this function.
- **Recommendation**: Either extend `authConfigTypeBadge` in `mcpUtils.ts` to accept both a config object and a plain string, or add a `getAuthTypeBadgeLabel(type: string): string` utility alongside it.
- **Rationale**: Three copies of the same mapping logic increases maintenance burden.

### Finding 8: Duplicate `AuthConfigType` and `OAuthRegistrationType` type aliases

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx` (lines 25-26) and `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx` (lines 13-14)
- **Location**: Top-level type definitions
- **Issue**: Both files define their own `AuthConfigType` and `OAuthRegistrationType` types. The new page version includes `'none'` in `AuthConfigType` while the form version does not. These are hand-rolled string literal unions that partially overlap with the `McpAuthType` type from `@bodhiapp/ts-client` (which is `'public' | 'header' | 'oauth'`).
- **Recommendation**: Extract these types to a shared location (e.g., `mcpUtils.ts` or a dedicated types file). Consider whether `McpAuthType` from ts-client can be used directly in more places.
- **Rationale**: Type duplication creates drift risk. The `'none'` vs no-`'none'` difference is already a divergence point.

### Finding 9: Substantial DCR submission logic duplicated between new and view server pages

- **Priority**: Important
- **File**: `crates/bodhi/src/app/ui/mcp-servers/new/page.tsx` (lines 115-154) and `crates/bodhi/src/app/ui/mcp-servers/view/page.tsx` (lines 107-161)
- **Location**: `handleSubmit` and `handleCreateSubmit` functions
- **Issue**: The DCR submission flow (check registration endpoint, call `standaloneDcr.mutateAsync`, extract results, compose `createAuthConfig.mutate` / `createMutation.mutate` payload with DCR fields) is nearly identical in both files. Both contain the same ~30 lines of DCR orchestration, the same field mapping (`dcrResult.client_id`, `dcrResult.client_secret`, etc.), and the same error handling pattern.
- **Recommendation**: Extract a shared `buildDcrAuthConfig` helper function (or custom hook) that takes DCR results and form fields, and returns the auth config payload. This reduces the surface area for bugs when the payload shape changes.
- **Rationale**: Both files must be updated in lockstep when the DCR response shape or auth config payload changes. A single helper eliminates this risk.

### Finding 10: `mcps/new/page.tsx` at 816 lines exceeds 500-line target

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcps/new/page.tsx`
- **Location**: Entire file
- **Issue**: At 816 lines, this file significantly exceeds the project's 500-700 line target. The file contains: the Zod schema, type definitions, `OAuthConnectedCard` sub-component, `getAuthConfigTypeBadge` utility, and the large `NewMcpPageContent` component with session restoration, auth config handling, OAuth connect/disconnect, tool fetching, and form submission logic.
- **Recommendation**: Consider extracting `OAuthConnectedCard` to a separate component file, moving `getAuthConfigTypeBadge` to `mcpUtils.ts`, and potentially extracting the OAuth connect/disconnect logic into a custom hook (e.g., `useOAuthConnection`). This would bring the file closer to the 500-line target.
- **Rationale**: Large files are harder to review, test, and modify safely.

### Finding 11: Callback page does not clear sessionStorage on error

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx`
- **Location**: Lines 29-93 (useEffect)
- **Issue**: When the token exchange fails (line 87-89 `onError`), the session data remains in sessionStorage. The user sees an error message with a "Back to form" button, but the stale session data (containing the old `selected_auth_config_id`, partial form values) persists. If the user navigates back and tries again, `restoreFromSession()` in the MCP new page reads and removes the stale data, which may contain an invalid `oauth_token_id` or inconsistent state.
- **Recommendation**: On token exchange error, either clear the session data or at minimum remove the stale `oauth_token_id` to prevent partial state from being restored.
- **Rationale**: Stale session data after a failed exchange could cause confusing behavior when the user retries the OAuth flow.

### Finding 12: `tokenExchangeMutation` missing from useEffect dependency array

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx`
- **Location**: Line 93 (`// eslint-disable-line react-hooks/exhaustive-deps`)
- **Issue**: The `useEffect` at line 29 calls `tokenExchangeMutation.mutate()` but `tokenExchangeMutation` is not in the dependency array. The eslint rule is suppressed. While this works because the mutation reference is stable (from react-query), the eslint suppression is a code smell that makes it harder to catch real dependency bugs.
- **Recommendation**: Add `tokenExchangeMutation` to the dependency array (it is stable) so the eslint suppression can be removed, or narrow the suppression to only the specific dependency that is intentionally omitted with a comment explaining why.
- **Rationale**: Blanket eslint-disable for exhaustive-deps hides real bugs and makes code review harder.

### Finding 13: OAuth callback redirect happens on success before user sees confirmation

- **Priority**: Nice-to-have
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx`
- **Location**: Lines 79-85 (onSuccess handler)
- **Issue**: On successful token exchange, the status is set to `'success'` and `router.push(returnUrl)` is called immediately in the same handler. The success UI (lines 113-118) flashes briefly before the redirect happens, or may not be visible at all depending on rendering timing. This is a minor UX concern -- the "Redirecting back to form..." message may never be seen.
- **Recommendation**: This is acceptable behavior. No action needed unless user feedback indicates confusion.
- **Rationale**: Informational only; the redirect is the desired outcome and a delay would slow the flow.

### Finding 14: State parameter has no TTL (acknowledged constraint)

- **Priority**: Informational
- **File**: `crates/bodhi/src/app/ui/mcps/oauth/callback/page.tsx` and `crates/bodhi/src/stores/mcpFormStore.ts`
- **Location**: N/A
- **Issue**: As noted in the review constraints, the CSRF `state` parameter embedded in the OAuth flow has no time-to-live enforcement on the frontend. The state is generated server-side and validated server-side during token exchange. SessionStorage naturally scopes the lifetime to the browser tab's session, which provides implicit (but not strict) TTL.
- **Recommendation**: No frontend action needed. If strict TTL is desired, it should be enforced server-side during the token exchange endpoint. The server could reject state parameters older than a configurable timeout.
- **Rationale**: Frontend TTL enforcement would be bypassable and adds complexity without meaningful security benefit.

### Finding 15: All types correctly sourced from `@bodhiapp/ts-client`

- **Priority**: Informational (positive finding)
- **File**: `crates/bodhi/src/hooks/useMcps.ts`
- **Location**: Lines 1-68
- **Issue**: No issue. All MCP-related types (`McpAuthConfigResponse`, `CreateAuthConfigBody`, `OAuthLoginRequest`, `OAuthTokenExchangeRequest`, `McpAuthType`, etc.) are imported from `@bodhiapp/ts-client` and re-exported through `useMcps.ts` for component consumption. No hand-rolled API types were found for the OAuth flow. The `AuthConfigType` / `OAuthRegistrationType` types in the form components are UI-local types (not API types) and their existence is acceptable.
- **Recommendation**: None.
- **Rationale**: Confirms compliance with the "all types from @bodhiapp/ts-client" constraint.

### Finding 16: Config name auto-update correctly implements `header-default` / `oauth-default`

- **Priority**: Informational (positive finding)
- **File**: `crates/bodhi/src/app/ui/mcp-servers/components/AuthConfigForm.tsx`
- **Location**: Lines 98-106
- **Issue**: No issue. The `useEffect` correctly auto-populates the config name: switching to `header` sets `header-default` (replacing `oauth-default` if present), and switching to `oauth` sets `oauth-default` (replacing `header-default` if present). Empty names are also populated. The logic preserves custom names that are neither default value.
- **Recommendation**: None.
- **Rationale**: Confirms compliance with the naming convention constraint.

## Summary

### Critical Issues: 0

### Important Issues: 3
- Finding 2: Duplicated `OAUTH_FORM_STORAGE_KEY` (silent breakage risk)
- Finding 3: Missing try/catch in `restoreFromSession` (crash on corrupt data)
- Finding 5: View page auto-DCR behavior contradicts design spec (admin UX)

### Nice-to-have Issues: 7
- Finding 1: Dead `useDiscoverAs` hook
- Finding 4: Unused `useStandaloneDynamicRegister` import in AuthConfigForm
- Finding 6: Duplicate `authConfigTypeBadge` in mcp-servers/page.tsx
- Finding 7: Duplicate `getAuthConfigTypeBadge` in mcps/new/page.tsx
- Finding 8: Duplicate type aliases across files
- Finding 10: mcps/new/page.tsx at 816 lines
- Finding 11: Callback page does not clear sessionStorage on error

### Duplication Worth Extracting: 1
- Finding 9: DCR submission logic duplicated between new and view server pages

### Positive Findings: 3
- Finding 14: State parameter TTL is a known accepted limitation
- Finding 15: All API types correctly from ts-client
- Finding 16: Config name auto-update works as designed
