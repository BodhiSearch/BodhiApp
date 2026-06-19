# UI V2 Migration — Tracker

Per-screen status for review. **Scope:** the 13 shell app screens + the bare OAuth Access-Request
review = **14 screens** (see @screen-coverage.md). Verified against the code, not memory
(`grep useUiV2Flag`, `useShellChrome`, `BARE_PREFIXES`, `SHELL_NAV` on the date below).

> Status key: ✅ done (V2-only, flag retired, old code deleted) · 🟩 done-behind-flag (shipped V2 but
> flag kept default-off; V1 still present — see note) · 🚧 in progress · ⬜ not started ·
> ▫️ bare (renders outside AppShell).
> Last verified: **2026-06-19** (after Batch 3-1 + its design-refinement follow-up).

## Batches

| Batch | Section | Status |
|---|---|---|
| 0 | Foundation (AppShell, flags, tokens merge, id_token, reference_api_url) | ✅ done |
| 1 | **API Keys → Access Tokens** (4 screens) | ✅ done |
| 1.1 | **Batch-1 follow-up** — IA correction (2026-06-18): nav section "API Keys"→"Access Tokens" (sub-pages "API Tokens"/"New API Token"); new top-level **Users** section (User Access Requests + Manage Users, moved out of Settings); Access Requests redesigned to match `design/User Access Requests.html` (avatar rows, status chips, role/approve-reject flow, detail rail). | ✅ done |
| 2 | **Settings** (App Settings) + **Manage Users** (nav under Users per 1.1) | ✅ done |
| 3 | Models — **split into sub-phases 3-1…3-7** (2026-06-19, see below) | 🚧 in progress |
| 3-1 | **My Models** list + faceted sidebar + detail rail (first **full-stack** batch: backend `size`/capability/**search** + server-side filters) | 🟩 done-behind-flag |
| 3-2 | **New API Model** form (`api/new`+`edit`) — **NEXT** | ⬜ not started |
| 3-3 | **New Fallback** alias form (`router/new`+`edit`) | ⬜ not started |
| 3-4 | **New Local Model** form (`alias/new`+`edit`) — richest; sequenced after the simpler forms | ⬜ not started |
| 3-5 | files / pull consolidation (fold download + quant table into the local-model form) | ⬜ not started |
| 3-6 | **Local Models** discovery sub-view (reference API) | ⬜ not started |
| 3-7 | **API Models** discovery sub-view (reference API) | ⬜ not started |
| 4 | MCP (3 screens) | ⬜ not started |
| 5 | Chat (1 screen, highest risk, last) | ⬜ not started |

> **Batch 3 split + re-sequenced (2026-06-19):** the My Models *list* lands first on the real backend
> with a server-side faceted sidebar + search (3-1). The forms follow **simplest-first**: API (3-2) →
> Fallback (3-3) → Local (3-4); then files/pull consolidation (3-5); then the reference-API discovery
> views, split Local (3-6) and API (3-7). Kick-offs: `batch-3-2…3-7-*-kickoff.md`. Rationale +
> decisions: `batch-3-1-models-retro.md`.
>
> **3-1 is `done-behind-flag`, not `done`:** V2 shipped behind the default-off `models` flag and the V1
> `ModelsPageContent` list is **kept**, because the existing model/api-model E2E specs drive
> create/edit/delete via the **V1 list↔form flow** and the forms are still V1. The `models` flag + the
> V1 list retire in the sub-phase that migrates the last consuming form + its specs (≈3-4/3-5). See the
> retro's flag-deferral note.

## Screen-by-screen

| # | Section | Screen | Route | Layout | Flag | Status | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Chat | Chat | `/chat/` | shell | `chat` | ⬜ | Batch 5 (last). Old chat UI renders inside the new shell today. |
| 2 | Models | **My Models** (was "All Models") | `/models/` | shell | `models` (default-off) | 🟩 | **Batch 3-1** (+ design refinement). V2 shell list + published **faceted `sidebar`** (TYPE / CAPABILITY vision·tool-use·reasoning / SIZE dual-slider / API-FORMAT incl. **Liberty**) + **always-visible search** + selectable rows (`LinkRow`) + **read-only detail rail, 4 variants** (Local File / Model Alias / API Model w/ models list / Fallback w/ routing chain), Edit CTA → V1 form routes. **First full-stack batch:** added `size`+capability+**`search`** + **server-side facet filters** (`type`/`api_format`/`size_*`/`capability`/`search`) to `GET /bodhi/v1/models` + regen; `useListModels` gained `keepPreviousData`. **Refinement (2026-06-19):** nav "All Models"→**"My Models"** (id `my-models`); **TYPE removed from the top bar** (sidebar-only); **search moved server-side**, submit-on-Enter; **colorful** per design (per-type icon tiles saffron/lotus/indigo/teal, API provider badge, active-row left-accent). **Flag NOT retired — V1 list kept** (see batches note). |
| 3 | Models | New API Model | `/models/api/new/` (+ edit) | shell | `new-api-model` | ⬜ | Batch **3-2** (NEXT). `/new/`+`/edit/` share one design. |
| 4 | Models | New Fallback Alias | `/models/router/new/` (+ edit) | shell | `new-fallback-model` | ⬜ | Batch **3-3**. |
| 5 | Models | New Local Model | `/models/alias/new/` (+ edit) | shell | `new-local-model` | ⬜ | Batch **3-4** (richest; after the simpler forms). |
| 6 | MCP | All MCPs / Discover | `/mcps/` | shell | `mcp-discover` | ⬜ | Batch 4. Needs the reference-API catalog. |
| 7 | MCP | New Instance | `/mcps/new/` (+ edit) | shell | `new-mcp` | ⬜ | Batch 4. |
| 8 | MCP | Playground | `/mcps/playground/` | shell | `mcp-playground` | ⬜ | Batch 4. |
| 9 | **Access Tokens** | **API Tokens (list)** | `/tokens/` | shell | — (retired) | ✅ | Nav sub-page "API Tokens" (was "App Tokens"). Design-faithful: table (Token/Created/Updated/Status cols), themed filter pills (All/Active/Inactive), collapsible search button, selectable rows → detail rail (Details + Token-active toggle). No "New Token" header button (it's in the sidebar nav). Real data only (no model/MCP-access chips, no last-used). **Detail-rail open/close has a view transition.** |
| 10 | **Access Tokens** | **New API Token** | `/tokens/new/` (NEW) | shell | — (retired) | ✅ | Nav sub-page "New API Token" (was "New Token"). Was a dialog → full page. Real `useCreateToken`; reuses `TokenDialog` reveal; Done → `/tokens/`. Name+scope only (real data). |
| 11 | **Users** | **User Access Requests (list)** | `/users/access-requests/` | shell | — (retired) | ✅ | Moved out of API-Keys into the new **Users** section (1.1). Redesigned to match `design/User Access Requests.html`: avatar rows · status chips (pending/approved/rejected) · pending-count header pill · filter tabs + collapsible search · pending rows show role picker + approve/reject; **selectable rows → detail rail** mirroring the row (Account/Assign-role/Timeline; decided rows show a decided-note, no actions). Role picker is an **approval input** (no role stored on a request). |
| 12 | **Access Tokens** | **Access Request review** | `/apps/access-requests/review/?id=` | ▫️ bare | — (retired) | ✅ | OAuth consent (3rd-party app, deep-linked — NOT a nav item). Renders via `BareLayout` (slim topbar). MCP servers+instances + role + approve/deny (real data; no model-slots). |
| 13 | Settings | App Settings | `/settings/` | shell | — (retired) | ✅ | Batch 2. V2 shell list: **published `sidebar`** = settings-group scroll-spy nav + Legend; All/Modified/Env filter pills (counts derived); collapsible search w/ highlight; rows → detail rail. **Read-only rail by default** — editor (NEW VALUE + Save/Cancel/Reset via `useUpdateSetting`/`useDeleteSetting`) only for the 2 backend-editable keys (`BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`); others show a "read-only (set via *source*)" note. `source==='system'` hides current value. Dropped the prototype's restart banner/"Needs Restart" tab (no backing data). Rail open = view transition. |
| 14 | **Users** | Manage Users | `/users/` | shell | — (retired) | ✅ | Batch 2. V2 shell list mirroring Access Requests: 5 role filter pills (All/User/Power User/Manager/Admin, per-page counts) + collapsible search; rows → **role-editor rail** (CHANGE ROLE select + Save via `useChangeUserRole`, two-click Remove via `useRemoveUser`) reproducing the real role-hierarchy guards (self → "You", outranked → "Restricted"). Real pagination kept. **Invite link** ported as a header-action popover (multi-tenant only). Reuses `list.css`. |

**Retired/consolidated (no separate screen):** `/users/pending/` → folded into Access Requests
(Batch 1 migrated the target; **`/users/pending/` route + `UserManagementTabs` + the V1 `components/users/`
table/dialogs DELETED in Batch 2**). `/models/files/`, `/models/files/pull/`, `/mcps/servers/*` →
absorbed (see @screen-coverage.md §B).

**Out of scope (deferred):** access-request standalone (`/request-access/`, pending), setup wizard,
Keycloak/auth (@screen-coverage.md §C).

## Cross-cutting (done)
- **Reusable patterns** (Batch 1): `BareLayout` (standalone chrome) + `ShellSlotsContext`/`useShellChrome`
  (screens publish breadcrumb/headerActions/rail to the root `<AppShell>`). Both are **temporary
  scaffolding** → @techdebt.md.
- **`sidebar` chrome slot** (Batch 2): `ShellSlots` gained a `sidebar?: ReactNode` slot so a screen
  can publish a page-body sidebar to the root `<AppShell>` (App Settings' settings-group scroll-spy
  nav is the first consumer; `__root` already spreads `{...slots}`, so no root edit). Part of the
  temporary `ShellSlotsContext` scaffolding.
- **`useViewTransition` hardening** (Batch 2): the hook now swallows the **async** rejection of the
  transition's `ready` promise too (not just `finished`) — an interrupted transition (router
  cross-fade overlapping an in-page rail toggle) rejects `ready` with `InvalidStateError`, which was
  surfacing as an uncaught console exception app-wide. Caught live in GATE B.
- **Mobile rail-drawer fix** (Batch 2 follow-up): at `<768px` the rail is a `position:fixed` drawer
  that slides in via its own `transform` transition. Two fixes so **clicking a row opens the panel on
  mobile** (was broken — applies to every migrated list+rail screen): (1) `useViewTransition` now
  **skips the transition on mobile** (the document-level VT fought the drawer's transform and left it
  closed — same skip path as `prefers-reduced-motion`); (2) `AppShell`'s auto-open effect opens the
  mobile drawer **whenever rail content is present** (not only on the `hasRail` false→true edge), so
  re-selecting after a manual close still opens it. Guarded by RTL (`AppShell`/`useViewTransition`
  mobile cases) + a live mobile E2E (`specs/users/mobile-rail.spec.mjs`, 390px viewport).
- **View Transitions** (React-18 native): router `defaultViewTransition` (route cross-fade) +
  `useViewTransition()` for the App-Tokens detail-rail open/close. Skill:
  `web-animation-view-transitions`. Details + rules: @view-transitions.md.
- **Theme switch in the sidebar footer** (above the user chip, always visible): icon-only
  Light/Dark/System segments expanded; a single cycling button when the sidebar is collapsed.
- **Reusable list-toolbar components** (`components/shell/`, first built on App Tokens — **reuse
  these on every later list screen** instead of re-inlining markup):
  - `ShellFilterTabs` — themed single-select filter pills (`l-cat`) with count badges. Used by App
    Tokens + Access Requests.
  - `useCollapsibleSearch` — search icon button that expands to a search row and **collapses on blur
    when empty** (also Esc / close). Returns `{ row, toggle }` for the toolbar to place.
  - `ShellSearch` gained an `onBlur` prop. `AppShell` now **auto-opens the rail** when a screen
    publishes rail content (so row-click → detail panel works without a manual rail toggle).
- **Helper CSS vars** (`--font-*`, `--shadow-*`, `--border-strong`, …) added to `globals.css`
  (repaired a Batch-0 token-merge gap).

## How to verify Batch 1 (done) yourself
1. **Run the app:** `make app.run.live`, log in. The API-Keys screens are now V2 **by default**
   (flags retired) — no toggle needed.
   - `/tokens/` — filter tabs (All/Active/Revoked) with counts, search, "New Token" in the header,
     click a row → detail rail slides in (view transition), toggle a row's status switch.
   - `/tokens/new/` — name+scope form → Generate → reveal dialog → Done returns to the list.
   - `/users/access-requests/` — filter tabs, pending pill in header, approve/reject a pending row.
   - `/apps/access-requests/review/?id=<id>` — bare consent page (slim topbar, no sidebar).
   - Light/dark both; `prefers-reduced-motion` disables the transitions.
2. **Tests:** `cd crates/bodhi && npm run test` (RTL, 977 pass). E2E (Docker up, from
   `crates/lib_bodhiserver`): `specs/tokens/api-tokens.spec.mjs` +
   `specs/request-access/multi-user-request-approval-flow.spec.mjs` (3/3 on standalone).
3. **Read:** @batch-1-api-keys-retro.md (what landed + decisions) and @view-transitions.md.

## How to verify Batch 3-1 (My Models) yourself
1. **Run the app:** `make app.run.live`, log in, then set the flag (default-off):
   `localStorage.setItem('bodhi.ui-v2.models','true')` and reload `/ui/models/`.
   ⚠ **If you changed the backend, rebuild the binary** (`ports kill 1135 3000` → `make app.run.live`) —
   a stale `app.run.live` binary serves the new frontend via HMR but **ignores the new query params**
   (filters/search silently no-op). See `feedback_gateb_rebuild_binary_for_backend_batches` + the retro.
   - Left sidebar nav reads **"My Models"**; the toolbar is just the **always-visible search** (no TYPE
     tabs). Type a query + **Enter** → server-side `?search=` filters (match highlighted); clear → resets.
   - **Faceted sidebar**: TYPE / CAPABILITY / SIZE dual-slider / API-FORMAT (incl. **Liberty**) →
     server-side filtering (TYPE=API → only API rows; API-FORMAT=Anthropic → the anthropic_oauth row).
   - **Colorful:** per-type icon tiles (saffron/lotus/indigo/teal), API rows show the **provider** as a
     green badge + connection status; click a row → detail rail (4 variants) + Edit CTA → V1 form; the
     active row has a lotus left-accent. Light/dark/responsive all clean.
2. **Tests:** `cargo test -p services -p routes_app --lib` (817 pass); `cd crates/bodhi && npm test`
   (986 pass, incl. `routes/models/index.v2.test.tsx`); E2E `specs/models/all-models-v2.spec.mjs`
   (standalone; list + facets + server-side search + rail + Edit CTA).
3. **Read:** @batch-3-1-models-retro.md (what landed + decisions + the flag-deferral + design refinement).

## Open follow-ups (carry forward)
- **Retire the `models` flag + delete V1 `ModelsPageContent`** when the last consuming form migrates
  (≈3-4/3-5); migrate the model/api-model E2E specs (which drive create/edit/delete via the V1 list↔form
  flow) at the same time. Tracked in `batch-3-1-models-retro.md`.
- Run the **full E2E matrix** at each sub-phase commit — the shared `useListModels`/shell/nav changes
  are app-wide. (Known pre-existing failures: chat-resize techdebt, MCP-OAuth, `api-live-upstream`
  live-provider timeouts, browser-extension — not migration regressions; see @techdebt.md.)
- Deferred architectural item: the scalable route-declared layout seam (replace `BARE_PREFIXES`) —
  @techdebt.md.

> Update this tracker at the end of every batch: flip the section's screens to ✅, retire their flag
> ids, and note any new consolidations/deletions.
