# Batch 4 — Explore · MCP Servers (catalog page)

## Context

Screen-V2 migration has finished the Models section; per the roadmap (`process.md`:
`Foundation → API Keys → Settings → Models → **MCP** → Chat`) the next nav section is **MCP**. This
batch builds **one screen**: a browsable, searchable **Explore · MCP Servers** catalog, fed by the new
Bodhi Reference API `GET /api/v1/mcp-servers`, joined client-side with the user's own configured
instances (`GET /bodhi/v1/mcps`) so each catalog row shows whether the user already has it.

The hi-fi mock (`Bodhi MCP Discover v2.html`) is the UX intent, but it was drawn before the v1 catalog
API existed and depicts a much richer server than the scraped data supports. **Decision rule (per the
user): the shipped Explore · API Models page wins on layout/components; the reference API wins on data.**
Every mock field the v1 API doesn't serve is logged in `mcp-techdebt.md` (companion doc) and either
dropped or derived from the user-instance join — not built as dead UI.

This is a near-clone of the shipped **Explore · API Models** page
(`crates/bodhi/src/routes/models/explore/api/`), reusing its shared catalog kit wholesale: same table,
keyboard nav, per-row `<Link>` link-hints (Vimium), detail rail, numbered pagination, reset button,
column picker, sort-preference, and data-driven facet sidebar. **No new design primitives.**

**Scope decisions (confirmed):** Explore page only (one new MCP nav entry; keep V1 My MCPs at
`/ui/mcps/` untouched). Per-row status from the catalog↔instances join. Facets data-driven from the API
`facets` arrays. Bump `@bodhiapp/reference-api-types → ^0.0.12` (MCP types ship there). Reuse Reset +
Column-picker + sort-preference where they make sense.

## How this batch runs — thin evolutionary slices

Each phase ships **one working vertical slice**: build → **verify live in Claude-in-Chrome** → tests
(unit, and grow the **single** e2e spec with a new `test.step`) → **review styling in Chrome, adjust** →
**commit**. The screen grows feature-by-feature, never "build everything then test." Get a 👍 between
phases. (Per `feedback_phased_vertical_slice_dev`.)

---

## Prerequisites (Phase 0 — fold into Phase 1's commit)

1. **Bump** `crates/bodhi/package.json` `@bodhiapp/reference-api-types ^0.0.11 → ^0.0.12`, `npm install`.
   Verify `McpServerSummary, McpServerDetail, ListMcpServersQuery, ListMcpServersResponse, McpAuthType,
   McpFacets, GetMcpServerResponse` import cleanly (confirmed present in published 0.0.12 `.d.ts`).
2. **No backend changes / no ts-client regen.** The user-instance endpoint (`/bodhi/v1/mcps`,
   `useListMcps()` → `ListMcpsResponse`) already exists.

---

## Phase 1 — Basic list screen consuming the API (no selection, no facets)

**Goal:** a real table of MCP servers rendering from the live reference API, reachable from nav.

- **Hook** `hooks/reference/useMcpServers.ts` (mirror `useCatalog.ts`): `buildMcpServersQuery(params)`
  (`URLSearchParams`, `.append()` for repeatable `category`/`auth`, omit empties),
  `useMcpServers(params)` → `useAnonymousReferenceApi().get<ListMcpServersResponse>(...)` with
  `placeholderData: keepPreviousData`. Add `REF_ENDPOINT_MCP_SERVERS = '/api/v1/mcp-servers'` +
  `referenceKeys.mcpServers(...)` to `hooks/reference/{constants,index}.ts`.
- **Route** `routes/mcps/explore/index.tsx` (clone `models/explore/api/index.tsx`): Zod
  `validateSearch` for `q, sort('name'), order, page` only (facets/select added later). Wrap in
  `<AppInitializer allowedStatus="ready" authenticated>` — match how `/ui/mcps/` gates today.
- **Screen** `routes/mcps/explore/-components/ExploreMcpScreen.tsx` (slim clone of `ExploreApiScreen`):
  `useMcpServers` + `CatalogTable` + columns (`#` · **Server** = monogram/`logo_url` + `name` +
  `description` · **Auth** chip), `ShellSearch` (⌘K, debounced `?q=`), `ShellPagination`
  (`unit="servers"`), `useShellChrome({ breadcrumb })`. Loading skeleton + `ErrorPage` + empty state.
  Reuse `catalog.css` + `list.css` + monogram/tint from `catalog-format`.
- **Nav:** add the **MCP** section with one **Explore · MCP Servers** entry → `/ui/mcps/explore/`
  (mirror the Models section's nav config; keep My MCPs reachable).
- **Verify (Chrome):** `make app.run.live`, open `/ui/mcps/explore/` → 198 servers list, search filters,
  pagination, light/dark, console clean.
- **Test:** `useMcpServers.test.ts` (query build + keys, clone `useCatalog.test.ts`);
  `routes/mcps/explore/index.test.tsx` (list renders from MSW reference-API stub, `?q=` search,
  pagination, empty/error). Start the **single** e2e spec `specs/mcps/explore-mcp.spec.mjs` with
  `test.step('list + search + paginate')` (black-box UI-only; throw in `beforeAll` if env missing;
  `reducedMotion:'reduce'`).
- **Review styling in Chrome → adjust → commit.**

## Phase 2 — Row selection + right detail rail

- **Hook:** `useMcpServerDetail(selected)` → `GetMcpServerResponse`, enabled when selected.
- **Route:** add `select?: string` to the Zod schema.
- **Screen:** wire `?select=` (URL-driven, `replace:true`), `useListKeyNav` (↑/↓/Home/End, no wrap),
  per-row `LinkRow` "Open {name}" (Vimium link-hint), `useViewTransition` on select→rail.
- **Rail** `ExploreMcpRail.tsx` (clone `ExploreApiRail`, **single-scroll, no tabs**): header
  monogram + `name` + subtitle; sections, hiding null ones — **Description** (`details ?? description`),
  **Connection** (`endpoint_url`, static `transport`, `external_link` "Official docs"), **Metadata**
  (`verified`/`featured`, `source`/`sources`, `first_seen_at`/`last_scraped_at` via `fmtDate`).
  Tools/publisher/license/repo **hidden** (null in v1 — `mcp-techdebt.md`). New
  `mcps/explore/-shared/breadcrumbs.ts`.
- **Verify (Chrome):** click row → rail slides in; ↑/↓ moves selection + rail; Vimium hints; back/fwd.
- **Test:** RTL select→rail + detail fetch; add `test.step('select → rail + keyboard nav')` to the spec.
- **Review styling → adjust → commit.**

## Phase 3 — Faceted sidebar (data-driven) + Reset + Column picker

- **Route:** add `category[], auth[], installed?` to the Zod schema (`arrayParam`).
- **Sidebar** `ExploreMcpSidebar.tsx` (clone `ExploreApiSidebar`), **data-driven from response
  `facets`** (not hard-coded): Category rail from `facets.category` (renders nothing when empty in v1),
  Auth pills from `facets.auth` (single `http` chip today), **Verified** pill (`verified`). Wire into
  `useShellChrome({ sidebar })`; `hasActiveMcpFacets()` + reset.
- **Reuse:** `ResetButton` (waterfall: filters → query → none) and `ColumnPicker`/`useHiddenColumns`
  (mark **Verified** column `optional:true`); `useSortPreference` (sort=`name` only → toggle asc/desc).
- **Verify (Chrome):** facet filters narrow results; Category rail hidden (empty); Auth single chip;
  Reset + Columns work; sort toggles.
- **Test:** RTL facet-filter + reset + column-hide; add `test.step('facets + reset')` to the spec.
- **Review styling → adjust → commit.**

## Phase 4 — Join user instances → status + Installed facet

- **Join:** `useListMcps()` in the screen → `Map<normalizedUrl, Mcp>` from `mcps[].mcp_server.url`; join
  each catalog row on `endpoint_url` (normalize trailing slash/case) → derive `installed` + `enabled`.
- **Status column** (derived badge: Installed·Enabled / Installed·Disabled / Not installed) + **Installed**
  facet pill (All / Installed / Not installed) in the sidebar (client-side filter on the joined field).
- **Rail Status section:** joined badge, or a **"Add to My MCPs"** CTA `<Link to="/mcps/new/">`
  (deep-link to the existing V1 create flow — building a V2 form is out of scope, `mcp-techdebt.md`).
- **Verify (Chrome):** with a logged-in user holding ≥1 configured MCP, the matching catalog row shows
  Installed/Enabled; Installed facet filters; CTA deep-links.
- **Test:** RTL — a catalog row whose `endpoint_url` matches a mocked `/bodhi/v1/mcps` instance shows
  Installed/Enabled; Installed-facet filter. Add `test.step('installed join + status')` to the spec.
- **Review styling → adjust → commit.**

## Phase 5 — Final polish + full gate

- Light AND dark, responsive (container-query column drops, sidebar collapse), `read_console_messages`
  = 0 errors across all phases.
- Full gate (`process.md` GATE B): `cd crates/bodhi && npm test` green; the e2e spec green; run the
  **full E2E matrix once** (shared shell/nav touched) — note known pre-existing failures per
  `techdebt.md`. Update `tracker.md` (MCP·Explore ✅) + write `batch-4-mcp-explore-retro.md`.

---

## Reuse map (no change to these)

`routes/models/explore/-shared/*` (`CatalogTable`, `LinkRow`, `ColumnPicker`, `ResetButton`,
`useSortPreference`, `catalog-format`, `catalog.css`, `search-params.arrayParam`); `components/shell/*`
(`ShellSearch`, `ShellPagination`, `useListKeyNav`, `useShellChrome`, `ShellIcon`, `list.css`);
`hooks/reference/useReferenceApi` (`useAnonymousReferenceApi`); `hooks/mcps/useMcpInstances`
(`useListMcps`, `Mcp`/`ListMcpsResponse` from `@bodhiapp/ts-client`); `hooks/useViewTransition`.

## Files

**New:** `hooks/reference/useMcpServers.ts` (+`.test.ts`); `routes/mcps/explore/index.tsx`
(+`index.test.tsx`); `routes/mcps/explore/-components/{ExploreMcpScreen,ExploreMcpRail,
ExploreMcpSidebar,explore-mcp-columns}.tsx`; `routes/mcps/explore/-shared/breadcrumbs.ts`;
`specs/mcps/explore-mcp.spec.mjs`; `mcp-techdebt.md` (done).
**Edit:** `crates/bodhi/package.json`; `hooks/reference/{constants,index}.ts`; V2 sidebar nav config.
**Excluded from commit:** user-owned `design/` files.

## Companion doc

`docs/claude-plans/202606/screen-v2/mcp-techdebt.md` — full mock→API gap table (Status/Tools/Category/
Auth-taxonomy/Publisher/Stats/tabs all logged with the drop-or-derive decision).
