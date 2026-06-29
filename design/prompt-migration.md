# Kickoff: Migrate remaining Bodhi pages onto the common layout

## Mission
Bring the **6 remaining pages** onto the shared layout system so the whole app uses one set of layout primitives. This migration is the prerequisite for a follow-up phase: once every page is on the common layout, we'll make **central fixes once** (in shared files) instead of per-page. So a second, equally important goal runs through this work: **maximize use of the shared files and minimize page-local overrides**, and **surface divergences** so they can be folded into the shared system rather than patched locally.

Treat this as *adopting an existing convention*, not designing a new one. When in doubt, match the reference pages — don't invent.

## The system of record (read these FIRST, in order)
- `colors_and_type.css` — design tokens (lotus palette, Inter + JetBrains Mono, radii, shadows). Source of truth for all color/type/spacing values. Never hardcode a hex when a token exists.
- `bodhi-form.css` — the **`.bf-*` form-page system**: `.bf-shell` (grid), 52px `.bf-topbar` (breadcrumb only), centered `.bf-container` (max-width 860px) + `.bf-card`, and all form primitives (`.bf-field`, `.bf-label`, `.bf-input`, `.bf-select`, `.bf-textarea`, `.bf-checkbox`, `.bf-radio-option`, `.bf-footer`, `.bf-btn-*`). Read the header comment — it states the contract.
- `bodhi-resize.js` — `initBodhiResize({ grid, edges, breakpoint })`. Draggable, persisted, always-visible centered grip on column dividers.
- `bodhi-sidebar-react.jsx` — the React `<BodhiSidebar>`. 52px brand row; **auto-wires left-sidebar resize on mount** (props `resizable` default true, `resizeKey`). It calls `initBodhiResize` on `aside.parentElement`, which **must be a CSS grid whose first track is `var(--side-w, …)`**. The `aside` width is `var(--side-w, 220px)` so it works in both grid (fills track) and flex (fixed 220) shells.
- `bodhi-sidebar.js` — the **vanilla** sidebar equivalent (used by the already-migrated non-React pages). Same 52px brand.

### Reference implementations (copy these patterns)
- **Form pages** → `Models-New-API.html` + `create-api-model-app.jsx`, and `Models-New-Local.html` + `local-model-app.jsx` + `local-model.css`. Study how the JSX uses `.bf-shell` → `.bf-topbar` (breadcrumb only) → `.bf-scroll` → `.bf-container` → `.bf-card` → sections → `.bf-footer`, and how page-specific styling (quant table, model picker) lives in a small page CSS file using a page prefix while everything generic comes from `bodhi-form.css`.
- **List / 3-column pages** → `Models-My-Models.html` (full-width main, flat rows, right detail panel, both columns resizable).
- **2-column vanilla page** → `Settings.html`.

## Locked conventions (prescriptive — do not redesign)
1. **52px header gridline.** Sidebar brand, main topbar, and any right-panel header are all exactly **52px** and sit on one unbroken line. Verify `brand.bottom === topbar.bottom === 52`.
2. **Grid columns via CSS vars.** `grid-template-columns: var(--side-w, …) 1fr [var(--panel-w, …)]`. **Never put a `transition` on `grid-template-columns`** — it breaks when the track contains `var()` (Chromium bug). Collapse is instant by design.
3. **Layout per page type:**
   - **List pages** → main content is **full-width**.
   - **Form pages** → main hosts a **center-aligned container** (`.bf-container`, 860px).
4. **Actions live only in a footer at the end of the form** — never in the top breadcrumb bar. The topbar is breadcrumb-only. Remove any duplicate mobile action bars.
5. **Resizable sidebar is part of the layout, not per-page boilerplate.** For React pages: load `bodhi-resize.js` and ensure the shell is a grid with `--side-w`; the sidebar auto-wires the rest. For vanilla pages: call `initBodhiResize` explicitly (see the already-migrated vanilla pages).
6. **Reuse the shared system; don't fork it.** If a page needs something `bodhi-form.css` doesn't have, prefer **extending the shared file** over adding a page-local override — flag it explicitly in your summary so we keep the system central.
7. **Tokens, not literals.** Colors/spacing/radii/shadows from `colors_and_type.css`. No new invented colors.

## The 6 pages (all currently React, using `bodhi-sidebar-react.jsx`)
Their brand row is already 52px-aligned (from the shared sidebar). What's missing: resize wiring, the form system (for forms), footer-only actions, and a layout pass.

**Form / create pages** (apply `bodhi-form.css`):
- `Models-New-Router.html` (+ `create-fallback-model-app.jsx`, `create-fallback-model.css`) — **do this first**; it still rides the old `create-api-model.css`, and its topbar is ~8px off the gridline. Note `create-api-model.css` is *shared* with the API page, so migrate Fallback off it cleanly (mirror how Local Model got its own `local-model.css`).
- `Tokens-New-API.html`
- `App-Access-Review.html`

**List / table pages** (full-width main, no centered container):
- `Tokens-App.html`
- `Users-Manage.html`
- `Access Requests.html`

## Discovery-oriented work (do this per page, before editing)
For each page, investigate and decide rather than assume:
- **Read the page + its JSX + its CSS.** Identify: the shell type (flex vs grid), current header height, where actions currently live, and what's genuinely page-specific vs. a duplicate of something in `bodhi-form.css`.
- **Shell conversion:** the React sidebar's auto-resize only engages if the parent is a **grid with `--side-w`**. List pages on a flex shell will need converting to a grid shell (or for the 3-column ones, the `Bodhi Models` pattern). Confirm the grid before expecting resize to work.
- **Delete duplication:** anywhere a page redefines inputs/labels/buttons/topbar that `bodhi-form.css` now provides, delete the local version and use `.bf-*`. Keep only truly page-unique styles (tables, pickers, chips) in a small page CSS file with a page prefix.
- **Hunt dead/legacy code & misnamed files** (we already found `api-model.*` and `new-model-*` this way). Remove with confidence only after grepping for references.
- **List-page convergence opportunity:** App Tokens / Manage Users / Access Requests are all tables/lists. Look for a shared row/table pattern (mirroring the flat rows in `Bodhi Models`). If 2+ pages want the same thing, that's a candidate to **promote into a shared stylesheet** (propose it; don't silently fork). This directly serves the "central fixes later" goal.
- **`resizeKey` grouping:** decide whether related pages should share a width key (the two create-model pages share `bodhi.createmodel.sideW`) or get their own. Pick deliberately.

## Watch-outs (learned the hard way)
- **Shared files are global.** Touching `bodhi-sidebar-react.jsx`, `bodhi-form.css`, or `bodhi-resize.js` affects all consumers. After any such change, re-verify **every** page that uses it, not just the one you're on.
- **Screenshots lie for flex layouts.** The html-to-image capture renders flex `align-items:center` labels with false line-wrapping and may drop `::after` checkmarks. **Trust `eval_js` DOM measurements** (getBoundingClientRect, getClientRects().length) over screenshots for alignment/wrapping checks.
- **Responsive breakpoints:** form pages fold to single-column and hide the sidebar at ≤760px (`bodhi-form.css`); the 3-column list/detail pages use 1099px with a drawer. Match the right one to the page type.
- **`var()` + grid transition** = stuck values. Already stated, but it's the #1 trap.

## Definition of done (per page)
- 52px gridline verified by measurement (brand/topbar/panel all 52, same row).
- Sidebar resizes (grip visible, drag works, persists, double-click resets) where applicable.
- Form pages: centered 860px container, footer-only actions, inputs are `.bf-*`.
- List pages: full-width main, flat rows, resize wired.
- Page-local CSS contains *only* page-unique rules; generic styling comes from shared files.
- Dead code/misnamed files removed (after grep).
- No console errors; run the background verifier; spot-check the shared-file consumers you touched.

## End-of-migration deliverable
A short note listing: (a) every divergence you found and whether you folded it into a shared file or left it page-local (and why), and (b) a ranked list of **central fixes now unlocked** by everyone being on the common layout — so the follow-up phase can start immediately.

---

## Current migration status (as of this writing)

### On the new layout (8)
- Bodhi Models, Bodhi App Settings, Bodhi Chat, Bodhi MCP Discover v2, Bodhi MCP New Instance, Bodhi MCP Playground (vanilla `bodhi-sidebar.js` + `bodhi-resize.js`)
- Create API Model, Create New Local Model (React + `bodhi-form.css`)

### Remaining (6)
- Forms: Create Fallback Model, New App Token, App Access Request
- Lists: App Tokens, Manage Users, Access Requests

### Not a page
- Layout-System.html — the spec/playground reference for the shell, not an app screen.
