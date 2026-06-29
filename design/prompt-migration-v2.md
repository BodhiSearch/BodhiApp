# Kickoff v2: Migrate remaining Bodhi pages onto the **AppShell** layout system

## What changed since v1
The app now has **one canonical React layout component — `AppShell`** (`bodhi-app-shell.jsx` + `bodhi-app-shell.css`). It supersedes the three older, parallel layout systems described in `prompt-migration.md` (the vanilla `bodhi-sidebar.js`, the React `bodhi-sidebar-react.jsx`, and the standalone `bodhi-resize.js` + 52px `.bf-topbar` gridline). **Do not use or extend those old systems.** Every page must be a React app mounted into `#root` that renders a single `<AppShell>`.

The goal is unchanged: **one layout system, zero duplication, page files contain only page-unique content.** Once all pages are on `AppShell`, central fixes happen once in the shared files.

## The new conventions (prescriptive — match the reference pages, don't redesign)
1. **`AppShell` owns all chrome.** Sidebar, brand, primary nav, breadcrumb bar, the sidebar-collapse toggle, hover-reveal column resize, the optional right rail, mobile drawers, and all responsive behavior live in `AppShell`. Pages never hand-roll any of it.
2. **56px header gridline** (`--shell-header-h`, was 52px in v1). Sidebar brand row, main header, and rail header all sit on it automatically — you don't manage this.
3. **Collapse-to-icon-rail.** The toggle at the **start of every breadcrumb** collapses the left sidebar to a 60px icon rail: logo → section icon (click = nav dropdown popover) → current section's sub-page icons (hover = tooltip, click = navigate) → **page controls as icons** → avatar. Collapse state is ephemeral (resets each load), by design.
4. **Hover-only resize.** Column dividers show a grip only on hover; drag to resize (persisted per `resizeKey`), double-click to reset. `AppShell` wires this — pages do nothing.
5. **Responsive:** ≥768px → sidebar + main + (optional) rail all inline; panels are capped to a viewport share so main never starves. <768px → both panels become overlay drawers over a scrim, and the sidebar opens **full** (never the icon rail). All handled by `AppShell`.
6. **Tokens, not literals.** All color/type/spacing/radii/shadows from `colors_and_type.css`. Semantic color helpers (`--c-indigo-*`, `--c-lotus-*`, `--c-saffron-*`, `--c-leaf-*`, `--c-teal-*`, `--c-warning-*`) are defined in `bodhi-app-shell.css` for both themes — use them.
7. **Reuse, don't fork.** Page-local CSS holds only page-unique rules (tables, pickers, chat bubbles, etc.). If multiple pages want the same thing, promote it into a shared file and flag it.

## Read these FIRST, in order
- `bodhi-app-shell.jsx` — the component + its full prop doc (header comment). Exports: `AppShell`, `ShellNav`, `ShellIcon`, `ShellModeSwitch`, `ShellFilterGroup`, `useShell`, `ShellContext`, `SHELL_NAV`.
- `bodhi-app-shell.css` — the layout CSS; read the top comment for the grid diagram and the class vocabulary (`.shell-*`, `.shell-railbtn`, `.shell-tip`, `.shell-pop`, `.shell-fc`, `.shell-mode-*`).
- `colors_and_type.css` — design tokens.

### Reference implementations (copy these patterns exactly)
- **3-column list/detail page** → `Models-My-Models.html` + `bodhi-models-app.jsx` + `bodhi-models.css` + `bodhi-models-data.js`. Shows: `sidebar` slot with `<ShellModeSwitch>` + `<ShellFilterGroup>`, `contentClass="flush"` with `mainScroll={false}` for a self-managed scroll region, a `rail` + `railHeader` detail panel, and `useShell().openRail()` to open the rail on selection (mobile drawer).
- **Form page (centered, no rail)** → `Models-New-API.html` + `create-api-model-app.jsx` (also `local-model-app.jsx`, `create-fallback-model-app.jsx`). Shows: `bodhi-form.css` `.bf-*` form primitives inside `AppShell` children, `contentClass="flush"` + `mainScroll={false}`, breadcrumb array, no rail.
- **Vanilla→React conversion of a 3-column app** → `Chat.html` + `bodhi-chat-app.jsx` + `bodhi-chat.css`. The closest template for the MCP pages. Shows: sidebar with a collapse-aware body (renders icon-rail buttons when `useShell().collapsed`), a flush non-scrolling main with its own scroll, a tabbed `rail`, breadcrumb as a custom node, and `openRail`/`collapseRail` from context.

## `AppShell` API cheat-sheet (see the JSX header for the full list)
```jsx
<AppShell
  section="models"            // SHELL_NAV id → highlights nav + drives sub-pages
  subPage="my-models"         // optional sub-page id
  resizeKey="models"          // localStorage namespace for column widths
  sidebarWidth={240} railWidth={340} headerHeight={56} bandHeight={52}
  breadcrumb={[{label,href}, {label,current:true}]}  // OR a custom node
  headerActions={<…/>}        // right side of header band
  sidebar={<…/>}              // body below the primary nav (filters, history, etc.)
  toolbar / sidebarToolbar / railToolbar={<…/>}      // optional shared band row
  banner={<…/>}               // optional alert sub-band under the header
  rail={<…/>} railHeader={<…/>} railDefaultOpen      // optional 3rd column
  contentClass="narrow|wide|flush" mainScroll railScroll
>
  {main content}
</AppShell>
```
- **Sidebar building blocks** (read collapse from context, auto-render icon+popover when collapsed): `<ShellModeSwitch value onChange options={[{id,label,sub,icon}]} />` and `<ShellFilterGroup icon label note clearable chips={[{label,color,defaultOn}]} />`.
- **`useShell()`** → `{ collapsed, isMobile, openRail(), closeRail(), collapseRail() }`. Use `openRail()` when the user selects an item that should reveal the rail (esp. on mobile).
- **`ShellIcon`** → `<ShellIcon name="lucide-name" size={14} />`. Use everywhere for icons.
- **`SHELL_NAV`** (top of `bodhi-app-shell.jsx`) is the single nav config (sections + sub-pages + hrefs + badges). If a migrated page's sub-pages or hrefs are wrong, fix them **here**, once.

## Standard page HTML (every page looks like this)
```html
<!doctype html><html lang="en" data-theme="light"><head>
  <meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Bodhi · …</title>
  <link rel="stylesheet" href="colors_and_type.css">
  <link rel="stylesheet" href="bodhi-app-shell.css">
  <link rel="stylesheet" href="bodhi-form.css">      <!-- form pages only -->
  <link rel="stylesheet" href="PAGE.css">             <!-- page-unique only -->
  <link href="…Google Fonts…" rel="stylesheet">
  <script src="https://unpkg.com/lucide@latest"></script>
</head><body>
  <div id="root"></div>
  <!-- pinned React + Babel (copy exact tags+integrity from Models-My-Models.html) -->
  <script type="text/babel" src="tweaks-panel.jsx"></script>   <!-- if the page has tweaks -->
  <script type="text/babel" src="bodhi-app-shell.jsx"></script>
  <script type="text/babel" src="PAGE-app.jsx"></script>
</body></html>
```
Mount with `ReactDOM.createRoot(document.getElementById('root')).render(<PageApp/>)`.

## Migration recipe (per page)
1. **Read** the page HTML + its JSX + its CSS. Identify the real content (the stuff inside the old shell) vs. chrome that `AppShell` now provides.
2. **React-ify if vanilla** (MCP pages, App Settings): convert the static markup into a React app. Lift the page's interactive bits (tabs, toggles, accordions) into small components — mirror `bodhi-chat-app.jsx`.
3. **Wrap in `AppShell`:** move the page body into children (or the `sidebar`/`rail` slots); pass `section`/`subPage`/`breadcrumb`/`headerActions`. Delete the old sidebar markup, topbar, resize calls, and any hand-rolled collapse/drawer/responsive CSS.
4. **Strip the page CSS** down to page-unique rules; delete anything `AppShell`/`colors_and_type` now own. Add a header comment "load AFTER bodhi-app-shell.css".
5. **Self-scrolling regions:** for full-height pages (lists, chat, playground) use `contentClass="flush" mainScroll={false}` and give your inner content `display:flex;flex-direction:column;height:100%` with the scroll child getting `min-height:0; overflow-y:auto`. (The flexbox `min-height:0` trap bit us on the form pages — remember it.)
6. **Verify:** `ready_for_verification`. Check collapse (icon rail + tooltips + nav popover), hover-resize, row/tab interactions, and the rail-on-mobile drawer. Then move to the next page.

## The 9 remaining pages
**MCP batch (vanilla `bodhi-sidebar.js` → React `AppShell`, `section="mcp"`)** — use `bodhi-chat-app.jsx` as the template:
- `MCP-Explore.html` (subPage `discover`)
- `Bodhi MCP Playground.html` (subPage `my-instances` or a playground sub-page — check `SHELL_NAV`)
- `MCP-New-Instance.html` (subPage `new-mcp`) — this is a **form**, use `bodhi-form.css`.

**API Keys / Settings batch (React `bodhi-sidebar-react.jsx` → `AppShell`)** — these already have app JSX, so it's mostly swapping the shell wrapper like the create pages:
- `Tokens-App.html` + `app-tokens-app.jsx` — list page (`section="api-keys"`, subPage `app-tokens`).
- `Access Requests.html` + `access-requests-app.jsx` — list page (subPage `access-requests`).
- `Users-Manage.html` + `manage-users-app.jsx` — list page (`section="settings"`, subPage `manage-users`).
- `Tokens-New-API.html` + `new-app-token-app.jsx` + `model-access-picker.jsx` — form (subPage `new-token`).
- `App-Access-Review.html` + `access-request-app.jsx` + `model-access-picker.jsx` — form/review page.
- `Settings.html` (vanilla) — settings page (`section="settings"`, subPage `app-settings`); React-ify like Chat.

> **List-page convergence:** App Tokens / Access Requests / Manage Users are all tables. Look at the flat-row pattern in `bodhi-models.css`; if 2+ tables want the same row/table styling, promote it to a shared stylesheet rather than copying. Flag it in the summary.

## Watch-outs (learned the hard way this phase)
- **Never transition `grid-template-columns`** with `var()`/`minmax` tracks — Chromium leaves the track stuck at its old width. `AppShell` already avoids this; don't reintroduce it. Collapse is instant by design.
- **Panels can starve main.** `AppShell` caps each panel to a viewport share (`min(width, 32vw/44vw)`); keep that if you touch the grid.
- **The scrim is mobile-only.** It must only dim behind the `<768px` drawers — never when an inline rail opens on desktop/tablet. (Fixed once; don't regress.)
- **Screenshots can't show inner-scroll position** (the html-to-image capture resets scrollTop). Verify scroll with `eval_js` measuring `scrollHeight/clientHeight`, not screenshots.
- **Shared files are global.** After editing `bodhi-app-shell.{jsx,css}`, `bodhi-form.css`, or `colors_and_type.css`, re-verify a sample of *every* page type that consumes them, not just the current page.
- **Collapse-aware sidebar content:** any custom `sidebar` slot must check `useShell().collapsed` and render compact icon-rail buttons (`.shell-railbtn.shell-tip` with `data-tip`) in that mode — see `ChatSidebar` and `ModelsSidebar`.

## Cleanup at the very end (after ALL pages are on AppShell — grep first!)
Once nothing references them, delete the dead layout files:
- `bodhi-sidebar.js`, `bodhi-sidebar-react.jsx`, `bodhi-resize.js`
- `Bodhi Models (v1 vanilla).html` (kept as a backup during the Models rebuild)
- Any orphaned page CSS/JS the migration leaves behind (grep before deleting).
Keep `bodhi-form.css` (still used by form pages) and `Layout-System.html`/`layout-system.jsx` (the spec/playground for the shell, already on AppShell — not an app screen).

## Definition of done (per page)
- Renders a single `<AppShell>`, no console errors, background verifier run.
- Collapse to icon rail works (logo · section icon w/ nav popover · sub-page icons w/ tooltips · page controls · avatar); expands back; resets on reload.
- Hover-reveal resize works (grip on hover, drag persists, double-click resets).
- Responsive: inline ≥768px (main never starves), overlay drawers <768px (sidebar opens full, scrim only on mobile).
- Page CSS contains only page-unique rules; chrome comes from shared files.
- Correct `section`/`subPage` highlight; breadcrumb starts with the collapse toggle.

## Status (as of this writing — June 2026)
**On AppShell (done, 5 app pages + 1 spec):**
- Bodhi Models · Bodhi Chat · Create API Model · Create New Local Model v4 · Create Fallback Model
- Layout System (shell spec/playground — not an app screen)

**Remaining (9):** the MCP batch (3) + the API Keys/Settings batch (6) listed above.

## End-of-migration deliverable
A short note listing (a) every divergence found and whether it was folded into a shared file or left page-local and why, and (b) the dead files removed after grep — confirming the app now runs on a single layout system.
