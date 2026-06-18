# Category-chip loading state for migrated V2 list pages

## Context

The HEAD commit `c23f5b40 [Amir] category loading state` updated the **design prototypes** (`design/bodhi-list.jsx`, `design/bodhi-list.css`, `design/user-access-requests-app.jsx`) to give the category-chip count badges a **loading shimmer** that is shown until the list data resolves. The motivation: while a list page's query is still fetching, the category badges currently render counts derived from empty data — so the user momentarily sees misleading `All (0)`, `Pending (0)`, etc., as if the lists were empty, before the real counts pop in.

The design changes live only in the `design/` reference folder. This plan ports the same behavior into the **actual React implementation** so the migrated V2 list pages stop showing `(0)` badges during their initial load.

### Current state (from exploration)

- The real equivalent of the design's `ListToolbar` category pills is the **already-shared** component `ShellFilterTabs` (`crates/bodhi/src/components/shell/ShellFilterTabs.tsx`). **No refactor is needed to make it common — it already is.** It renders `{tab.count}` inside `.l-cat-badge` whenever `count != null`, with no loading awareness.
- `.l-cat-badge` styling lives in `crates/bodhi/src/components/shell/list.css`. The `.l-cat-badge--loading` shimmer class from the design does **not** exist in the real CSS yet.
- **4 migrated pages** use `ShellFilterTabs` with count badges, each already holding an `isLoading` flag from its TanStack Query hook:
  | Page | File | Loading flag |
  |------|------|--------------|
  | App Settings | `crates/bodhi/src/routes/settings/-components/SettingsPageV2.tsx` | `isLoading` (`useListSettings`) |
  | Manage Users | `crates/bodhi/src/routes/users/-components/ManageUsersV2.tsx` | `isLoading` (`useListUsers`) |
  | Access Requests | `crates/bodhi/src/routes/users/access-requests/index.tsx` | `isLoading` (`useListAllRequests`) |
  | API Tokens | `crates/bodhi/src/routes/tokens/index.tsx` | `tokensLoading` (`useListTokens`) |
- Settings / Manage Users / Access Requests already render a list-body skeleton while loading. **Tokens does not** — it flashes the `No tokens` empty state during `tokensLoading`.

### Decisions (confirmed with user)

- **Add a body skeleton to the Tokens page** so all four pages behave consistently during load.
- **Do not reorder** the category chips. `All`-last is only on Access Requests today; leave Tokens/Settings/Users with `All` first. Scope stays on the loading state.

## Implementation

### 1. `ShellFilterTabs` — add a `loading` prop (the shared change)

File: `crates/bodhi/src/components/shell/ShellFilterTabs.tsx`

- Add `loading?: boolean` to `ShellFilterTabsProps`, defaulting to `false`.
- In the badge render, mirror the design's logic (`design/bodhi-list.jsx`): when `loading` is true, render the shimmer placeholder instead of the number, for every tab — so the row of chips shows shimmering badges, not `(0)`:
  ```tsx
  {loading ? (
    <span className="l-cat-badge l-cat-badge--loading" aria-label="Loading count" />
  ) : (
    tab.count != null && <span className="l-cat-badge">{tab.count}</span>
  )}
  ```
  Render the shimmer for a tab even when its `count` is undefined during loading, so a tab whose count hasn't been computed still shows the placeholder (matches the design, where `loading || badge != null` gates the span).

### 2. `list.css` — add the shimmer styles

File: `crates/bodhi/src/components/shell/list.css` (after the existing `.l-cat-badge` block, ~line 154)

Port the design's rules verbatim from `design/bodhi-list.css`:
```css
/* loading state: shimmer placeholder shown until counts resolve */
.l-cat-badge--loading { width: 17px; min-width: 17px; padding: 0; color: transparent; position: relative; overflow: hidden; }
.l-cat-badge--loading::after { content: ""; position: absolute; inset: 0; transform: translateX(-100%); background: linear-gradient(90deg, transparent, rgba(255,255,255,.55), transparent); animation: l-badge-shimmer 1.15s ease-in-out infinite; }
.l-cat.on .l-cat-badge--loading::after { background: linear-gradient(90deg, transparent, rgba(255,255,255,.35), transparent); }
@keyframes l-badge-shimmer { 100% { transform: translateX(100%); } }
@media (prefers-reduced-motion: reduce) { .l-cat-badge--loading::after { animation: none; } .l-cat-badge--loading { opacity: .55; } }
```
(Format to the file's multi-line style with `npm run format` rather than committing the one-liners.)

### 3. Wire `loading` into all four pages

Each page already has its loading flag in scope. Pass it to the toolbar:

- `SettingsPageV2.tsx` (~line 532): `<ShellFilterTabs ... loading={isLoading} />`
- `ManageUsersV2.tsx` (~line 389): `<ShellFilterTabs ... loading={isLoading} />`
- `access-requests/index.tsx` (~line 189): `<ShellFilterTabs ... loading={isLoading} />`
- `tokens/index.tsx` (~line 148): `<ShellFilterTabs ... loading={tokensLoading} />`

No new state or hooks — reuse the existing `isLoading`/`tokensLoading`.

### 4. Tokens page — add the missing list-body skeleton

File: `crates/bodhi/src/routes/tokens/index.tsx`

Today the body only branches on `visible.length === 0` (line ~160), so it flashes `No tokens` during `tokensLoading`. Add a leading `tokensLoading` branch that renders the same skeleton pattern the other three pages use (`Skeleton` is already imported):
```tsx
{tokensLoading ? (
  <div style={{ padding: 16 }} data-testid="loading-skeleton">
    {Array.from({ length: 5 }).map((_, i) => (
      <Skeleton key={i} className="h-12 w-full mb-3" />
    ))}
  </div>
) : visible.length === 0 ? (
  /* existing empty state */
) : (
  /* existing list */
)}
```
Keep the existing `appLoading` early-return (lines 128–137) as-is; it covers the pre-app-info phase. The new branch covers the `tokensLoading` phase after app info is ready. `data-pagestatus` already flips on `tokensLoading`.

## Tests

Component tests run via `cd crates/bodhi && npm test`. The existing V2 tests wait for `data-pagestatus="ready"` before asserting resolved badge counts (e.g. `tokens-filter-all` shows `2` in `tokens/index.v2.test.tsx:113`), so the loading shimmer does not break them — once loading completes, `loading` is false and the numeric badge renders exactly as before.

Add focused coverage:

- **`ShellFilterTabs`** — if a sibling unit test exists, add cases: `loading` renders `.l-cat-badge--loading` placeholders (with `aria-label="Loading count"`) and no numeric text for any tab; `loading=false` renders numeric badges as today. Otherwise add the assertions inline to the page tests below.
- **Each of the 4 page V2 tests** (`tokens/index.v2.test.tsx`, `users/index.v2.test.tsx`, `users/access-requests/index.v2.test.tsx`, and the settings V2 test): add an assertion that while the query is pending (before `data-pagestatus="ready"`) the filter chips show the loading placeholder rather than `(0)` counts — e.g. query `aria-label="Loading count"` / `.l-cat-badge--loading` is present and the resolved count text is absent. Use the existing MSW delayed-response / pending-state pattern already in these suites.
- **Tokens body skeleton** — add a test that `loading-skeleton` (testid) renders while the tokens query is pending and the `No tokens` empty state is NOT shown during load (closes the current flash-of-empty gap).

## Verification

1. `cd crates/bodhi && npm run lint && npm run format` — typecheck/lint/format clean (strict TS, no `any`).
2. `cd crates/bodhi && npm test` — full component suite green, including the new loading-state assertions.
3. Manual / browser check via `make app.run.live` (live Vite, HMR): open each of the four screens — **Settings**, **Manage Users**, **Access Requests** (`/ui/users/access-requests/`), **Tokens** (`/ui/tokens/`) — and on initial load confirm the category badges show a shimmer (not `(0)`), then resolve to real counts. Throttle the network (or rely on the natural fetch delay) to see the shimmer. Confirm Tokens now shows the body skeleton instead of a `No tokens` flash. Verify `prefers-reduced-motion` disables the shimmer animation (static dimmed badge).
4. No backend/API/OpenAPI changes — this is frontend-only; no `make build.ts-client` needed.
