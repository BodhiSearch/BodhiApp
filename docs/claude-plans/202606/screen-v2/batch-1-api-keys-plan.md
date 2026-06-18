# Batch 1 — API Keys — Plan

> Status: **DRAFT for approval**. Working folder: `docs/claude-plans/202606/screen-v2/`.
> Loop ref: `process.md` §"The per-batch loop". Kickoff: `batch-1-api-keys-kickoff.md`.
> Grounded in a fresh read of the current code (2026-06) + visual review of the 4 prototypes +
> the `tanstack-router`, `tanstack-query`, and `vercel-react-best-practices` skills.
>
> **On approval, this file is renamed/copied to `screen-v2/batch-1-api-keys-plan.md`** (the canonical
> per-batch plan location). It lives at the plan-mode path only because that's the one file editable
> while planning.

## Context — why this batch

Batch 0 made AppShell the root layout (all app routes render inside the shell behind default-old
per-screen flags) and shipped the backend id_token + `reference_api_url`. **Batch 1 migrates the
first real section — API Keys** — porting its screens to the V2 design behind their flags, then
retiring the flags and deleting the old code. It's the first batch that needs **rich shell chrome**
(breadcrumb, header actions, a detail rail) and the first to introduce a **non-shell (bare) screen**,
so it establishes two patterns every later batch reuses.

## Scope (4 screens) — decisions locked with the user

| Screen | prod route | layout | flag id | gap decision |
|---|---|---|---|---|
| App Tokens (list) | `/tokens/` | **shell** (api-keys / app-tokens) | `app-tokens` | real-data-only: no model/MCP-access badges, no "last used", no rich rail beyond real fields |
| New App Token | **NEW** `/tokens/new/` (was a dialog) | **shell** (api-keys / new-token) | `new-token` | real-data-only: page has **name + scope only** (no Model/MCP-access pickers — backend `CreateTokenRequest = {name?, scope}`) |
| Access Requests (list) | `/users/access-requests/` | **shell** (api-keys / access-requests) | `access-requests` | all fields real |
| Access Request review | `/apps/access-requests/review/?id=` | **bare / standalone** (no sidebar) | `access-request-review` | all rendered fields real (MCP servers+instances, role, app name/desc); **no model-slots** (design-only, omit) |

**Key user directives baked in:**
1. **Real data only — do NOT increase scope.** The prototypes show pipeline features (per-token
   model/MCP access, "last used", model-slots on review) the backend doesn't have. We port the V2
   *look* but bind only to fields that exist. Those sections are added when their backend lands.
2. **Review screen is now in-scope and final** (was design-pending). It is a **standalone bare
   layout** (slim topbar, centered card — the `bodhi-standalone.css` family). Update the docs.
3. **Multi-layout: minimal now, scalable seam deferred** *(user decision)*. This batch makes the
   review screen bare via the existing `BARE_PREFIXES` + a reusable `BareLayout` component; the
   route-declared layout seam (TanStack Router `staticData` → pathless `_layout` routes) that future
   user-access-request / status pages will share is tracked as a deferred follow-up in `techdebt.md`.
4. **Temporary migration scaffolding is OK but must be tracked in `techdebt.md`** for removal once
   the whole migration completes.

## Prerequisites (port before implementing screens)

Side-effect CSS ports (design → `crates/bodhi/src/...`), stripped to our conventions:
- `design/bodhi-form.css` → form primitives (`bf-*`) — New App Token.
- `design/api-keys-shell.css` → list/toolbar/tag primitives (`page-header`, `filter-tabs`,
  `empty-state`, `form-card`, `btn-accent`/`btn-ghost`) — both list screens + form.
  ⚠ **generic class names** here (`.page-header`, `.empty-state`, `.btn-ghost`) risk colliding with
  app/Tailwind utilities — **scope them on port** (prefix or wrap under a screen-root class) and grep
  for existing uses before importing globally.
- `design/bodhi-list.css` → detail-panel/list primitives (`dp-*`, `bl-*`) — list screens + rail.
- `design/bodhi-standalone.css` → `std-*` bare layout (slim topbar, centered `std-page`) — review
  screen + the reusable bare layout. (Batch 0 deliberately did NOT port this; we port the subset the
  review screen needs.)
- **`ModelAccessPicker`**: per the "real-data-only" rule, the review screen's real data is **MCP
  servers + role**, not model slots — so we **do NOT port `model-access-picker.{jsx,css}`** this
  batch (its only real consumer, the New App Token pickers, is dropped; the review screen's
  model-slots are design-only). Reserve the port for the batch that has model-access backend data
  (Models/MCP). *(Confirmed against the review payload: no model/slot fields.)*

`shell.css` (Batch 0) already carries all `shell-*` chrome; these ports add disjoint namespaces.

No reference-API and no backend/OpenAPI changes this batch (presentation-only).

## Architecture pieces (the two reusable patterns)

### A. Multi-layout (bare) — minimal now, scalable seam deferred  *(user decision)*

**Problem:** post-Batch-0 the review route renders awkwardly inside the shell (`section=''`, no
highlight). It must render **bare** (slim topbar, centered card — the standalone family).

**This batch (minimal):** add `/apps/access-requests/review` to the existing `BARE_PREFIXES` in
`resolveShellRoute.ts` so it renders bare. Build the reusable **`BareLayout`** component
(`components/shell/BareLayout.tsx`) — slim topbar (brand + theme toggle) + centered `std-*` content —
and render bare routes through it in `__root` (instead of the current bare `<main>` passthrough), so
future user-access-request / status pages reuse the same standalone chrome.

**Deferred (tracked in `techdebt.md`):** the **scalable route-declared layout seam** — each route
declaring `staticData.layout: 'shell' | 'bare'`, read in `__root` via `useMatches()`, replacing the
central `BARE_PREFIXES` pathname switch (and eventually converging to idiomatic pathless
`_shell/`/`_bare/` layout routes). The user opted to defer this to a dedicated later step rather than
fold a cross-cutting routing refactor into the API-Keys batch. **`BareLayout` is built so the later
seam is a drop-in** (it doesn't depend on how the bare-vs-shell decision is made).

> Why deferred: the canonical TanStack pattern (pathless `_layout` route groups) requires **moving
> every app route file** under a layout dir — a large, risky restructure of `routeTree.gen.ts`
> inappropriate for one section's batch. The `staticData` variant avoids file moves but is still a
> routing-wide change. Minimal-now keeps Batch 1 presentation-scoped; the seam lands cleanly later.

### B. Shell-slots context — screens contribute breadcrumb / headerActions / rail

**Problem:** `AppShell` already accepts `breadcrumb`/`headerActions`/`sidebar`/`rail`/`railHeader`
props, but `__root` renders one `<AppShell>` and only passes `section`/`subPage`. A child screen
can't feed those slots up.

**Solution (option (b), per kickoff rec + user's "track as techdebt"):** a small **ShellSlots
context**. `__root`'s single `<AppShell>` consumes the published slots; a migrated screen publishes
them via a hook. Built to avoid the classic context re-render trap (vercel
`rerender-no-inline-components`, `rerender-derived-state-no-effect`):

- `components/shell/ShellSlotsContext.tsx`:
  - **Two contexts**: a **stable setter context** (`setSlots`, identity never changes) and a
    **value context** (`{breadcrumb, headerActions, rail, railHeader, railDefaultOpen}`). Splitting
    prevents publishers from re-rendering when values change and keeps `AppShell` the only consumer
    of values.
  - `useShellChrome(slots)` — a screen calls it; an effect publishes `slots` on mount/update and
    **clears on unmount** (so navigating away resets chrome). Deps are the individual slot nodes;
    screens pass **stable nodes** (defined at module scope or memoized) — never inline component
    definitions.
  - `__root`: `const slots = useShellSlots();` spread into `<AppShell section subPage {...slots}>`.
- **This whole context is temporary scaffolding** — once every screen is migrated, screens can pass
  props to a per-route `<AppShell>` directly (or we move to pathless layouts) and the context is
  deleted. → **recorded in `techdebt.md`** (see Docs step).

> React + Query best-practices applied throughout:
> - **Filter-tab counts derived during render** from the already-fetched list (or via Query `select`)
>   — no extra fetch, no effect/state (vercel `rerender-derived-state-no-effect`).
> - **Detail rail reads from the same cached list**: selection is a local `selectedId`; the rail body
>   is `list.find(x => x.id === selectedId)` — **no per-id query / new query key** (tanstack-query
>   master-detail). Memoize the selected node.
> - **Hooks/query-keys unchanged**: the existing hooks already invalidate their list keys on success
>   (`useCreateToken`/`useUpdateToken` → `tokenKeys.all`; `useApproveRequest`/`useRejectRequest` →
>   access-request keys). Presentation-only restructure must NOT touch keys or invalidation.
> - Keep row/card components at module scope with primitive props (vercel
>   `rerender-no-inline-components`); don't subscribe to state only read in callbacks; functional
>   `setState` for stable handlers.

## Per-screen implementation (behind flags; recipe = `process.md` §"Per-screen migration recipe")

All screens keep their `AppInitializer` wrapper **outside** the shell/bare layout (redirects must
still fire). Each screen renders **old vs new** via `useUiV2Flag('<id>')` (default-old). Hooks,
queries, mutations, fixtures are **unchanged** (presentation-only). **Preserve every `data-testid` +
ARIA** verbatim (inventory below).

### 1. App Tokens list (`/tokens/`, flag `app-tokens`)
- New V2 render: shell screen; publish via `useShellChrome` → breadcrumb `Bodhi › API Keys › App
  Tokens`, headerActions = **New Token** button (→ `navigate('/tokens/new/')`), optional detail
  **rail** = selected-token details (real fields only: name, scope, status, prefix, created/updated).
- Body: filter tabs (All / Active / Revoked) with **counts derived during render** from the token
  list; search box (client-side filter on name/prefix); token cards/rows; empty state.
- **Real fields only**: `TokenDetail = {id,user_id,name,token_prefix,scopes,status,created_at,
  updated_at}`. No model/MCP-access badges, no "last used".
- Hooks unchanged: `useListTokens`, `useUpdateToken` (status toggle). Keep
  `data-testid="tokens-page"` + `data-pagestatus`, `new-token-button`, `tokens-table`,
  `token-name-${id}`, `token-scope-${id}`, `token-status-switch-${id}` (+ `aria-label`).
- The list's "New Token" no longer opens `CreateTokenDialog`; it navigates to the new page.

### 2. New App Token (`/tokens/new/`, flag `new-token`) — dialog → page
- **New route** `routes/tokens/new/index.tsx`, `createFileRoute('/tokens/new/')` (trailing slash) →
  **regenerate `routeTree.gen.ts`**. Shell screen; breadcrumb `… › New App Token`.
- Form = **name (optional) + scope** (the existing `createTokenSchema`), wired to the **real
  `useCreateToken`** mutation (NOT the prototype's fake generator). On success, **keep the existing
  `TokenDialog` modal** (reused as-is) over the page to reveal the generated token (copy/show/done);
  **Done closes it and navigates back to `/tokens/`** *(user decision — reuse TokenDialog, minimal
  markup change)*.
- **Omit** the prototype's Model Access / MCP Access pickers and the User/PowerUser "scope cards"
  beyond the two real scope values (render scope as the existing select or as 2 real cards — visual
  only, same 2 values).
- Preserve testids by moving them to the page: `token-form`, `token-name-input`,
  `token-scope-select` (+ `scope-option-*`), `generate-token-button`, and the success
  `token-dialog`/`token-value-input`/`token-dialog-done` (+ copy/show toggles). The
  **creation-flow test moves** to the new route's test file.

### 3. Access Requests list (`/users/access-requests/`, flag `access-requests`)
- Shell screen under **api-keys / access-requests** (production `SHELL_NAV` already maps it there;
  the prototype's `section="users"` is a prototype artifact — we honor the production nav).
- Publish breadcrumb `Bodhi › API Keys › Access Requests` + headerActions **pending-count pill**
  (count **derived during render** from the list). Filter tabs (All / Pending / Approved / Denied);
  request cards/rows; optional detail rail (request details). Real fields only (username, status,
  date, reviewer; approve/reject per row).
- Hooks unchanged: `useListAllRequests`, `useApproveRequest`, `useRejectRequest`. Keep
  `all-requests-page`, `page-title`, `request-count`, `requests-table`, `pagination`,
  `request-username`, `request-date`, `request-status-${status}`, `request-reviewer`,
  `role-select-${username}`, `approve-btn-${username}`, `reject-btn-${username}`, `no-requests`,
  and the `request-row-${username}` + nav-link testids the page object queries.

### 4. Access Request review (`/apps/access-requests/review/?id=`, flag `access-request-review`)
- **Bare/standalone**: add the route to `BARE_PREFIXES`; it renders under the new `BareLayout`.
  V2 look = `std-page` centered card; title "Review Access Request" + subtitle.
- Render only **real** review data: app name/description (`app_name ?? app_client_id`, `app_description`),
  **MCP servers + instances** (`mcps_info[]` → per-server card: toggle approve, instance select),
  **Approved Role** select, **Deny / Approve** actions. Approve/deny flow (`flow_type` popup/redirect)
  unchanged.
- **Omit** the prototype's "Model Access" slots section (no data in `AccessRequestReviewResponse`)
  and the verified / 3rd-party badges (no fields).
- Hooks unchanged: `useGetAppAccessRequestReview`, `useApproveAppAccessRequest`,
  `useDenyAppAccessRequest`. Preserve `review-access-page` (+ `data-test-status`/`data-test-flow-type`),
  `review-access-loading`/`-error`, `review-status-${status}`, `review-app-name`/`-description`,
  `review-mcp-*` (toggle/select/trigger/instance-option/no-instances), `review-approved-role-*`,
  `review-approve-button`, `review-deny-button`. **Keep `McpServerCard`'s testids verbatim.**

## Tests (all three layers — per `feedback_testing_depth`)

**RTL / Vitest + MSW (handlers + fixtures reused unchanged — contract identical):**
- Update the 5 colocated tests for restructured markup, preserving queries by testid:
  `routes/tokens/index.test.tsx`, the moved create-flow test → `routes/tokens/new/index.test.tsx`,
  `routes/users/access-requests/index.test.tsx`, `routes/apps/access-requests/review/index.test.tsx`,
  and the token-dialog/-form component tests (folded into the new page or kept as components).
- New tests for the patterns: `ShellSlotsContext.test.tsx` (publish/clear on mount/unmount;
  `AppShell` consumes), `BareLayout.test.tsx` (renders slim topbar + centered content, no sidebar),
  and a `resolveShellRoute`/`__root` test asserting the review route resolves **bare** (added to
  `BARE_PREFIXES`) while the 3 token/access-request routes stay **shell** with correct
  `section`/`subPage`. Extend `AppShell`/root render helpers as needed.
- Each migrated screen tested **with the flag ON** (new render) — and the existing default-old path
  stays green until flags are retired.

**server_app integration (Rust):** none — presentation-only, no route/contract change (confirmed:
no server_app tests target these HTTP routes).

**Playwright E2E (black-box; `make test.e2e` from `lib_bodhiserver/tests-js`):**
- Update page objects: `pages/TokensPage.mjs` (nav via shell nav `shell-sub-app-tokens`;
  dialog→page: `navigateToNewToken()` clicks `shell-sub-new-token` / `new-token-button` then asserts
  `/ui/tokens/new/`), `pages/AllAccessRequestsPage.mjs` (nav via `shell-sub-access-requests`),
  `pages/AccessRequestReviewPage.mjs` (unchanged selectors; bare layout). Add a shared **shell-nav
  helper** in `pages/BasePage.mjs` (`navViaShell(section, sub)`).
- Specs updated in place: `specs/tokens/api-tokens.spec.mjs` (create flow now a page),
  `specs/request-access/multi-user-request-approval-flow.spec.mjs`. Keep them black-box.
- The flags must be **ON** for the E2E run of migrated screens (set via the page object's
  init/localStorage seed through the UI, or flip default at flag-retirement before the final E2E).

## Gate checks (all green before commit — `feedback_run_all_gate_checks`)
`cd crates/bodhi && npm run test` (RTL) · typecheck/lint on touched files ·
`make test.e2e` (start Docker; dev-server + Vite HMR, no ui-rebuild) · manual eyeball via
`make app.run.live` (each screen, light/dark, shell collapse/rail, bare review page). No backend
tests needed (no Rust change) — the 2 pre-existing live-test failures are NOT ours (`techdebt.md`).

## Retire flags + delete old code (end of batch)
Remove the 4 `useUiV2Flag` branches + the 4 flag ids' old paths; delete the old
`CreateTokenDialog` wiring from the list (the create flow now lives at `/tokens/new/`); delete any
now-dead old-render code for the 4 screens. Commit per batch (trunk-based, directly on `main`;
`git fetch && git rebase origin/main` first).

## Docs to update (this batch)
- `screen-v2/screen-coverage.md`: move **Access Request review** from §C (out-of-scope/deferred) to
  §A (covered, in-scope) as a **bare** screen; note the multi-layout (shell/bare) seam.
- `screen-v2/techdebt.md`: add two kinds of items —
  - **Temporary migration scaffolding to remove at end of migration**: (1) `ShellSlotsContext` +
    `useShellChrome` (replaced by per-route `<AppShell>` props / pathless layouts once all screens
    migrate); (2) the per-screen `useUiV2Flag` machinery (removed when the last batch lands).
  - **Deferred architectural improvement** (NOT temporary — a planned follow-up): the **scalable
    route-declared layout seam** to replace the `resolveShellRoute` `BARE_PREFIXES` pathname switch
    (route `staticData.layout` → eventually pathless `_shell/`/`_bare/` layout routes). Batch 1 does
    the minimal thing (adds the review route to `BARE_PREFIXES` + builds reusable `BareLayout`); the
    seam is a dedicated later step.
- `screen-v2/common-prompt.md`: **add the new skills** to the "build to our conventions" guidance —
  `tanstack-router`, `tanstack-query`, and `vercel-react-best-practices` (+ `web-design-guidelines`)
  as the references to consult on every screen port. *(User asked for this explicitly; done as the
  first implementation step since it's not editable in plan mode.)*
- Crate docs: note the new `/tokens/new/` route + the shell-slots/bare-layout patterns in
  `crates/bodhi/src/CLAUDE.md`/`PACKAGE.md` as touched.

## Risks / watch-outs
- **Generic CSS class collisions** from `api-keys-shell.css` (`.page-header`, `.empty-state`,
  `.btn-ghost`) — scope on port; grep first.
- **Trailing-slash + route tree**: new `/tokens/new/` needs `routeTree.gen.ts` regen; all nav hrefs
  trailing-slashed.
- **Shell-slots re-render trap**: split setter/value contexts; stable slot nodes; clear-on-unmount.
  Don't define slot components inline (vercel `rerender-no-inline-components`).
- **Never transition `grid-template-columns`** in the shell (carry-forward risk #1) — unaffected here
  but don't reintroduce.
- **testid discipline** is the linchpin — restructure markup freely, keep testids/ARIA verbatim.
- **Scope creep guard**: if a prototype element has no backing data, it is **out** (model/MCP-access
  pickers, last-used, model-slots, verified/3rd-party badges). Don't ship dead UI.
- **Flag ON during E2E** for migrated screens — black-box only (seed flag through the UI, not
  `page.evaluate`).

## Verification (end-to-end)
1. `make app.run.live`, log in, set each flag on (dev toggle), walk all 4 screens light/dark; verify
   shell chrome (breadcrumb/headerActions/rail), the new `/tokens/new/` create flow end-to-end
   (real token generated + appears in list), the bare review page (no sidebar, slim topbar), and
   nav highlight (`api-keys` + correct sub).
2. RTL green (incl. new context/layout tests); E2E green (incl. updated page objects/specs).
3. Flags retired + old code deleted → re-run RTL + E2E green → commit → retro → Batch 2 kickoff.
