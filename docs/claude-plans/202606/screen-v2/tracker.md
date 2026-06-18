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
| 1 | **API Keys** (4 screens) | ✅ done |
| 2 | Settings (App Settings, Manage Users) | ⬜ not started (kickoff written: @batch-2-settings-kickoff.md) |
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
| 9 | **API Keys** | **App Tokens (list)** | `/tokens/` | shell | — (retired) | ✅ | Design-faithful: table (Token/Created/Updated/Status cols), themed filter pills (All/Active/Inactive), collapsible search button, selectable rows → detail rail (Details + Token-active toggle). No "New Token" header button (it's in the sidebar nav). Real data only (no model/MCP-access chips, no last-used). **Detail-rail open/close has a view transition.** |
| 10 | **API Keys** | **New App Token** | `/tokens/new/` (NEW) | shell | — (retired) | ✅ | Was a dialog → full page. Real `useCreateToken`; reuses `TokenDialog` reveal; Done → `/tokens/`. Name+scope only (real data). |
| 11 | **API Keys** | **Access Requests (list)** | `/users/access-requests/` | shell | — (retired) | ✅ | Breadcrumb + pending-count pill + filter tabs + approve/reject. |
| 12 | **API Keys** | **Access Request review** | `/apps/access-requests/review/?id=` | ▫️ bare | — (retired) | ✅ | OAuth consent. Renders via `BareLayout` (slim topbar). MCP servers+instances + role + approve/deny (real data; no model-slots). |
| 13 | Settings | App Settings | `/settings/` | shell | `app-settings` | ⬜ | Batch 2. `reference_api_url` NOT editable. |
| 14 | Settings | Manage Users | `/users/` | shell | `manage-users` | ⬜ | Batch 2. Reuses `list.css`. |

**Retired/consolidated (no separate screen):** `/users/pending/` → folded into Access Requests
(Batch 1 migrated the target; confirm `/users/pending/` + `UserManagementTabs` deletion in Batch 2).
`/models/files/`, `/models/files/pull/`, `/mcps/servers/*` → absorbed (see @screen-coverage.md §B).

**Out of scope (deferred):** access-request standalone (`/request-access/`, pending), setup wizard,
Keycloak/auth (@screen-coverage.md §C).

## Cross-cutting (done)
- **Reusable patterns** (Batch 1): `BareLayout` (standalone chrome) + `ShellSlotsContext`/`useShellChrome`
  (screens publish breadcrumb/headerActions/rail to the root `<AppShell>`). Both are **temporary
  scaffolding** → @techdebt.md.
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
