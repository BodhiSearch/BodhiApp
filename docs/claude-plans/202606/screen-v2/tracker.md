# UI V2 Migration — Tracker

Per-screen status for review. **Scope:** the 13 shell app screens + the bare OAuth Access-Request
review = **14 screens** (see @screen-coverage.md). Verified against the code, not memory
(`grep useUiV2Flag`, `useShellChrome`, `BARE_PREFIXES`, `SHELL_NAV` on the date below).

> Status key: ✅ done (V2-only, flag retired, old code deleted) · 🚧 in progress ·
> ⬜ not started · ▫️ bare (renders outside AppShell).
> Last verified: **2026-06-18** (after Batch 1 + the View-Transitions add-on).

## Batches

| Batch | Section | Status |
|---|---|---|
| 0 | Foundation (AppShell, flags, tokens merge, id_token, reference_api_url) | ✅ done |
| 1 | **API Keys → Access Tokens** (4 screens) | ✅ done |
| 1.1 | **Batch-1 follow-up** — IA correction (2026-06-18): nav section "API Keys"→"Access Tokens" (sub-pages "API Tokens"/"New API Token"); new top-level **Users** section (User Access Requests + Manage Users, moved out of Settings); Access Requests redesigned to match `design/User Access Requests.html` (avatar rows, status chips, role/approve-reject flow, detail rail). | ✅ done |
| 2 | **Settings** (App Settings) + **Manage Users** (nav under Users per 1.1) | ✅ done |
| 3 | Models (4 screens) | ⬜ not started |
| 4 | MCP (3 screens) | ⬜ not started |
| 5 | Chat (1 screen, highest risk, last) | ⬜ not started |

## Screen-by-screen

| # | Section | Screen | Route | Layout | Flag | Status | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Chat | Chat | `/chat/` | shell | `chat` | ⬜ | Batch 5 (last). Old chat UI renders inside the new shell today. |
| 2 | Models | All Models | `/models/` | shell | `models` | ⬜ | Batch 3. |
| 3 | Models | New Local Model | `/models/alias/new/` (+ edit) | shell | `new-local-model` | ⬜ | Batch 3. `/new/`+`/edit/` share one design. |
| 4 | Models | New API Model | `/models/api/new/` (+ edit) | shell | `new-api-model` | ⬜ | Batch 3. |
| 5 | Models | New Fallback Alias | `/models/router/new/` (+ edit) | shell | `new-fallback-model` | ⬜ | Batch 3. |
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

## Open follow-ups (carry forward)
- Run the **full E2E matrix** (both projects, all specs) once before Batch 2 — the shared shell/CSS +
  the global route view-transition are app-wide; the broad sweep catches any other page object that
  reads the DOM right after a navigation.
- Deferred architectural item: the scalable route-declared layout seam (replace `BARE_PREFIXES`) —
  @techdebt.md.

> Update this tracker at the end of every batch: flip the section's screens to ✅, retire their flag
> ids, and note any new consolidations/deletions.
