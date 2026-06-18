# UI V2 Migration — Design Reference (the prototype)

The redesign ships as a **high-fidelity static prototype** in the repo's top-level `design/`
directory (path from repo root: `design/`). It is the **visual source of truth** for what each
screen should look like and how the shell behaves — but it is **reference material, not code to
copy**. See "How we use it" below.

## What's in `design/`

- **Per-screen prototypes** — each screen is a `*.html` page + a `*-app.jsx` React app (Babel-in-
  browser) + a `*.css` page stylesheet. The screens map to our 5 nav sections (Chat, Models, MCP,
  API Keys, Settings).
- **The shared foundation** — `bodhi-app-shell.jsx` + `bodhi-app-shell.css` (the canonical
  `AppShell` layout: sidebar/main/optional-rail grid, collapse-to-icon-rail, hover resize, mobile
  drawers, and helper components `ShellNav`/`ShellIcon`/`ShellSearch`/`ShellModeSwitch`/
  `ShellFilterGroup`/`useShell`). `colors_and_type.css` (design tokens — already in shadcn-
  compatible HSL form, dual light/dark).
- **Shared design components** — `model-access-picker.jsx` (model/MCP access picker used by New App
  Token + Access Request review), `bodhi-form.css` (form primitives), `api-keys-shell.css` (shared
  list/toolbar/tag primitives).
- **Roadmap & notes** — `Migration Roadmap.html` (the risk×traffic ordering and strangler
  rationale), `prompt-migration.md` + `prompt-migration-v2.md` + `migration-v2-complete.md` (the
  prototype's own internal migration notes — useful for the AppShell conventions and the
  "fold shared, keep page-local" discipline). `Layout System.html` is the shell spec/playground.

> The exact file list changes; `ls design/` to see the current set rather than trusting a snapshot.

## How to view it visually

The prototype is served over a static HTTP server on **http://localhost:8000/** (a plain directory
listing of `design/`). Open a screen by navigating to its `.html` (URL-encode spaces), e.g.
`http://localhost:8000/Bodhi%20Models.html`.

**Use Claude-in-Chrome to walk it visually:**
1. Load the browser tools (one batched `ToolSearch`):
   `select:mcp__claude-in-chrome__tabs_context_mcp,mcp__claude-in-chrome__navigate,mcp__claude-in-chrome__computer,mcp__claude-in-chrome__browser_batch,mcp__claude-in-chrome__get_page_text`
2. `tabs_context_mcp` (createIfEmpty) → `navigate` to a screen URL → `computer` screenshot.
3. Batch navigations + screenshots with `browser_batch` to sweep multiple screens fast.
4. To see the live, interactive shell behaviors (collapse-to-icon-rail, hover-resize grip, 3rd-rail
   open, responsive drawers under 768px), interact in the browser — screenshots alone won't show
   inner-scroll position or hover states.

If the server isn't running, ask the user to start it (the prototype lives in `design/`; serve that
directory on :8000).

## How we use it — visually yes, technically no

- **Reference it visually.** Match the layout, spacing, hierarchy, states, and interactions of the
  prototype screen-for-screen.
- **Build to OUR conventions, not the prototype's.** The prototype is Babel-in-browser, global
  `lucide`, `window.*` exports, `ReactDOM.createRoot` per page, `data-theme` setattr + a TweaksPanel
  — **none of that ships.** Our production app is Vite + ESM + TanStack Router + TanStack Query +
  shadcn/ui + `lucide-react` + a `ThemeProvider`. When porting, translate the prototype's structure
  into idiomatic production TSX and reuse existing production primitives/hooks.
- **Tokens map cleanly.** `colors_and_type.css` is already shadcn-HSL (`--background`, `--primary`,
  `--card`, `--border`, `--radius`, …), so it merges into our token system rather than living as a
  parallel stylesheet.
- **The prototype's data is sample data.** Real data comes from our existing domain hooks
  (`/bodhi/v1/*`) and, for new catalog-style screens, the reference API ([reference-api.md](@reference-api.md)).
- See [architecture.md](@architecture.md) for the production side and [process.md](@process.md) for
  the strip-on-port rules and risks.
