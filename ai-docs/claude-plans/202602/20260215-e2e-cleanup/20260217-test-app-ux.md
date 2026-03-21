# Plan: Test OAuth App UI/UX Polish

## Context

The test-oauth-app at `crates/lib_bodhiserver_napi/test-oauth-app/` is used by Playwright e2e tests in `tests-js/`. The current UI is functional but bare -- no header, grayscale colors, cramped form layout, plain-text JSON display, and no loading spinners. This plan improves the visual polish while keeping the app fast and e2e-test compatible (no animations, no heavy dependencies).

## Changes Summary

| # | Change | Files |
|---|--------|-------|
| 1 | Blue accent theme + code-block class | `src/styles/index.css` |
| 2 | New AppLayout with header bar | `src/components/AppLayout.tsx` (NEW) |
| 3 | Wire layout into App | `src/App.tsx`, `src/styles/index.css` (#root) |
| 4 | ConfigForm: grid layout, remove Reset, add spinner | `src/components/ConfigForm.tsx` |
| 5 | ScopeDisplay: badge/chip rendering | `src/components/ScopeDisplay.tsx` |
| 6 | DashboardPage: remove inline Logout, tighten layout | `src/pages/DashboardPage.tsx` |
| 7 | Code-block styling on all JSON pre blocks | `TokenDisplay.tsx`, `UserInfoSection.tsx`, `ToolsetsSection.tsx`, `RestClientSection.tsx` |
| 8 | Loading spinners on callback pages | `AccessCallbackPage.tsx`, `OAuthCallbackPage.tsx` |
| 9 | E2E page object update | `tests-js/pages/sections/DashboardSection.mjs` |

---

## Step 1: Theme Update -- `src/styles/index.css`

Update `@theme` block - replace grayscale primary with blue accent:

```
--color-primary: hsl(215 70% 45%);           /* Blue (was hsl(0 0% 15%)) */
--color-primary-foreground: hsl(0 0% 100%);  /* White (was hsl(0 0% 98%)) */
--color-ring: hsl(215 70% 55%);              /* Blue focus ring (was hsl(0 0% 64%)) */
```

Keep all other colors unchanged (secondary, muted, destructive, success, warning, border, input).

Add `.code-block` utility class after the `#root` rule:

```css
.code-block {
  background-color: hsl(220 13% 18%);
  color: hsl(220 9% 82%);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 0.75rem;
  line-height: 1.5;
  overflow: auto;
  max-height: 15rem;
}
```

**Why a CSS class**: Avoids repeating 6+ Tailwind utilities on every `<pre>` across 5 components.

---

## Step 2: New `src/components/AppLayout.tsx`

Create layout wrapper with minimal header bar (48px):

- **Left**: "OAuth2 Test App" (app name)
- **Center**: Page label (Configuration / Access Callback / Processing / Dashboard)
- **Right**: Contextual action button
  - On `/`: Reset button (`data-testid="btn-header-reset"`) - clears sessionStorage + navigates to `/`
  - On `/dashboard`: Logout button (`data-testid="btn-header-logout"`) - calls `clearAll()` + `setToken(null)` + navigates to `/`
  - Other pages: empty

Uses `useLocation()` for page detection, `useAuth()` for token/logout, `clearAll` from `@/lib/storage`.

`<main>` wraps children with `flex-1 flex justify-center`.

---

## Step 3: Wire Layout -- `src/App.tsx` + `index.css`

In `App.tsx`: Wrap `<Routes>` with `<AppLayout>` inside `<BrowserRouter>` (must be inside router for `useLocation`).

In `index.css`: Update `#root` - remove `display: flex; justify-content: center` (layout handles centering now). Keep `width: 100%; min-height: 100vh`.

---

## Step 4: ConfigForm Improvements -- `src/components/ConfigForm.tsx`

1. **Remove inline Reset button** (the `<Button variant="secondary" onClick={handleReset}>Reset</Button>` and the `handleReset` function). Header handles reset via reload.

2. **Grid for short fields**: Put Realm + Client ID side-by-side in `grid grid-cols-2 gap-4`.

3. **Add spinner to loading state**: Replace plain italic text with:
   ```tsx
   <div className="flex items-center justify-center gap-2 py-4 text-muted-foreground">
     <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
     <span className="text-sm italic">Requesting access...</span>
   </div>
   ```

4. **Adjust button container**: Just the submit button with `pt-2`, no flex gap needed.

All `data-testid` and `data-test-state` attributes preserved exactly.

---

## Step 5: ScopeDisplay Badge Rendering -- `src/components/ScopeDisplay.tsx`

Replace plain text with labeled rows of Badge chips:

- "Resource:" row with badges for each space-separated scope (or "none" italic)
- "Access Request:" row with badges (or "none" italic)
- Use existing `Badge` component with `variant="success"`
- Move `data-test-resource-scope` and `data-test-access-request-scope` to the outer div (tests use global query, not scoped)

---

## Step 6: DashboardPage -- `src/pages/DashboardPage.tsx`

1. **Remove inline Logout button** and its `handleLogout` function (header handles it)
2. Change `max-w-4xl` to `max-w-3xl` for tighter layout
3. Change `space-y-6` to `space-y-5`, `py-8` to `py-6`

All `data-testid` and `data-test-state` attributes preserved.

---

## Step 7: Code-Block Styling on JSON Displays

Replace `className="text-xs bg-muted p-3 rounded-md overflow-auto max-h-60"` with `className="code-block"` (max-height already in `.code-block`):

| File | Elements |
|------|----------|
| `TokenDisplay.tsx` | Raw token div, JWT header pre, JWT payload pre |
| `UserInfoSection.tsx` | `data-testid="user-info-response"` pre |
| `ToolsetsSection.tsx` | `data-testid="toolsets-list"` pre, `data-testid="toolset-result"` pre |
| `RestClientSection.tsx` | `data-testid="rest-response"` pre |

For TokenDisplay raw token: keep `break-all` and `max-h-[200px]` alongside `code-block`.

---

## Step 8: Loading Spinners on Callback Pages

**`AccessCallbackPage.tsx`**: Replace plain italic loading text (inside `data-testid="access-callback-loading"`) with spinner + text pattern (same as Step 4).

**`OAuthCallbackPage.tsx`**: Replace `<div className="text-center py-4 italic text-muted-foreground">{status}</div>` with spinner + text.

---

## Step 9: E2E Page Object Update -- `DashboardSection.mjs`

Update logout selector from `button:has-text("Logout")` to `[data-testid="btn-header-logout"]`.

**No other e2e files need changes.** Analysis:
- ConfigSection: never referenced the Reset button
- All other sections use `data-testid` selectors exclusively
- ChatSection uses `.justify-start` and `p.whitespace-pre-wrap` CSS classes -- we don't touch ChatSection message rendering
- RestClientSection parses "Status: NNN" text format -- unchanged

---

## Verification

1. **Visual check**: Run `cd crates/lib_bodhiserver_napi/test-oauth-app && npm run dev`, open localhost:5173, verify:
   - Blue primary buttons
   - Header bar with "OAuth2 Test App" + "Configuration" label + Reset button
   - Dark code blocks (when on dashboard)
   - Scope badges (after request-access flow)
   - Loading spinners (during access request)

2. **E2E tests**: Run full e2e suite from `crates/lib_bodhiserver_napi/tests-js/`:
   ```bash
   cd crates/lib_bodhiserver_napi && npm run test
   ```
   All specs in `specs/oauth/` and `specs/toolsets/` must pass.
