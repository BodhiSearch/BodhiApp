# Migration v2 — completion note

All **9 remaining pages** are now on the canonical **AppShell**. The app
runs on a single layout system; no page hand-rolls sidebar/header/resize/
drawer chrome anymore.

## Pages migrated this phase
| Page | section / subPage | Notes |
|---|---|---|
| App Tokens | api-keys / app-tokens | list page, no rail |
| Access Requests | api-keys / access-requests | list page, pending-count pill in `headerActions` |
| Manage Users | settings / manage-users | own sidebar tabs (collapse-aware), tables, dialogs, toast |
| New App Token | api-keys / new-token | form (`bodhi-form.css` flush) |
| App Access Request | api-keys / access-requests | form/review; sticky action bar inside flush main |
| Bodhi App Settings | settings / app-settings | React-ified; settings-group nav (collapse-aware), restart banner via `banner`, edit drawer |
| Bodhi MCP Discover v2 | mcp / discover | React-ified; 3-column list/detail, role switch, config slide-over |
| Bodhi MCP Playground | mcp / my-instances | React-ified; tool workspace + result area + exec log |
| Bodhi MCP New Instance | mcp / new-mcp | React-ified form; server combobox + dynamic auth |

Each renders a single `<AppShell>`, collapses to the icon rail (logo · section
icon w/ nav popover · sub-page icons w/ tooltips · page controls · avatar),
hover-resizes, and uses overlay drawers under 768px.

## (a) Divergences found — folded vs. left page-local

- **`api-keys-shell.css` → shared page primitives.** Stripped of all chrome
  (`.app/.main/.topbar/.breadcrumb`, the reset, and the duplicate `:root`
  color block — AppShell + colors_and_type own these). It now holds only the
  cross-page primitives shared by the API-Keys/Settings batch: `page-header`,
  `toolbar`, `filter-tabs`, `search`, `empty-state`, `section-label`,
  `btn-accent/ghost`, and the `.tag-*` set. Consumed by App Tokens, Access
  Requests, New App Token, App Access Request. **Folded (shared).**

- **List-page convergence (App Tokens / Access Requests / Manage Users).**
  The shared *controls* (toolbar, filter-tabs, search, empty-state, tags)
  already live in `api-keys-shell.css` and are reused. The three pages'
  *row presentations genuinely differ* — token card (`.token-card`), request
  card (`.req-card`), and a real `<table>` (`.mu-table`) — so each row/table
  style stayed **page-local** rather than being forced into one shared rule.
  Manage Users keeps its `mu-*` table styling; it did not need
  `api-keys-shell.css`. **Left page-local (intentional).**

- **`bodhi-form.css` legacy chrome removed.** `.bf-shell`, `.bf-content`,
  `.bf-topbar`, `.bf-breadcrumb`, `.bf-bc-*` and their `@media` rules were
  dead once every form mounts inside AppShell (`contentClass="flush"`). Removed;
  the `.bf-*` form **primitives** (scroll/container/card/section/field/inputs/
  radios/footer/buttons) are unchanged and still shared by all form pages.
  **Folded (shared file slimmed).**

- **Semantic colors `--c-connected-*` / `--c-pending-*`.** Not in AppShell's
  palette (which has `--c-leaf-*` / `--c-warning-*`). MCP Discover, MCP New
  Instance, and Playground need the distinct "connected/pending" greens/ambers,
  so a small page-local `:root` block defines them in each MCP page CSS.
  Manage Users needs `--c-red-*` (rejected) — also kept page-local. **Left
  page-local (genuinely page-specific tokens).**

- **`--c-pending` → `--c-warning`.** App Access Request's `.info-pending`
  banner was remapped to AppShell's `--c-warning-*` (same intent) instead of
  re-declaring a token. **Folded (uses shared token).**

- **Sticky action bar (App Access Request).** The old `position:fixed; left:
  var(--side-w)` action bar (which tracked the old resize var) became a normal
  `flex:none` footer inside a flush `ar-main` column — it now respects collapse
  and resize for free. **Fixed via the shell pattern.**

## (b) Dead files removed (grep-confirmed no live references)
- `bodhi-sidebar.js` (vanilla sidebar)
- `bodhi-sidebar-react.jsx` (React `<BodhiSidebar>` + `BSB_NAV`)
- `bodhi-resize.js` (standalone column resizer)
- `Bodhi Models (v1 vanilla).html` (Models rebuild backup)

`bodhi-app-shell.jsx` references `window.BSB_NAV` only as an optional fallback
(`|| [default]`); with the file gone it cleanly uses its own built-in
`SHELL_NAV`. No page loads the deleted scripts. `bodhi-form.css` and
`Layout System.html` are retained as intended.

## Result
One layout system (`bodhi-app-shell.{jsx,css}`), one form primitive sheet
(`bodhi-form.css`), one shared API-Keys/Settings primitive sheet
(`api-keys-shell.css`), and page files that contain only page-unique content.
