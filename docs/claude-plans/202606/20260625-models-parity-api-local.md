# Parity: My Models + Explore·Local Models ↔ Explore·API/Providers

## Context

The two Explore pages (`/ui/models/explore/api/`, `/ui/models/explore/providers/`) have accumulated a mature UX feature set over the last ~20 commits (see `docs/claude-plans/202606/20260624-explore-api-models-feedback.md`, `20260625-explore-url-select-row-panel.md`, `20260625-explore-parity-models-providers.md`). Two sibling pages lag behind:

- **My Models** — `/ui/models/` (`ModelsScreenV2.tsx`)
- **Explore·Local Models** — `/ui/models/explore/local/` (`LocalDiscoveryScreen.tsx`)

Both still use `useState` for filter/query/page/selection (no URL sync, no back/forward, no shareable links, selection lost on reload), render div-based `.l-listrow` lists instead of semantic tables, and lack the reset button / shared slider / column picker. This plan brings them to parity.

**The key principle is reuse, not duplication** (user emphasized this twice). The Explore API and Providers screens already carry *private, near-identical copies* of `ColSort`, `ColumnPicker`, the `<tr>` row renderer, the 3-state reset button, the `Column` interface, and `DualRangeControl`/`RangeControl`. Confirmed by diff: the two `ColSort` copies differ only in a generic sort type and a `data-testid` prefix; same for `ColumnPicker`. So the plan **extracts these into shared primitives first, refactors the two shipped screens onto them (guarded by their green E2E specs), then builds the two new pages on the same primitives** — collapsing four would-be copies into one. The only page-specific code that remains per page is a thin adapter: a `CatalogColumn<T>[]` array + a key extractor + the screen's URL wiring.

### Intended outcome (the user's feature list, mapped)
- URL sync of filters/query/page with browser back/forward — **both pages**
- `?select` row tracking with history `replace` (restores on reload, doesn't pollute history) — **both pages**
- Arrow up/down row navigation — already present via `useListKeyNav`; **verify survives table migration**
- Div rows → semantic `<table>` (more stable rendering) — **both pages**
- Reset button (3-state: filters → query → none) — **both pages**
- Reuse the compact dual-sided slider on My Models (replace raw `<input type=range>`) — **My Models**
- Remove the "no key" connection text — **My Models**
- `"… N exposed"` subtitle → `Models (N)` heading (saves space) — **My Models**
- My Models sortable common columns: Name, Provider/Repo, Base-URL/Filename (derived across the 4 alias types) — **My Models**
- Remove `Showing 30 · sorted by Column` result bar; move `Showing N` next to Load More — **Local Models**

## Reference patterns (reuse as-is)

- URL state machine + `replace`-selection + sort precedence: `crates/bodhi/src/routes/models/explore/api/-components/ExploreApiScreen.tsx`, `…/providers/-components/ExploreProvidersScreen.tsx`
- Zod search schema + `arrayParam` preprocess: `crates/bodhi/src/routes/models/explore/providers/index.tsx`
- Search↔facets↔params mappers: `crates/bodhi/src/routes/models/explore/providers/-components/explore-providers-search.ts`
- Sort precedence (URL > localStorage > none): `crates/bodhi/src/routes/models/explore/-shared/useSortPreference.ts` (`resolveSortPreference`, `persistSortPreference`)
- Table-row keyboard-nav contract: `crates/bodhi/src/components/shell/useListKeyNav.ts` — finds `.l-scroll` → `.l-listrow`, focuses + clicks each row's `.l-rowlink`. The shared table MUST keep `<tr className="l-listrow">` with `LinkRow` in the first cell (already proven by the shipped Explore tables).
- Already-shared: `catalog.css`, `catalog-format.ts`, `FacetCombobox.tsx`, `LinkRow`, `ShellIcon`, `ShellPagination`, `Slider`.

## Decisions (from interview)

- **My Models columns are universal/derived** across the 4 `Alias` variants (user-corrected from the original "base_url/api_format/#models"):
  - **Name** — alias name. Server-sortable (`sort='alias'`/`'name'` in `routes_models.rs:sort_aliases`).
  - **Provider/Repo** — `repo` (local-file + user-alias), `api_format`/provider (API), first target's `alias` (router).
  - **Base-URL/Filename** — `filename` (local + alias), `base_url` (API), first target's `model` else filename else `—` (router).
  - Provider/Base-URL are **sorted client-side on the current page only** (server sorts name/repo/filename/source). Documented in code; optional backend extension noted as follow-up.
- **Delivery**: phased vertical slices, verify each in Chrome, grow ONE E2E spec per page (many `test.step`), **commit per phase** (feature rollout).
- **Scope**: full parity set.

## Phase ordering & risk read

My Models is lower-risk → first: offset pagination (same as Providers), no backend change, has `data.total` for the heading. Local Models is second: keyset cursor + Load-More accumulation is the page-unique hard part, descending-only sort.

- **Phase 0** — extract shared primitives, refactor Explore API + Providers onto them.
- **Phases 1–3** — My Models (URL state → table+slider+cosmetics → sort+E2E).
- **Phases 4–6** — Local Models (URL state → table+resultbar → reset+E2E).

---

## Phase 0 — Extract shared primitives; refactor Explore API + Providers

**Goal:** Zero behavior change to the two shipped pages; lift their private primitives into `-shared/` and consume them. Regression guard = the existing `api-explore.spec.mjs` + `providers.spec.mjs` + both `index.v2.test.tsx` (must stay green).

**Create in `crates/bodhi/src/routes/models/explore/-shared/`:**
- `catalog-table.tsx`:
  - `interface CatalogColumn<T>` = `{ key; label; width; align?; sort?: string; optional?; cell: (row, idx) => ReactNode }` (generalizes the `Column` inlined at `ExploreApiScreen.tsx:109`).
  - `ColSort` — extracted from `ExploreApiScreen.tsx:66` / `ExploreProvidersScreen.tsx:56` (identical bar the sort type + testid). Add `testIdPrefix` prop; add `descendingOnly?: boolean` (active icon always `arrow-down`, `order` ignored) so Local Models reuses it instead of its bespoke `SortHeader`.
  - `CatalogTable<T>` — `<table className="cat-table">` + `<colgroup>` from `visibleColumns` + `<thead className="cat-listhead">` (sortable col → `ColSort`) + `<tbody className="l-listview">`. Props: `columns`, `rows`, `rowKey`, `activeKey`, `onSelect`, `sort`, `order`, `onSort`, `rowTestId`, `rowLabel`, `startIndex`, `numbered?`, `descendingOnly?`, and an optional `rowData(row)=>Record<string,string>` for extra row attrs (e.g. `data-model-type`).
  - `CatalogRow<T>` — the `<tr className="l-listrow cat-row">` + `columns.map` cells + `num`-cell `LinkRow` (from `ExploreApiScreen.tsx:258`). **Keep `l-listrow` + `.l-rowlink`** (key-nav contract).
- `ColumnPicker.tsx` — extract the Radix `DropdownMenu` checkbox picker (`ExploreApiScreen.tsx:211`). Props: `columns`, `hidden`, `onToggle`, `testIdPrefix`. Also export `useHiddenColumns()` → `{ hidden, toggle, visibleColumns(cols) }` to kill the duplicated `Set<string>` boilerplate.
- `ResetButton.tsx` — extract the `rotate-ccw` 3-state button (`ExploreApiScreen.tsx:498`). Props: `mode: 'filters'|'query'|'none'`, `onReset`, `testId`. Owns aria-label/title waterfall, `data-test-state`, `disabled`.
- `RangeControls.tsx` — move `DualRangeControl` (`ExploreApiSidebar.tsx:360`) + single `RangeControl` (~`:296`) out unchanged (debounced `onValueCommit`, `cat-range-*` hover-reveal).

**Refactor onto extractions:**
- `…/api/-components/ExploreApiScreen.tsx` — delete local `ColSort`/`ColumnPicker`/`ModelRow`/`Column`; import shared; keep only its `COLUMNS` adapter + URL wiring.
- `…/providers/-components/ExploreProvidersScreen.tsx` — same; convert its inline `<table>` to `CatalogTable` with a `CatalogColumn<ProviderSummary>[]` adapter (most code deleted here).
- `…/api/-components/ExploreApiSidebar.tsx` — import `DualRangeControl`/`RangeControl` from `-shared/RangeControls`.

**Verify (Chrome, `make app.run.live`):** Both Explore pages visually identical; sort/picker/reset/sliders behave as before; back/forward unchanged.
**Tests:** No new specs. `npm test` + both E2E specs must pass unchanged. If a `data-testid` moved, fix the `testIdPrefix`, not the spec.
**Commit:** `refactor(explore): extract CatalogTable/ColSort/ColumnPicker/ResetButton/RangeControls to -shared; consume in API+Providers`.

---

## Phase 1 — My Models: URL state machine + Zod schema (div list retained)

**Goal:** URL becomes the single source of truth (filters, query, page, `?select`), replacing the `useState` cluster, while still rendering the existing div rows. Isolates the state migration from the table migration.

**Create:** `crates/bodhi/src/routes/models/-components/models-search.ts`
- `searchToFilter(search): ModelsFilter` / `filterToSearch(filter)` over `types`/`apiFormats`/`capabilities`/`sizeMin`/`sizeMax` (reuse `ModelsFilter` + `buildModelsFilterParams` from `hooks/models/useModels.ts`).
- `searchToListArgs(search, {sort, order})` → the `(page, pageSize, sort, sortOrder, filter)` tuple for `useListModels`; maps `?sort=name` → backend `'alias'`; omits defaults.
- `PAGE_SIZE = 30`; an `arrayParam` helper (factor the route-schema preprocess into `-shared/search-params.ts` and import it from all four route schemas — small dedup win, do it here as the 3rd consumer).

**Modify:**
- `crates/bodhi/src/routes/models/index.tsx` — add `validateSearch` + exported `modelsSearchSchema`: `q`, `select`, `sort: z.enum(['name','provider','base_url'])`, `order`, `page`, facet array params `type`/`api_format`/`capability`, numeric `size_min`/`size_max`.
- `crates/bodhi/src/routes/models/-components/ModelsScreenV2.tsx`:
  - `getRouteApi('/models/')` + `useSearch`/`useNavigate`. Derive `facets`/`page`/`selectedId` from URL (drop those `useState`s).
  - `select()` = `navigate({ replace: true })` + dedup guard + `withViewTransition` (copy API screen).
  - Keep Downloads `railMode` in local `useState` (ephemeral UI, not URL); `?select` drives the model rail; Downloads toggle still overrides locally.
  - `commitSearch`/`onFilterChange`/`onPage` → `navigate`, dropping `page` on filter/search change.
  - Wire `resolveSortPreference` (`storageKey='bodhi.models.sort'`, `NATURAL_ORDER={name:'asc',provider:'asc',base_url:'asc'}`) — exercised in Phase 3.
  - Keep `useListKeyNav()` + the div list this phase.

**Verify (Chrome):** Facet → `?type=…`, back reverts; search → `?q`; row click → `?select` (replace; back doesn't re-add); reload restores rail; page 2 → `?page=2`.
**Tests:** Add `crates/bodhi/src/routes/models/index.v2.test.tsx` (MSW): facet pushes URL, `?select` replace, reload-with-`?select` opens rail. `npm test`.
**Commit:** `feat(models): URL-sync filters/query/page/select (div list retained)`.

---

## Phase 2 — My Models: table + shared slider + cosmetics

**Goal:** Swap div rows → `CatalogTable`; define universal columns; swap raw range inputs → shared `DualRangeControl`; drop "no key"; `Models (N)` heading.

**Modify `ModelsScreenV2.tsx`:**
- Define `COLUMNS: CatalogColumn<AliasResponse>[]` (the only bespoke code):
  - `num` (`#`).
  - `name` — title + type badge (`getAliasTypeMeta`); for API rows keep the `api_format` badge but **delete the `m-conn` "connected / no key" span** (`ModelsScreenV2.tsx:102-107`). `sort:'name'` (server).
  - `provider` (optional) — `aliasProvider(alias)`: repo (local+user-alias), `api_format` upper (API), `targets[0].alias` (router). `sort:'provider'` (client).
  - `base_url` (optional) — `aliasBaseUrl(alias)`: filename (local+alias), `base_url` (API), `targets[0].model` else filename else `—` (router). `sort:'base_url'` (client).
- Add `<h2 className="m-listheading" data-testid="models-heading">Models ({total})</h2>` above the table (the page title is a non-editable breadcrumb; result bar is gone → a small count-heading is the natural home). Add `.m-listheading` to `models.css`.
- Replace `<div className="l-listview">…ModelRow…` with `<CatalogTable<AliasResponse> … startIndex={(page-1)*PAGE_SIZE} numbered rowTestId={a=>'model-row-'+getAliasId(a)} rowData={a=>({'data-model-type':a.source})} />`. Add the shared `ColumnPicker` (optional: provider, base_url) + `ResetButton` to the toolbar.
- **Client-side sort** for `provider`/`base_url`: when active, sort the current page's `rows` in-memory by the derived value honoring `order`; `name` goes to the server. Comment: *derived columns sorted within current page only; cross-page ordering not guaranteed — see follow-up to extend `sort_aliases`.*

**Modify `ModelSidebarFacets.tsx`:** replace the two raw `<input type="range">` (`:134-153`) with shared `DualRangeControl` (GB units, ceiling 16, `maxLabel="16+"`, `onCommit(lo,hi)` → bytes via existing `GB`). Keep `data-testid="models-facet-size"`.

**Modify `models.css`:** add `.m-listheading`; remove dead `.m-conn`/`.m-size*` (grep first).

**Verify (Chrome):** Table with Name/Provider/Base-URL; no "no key"; heading `Models (N)`; picker hides optional cols; dual-slider filters; reset waterfalls; arrow keys still move selection.
**Tests:** Update `index.v2.test.tsx` (new column testids, heading, removed connection text). Create `crates/lib_bodhiserver/tests-js/pages/ModelsPage.mjs` (content, rows, columns btn, sort headers, clear-all, size-slider thumbs, heading); grow `all-models-v2.spec.mjs` (`test.step`: table renders, column hide, heading count). Black-box; `reducedMotion:'reduce'`; settle before asserting. `npm test` + E2E.
**Commit:** `feat(models): table layout, universal columns, shared slider; drop no-key + subtitle`.

---

## Phase 3 — My Models: sort wiring + key-nav verify + E2E completion

**Modify `ModelsScreenV2.tsx`:** `onSort` mirrors API screen (toggle on active col, natural default on new, `persistSortPreference('bodhi.models.sort', …)`, omit natural order, drop `page`). Name → server; provider/base_url → in-memory page sort (Phase 2).
**Modify `all-models-v2.spec.mjs`:** `test.step`s — sort-by-Name writes `?sort` + active header; sort derived col reorders page; `?select` replace + reload-restore + one-Back-skips; reset waterfall; arrow up/down opens rail.
**Verify (Chrome):** Name header → `?sort=name` server reorder; Provider header → client reorder; arrow nav; localStorage sort persists across clean-URL reload but not written to URL.
**Tests:** Full `all-models-v2.spec.mjs` + `npm test`.
**Commit:** `feat(models): URL-synced sort with localStorage precedence; complete E2E`.

---

## Phase 4 — Local Models: URL state (sort/facets/search/select), keep cursor in state

**Goal:** Move sort, facets, search, `?select` into the URL; keep keyset cursor + Load-More accumulation in component state. **Cursor stays out of the URL** (opaque/volatile); `order` is omitted entirely (descending-only).

**Create:** `…/explore/local/-components/local-discovery-search.ts` — `searchToFacets`/`facetsToSearch` over `specialisation`/`pipeline_tag`/`tag`/`language`/`license`/`author` (reuse `DiscoveryFacets` + `facetsToQuery`); `searchToParams(search, cursor)` building `sort` (default `downloads`), `q`, facets + the component-held cursor; `PAGE_SIZE`/`SEARCH_PAGE_SIZE`. No `order` anywhere.

**Modify:**
- `…/explore/local/index.tsx` — `validateSearch` + exported `localDiscoverySearchSchema`: `q`, `select`, `sort: z.enum(['downloads','likes','last_modified','created_at','trending'])` (no `order`), facet array params + `pipeline_tag`.
- `LocalDiscoveryScreen.tsx` — `getRouteApi('/models/explore/local/')`; derive `sort`/`facets`/`committedSearch`/`selectedKey` from URL (drop those `useState`s, lines ~52-57). Keep `cursor`, `extraPages`, `searchInput`, `railMode` in `useState`. Replace imperative `resetPaging()` in handlers with a `useEffect` that clears `cursor`+`extraPages` when the non-cursor request slice (`sort`+`facets`+`q`) changes. Handlers `navigate({search})`; `onSort` omits `downloads` default; `select()` = `navigate(replace)` + dedup + `withViewTransition`. `resolveSortPreference` with `naturalOrder = () => 'desc'`, `storageKey='bodhi.explore.local.sort'`. Keep `useListKeyNav()` + div list this phase.

**Verify (Chrome):** Sort header → `?sort=likes`, no `?order`; Load more works and is NOT in URL; reload preserves sort/facets/`q`/`select` (resets accumulated pages — expected); row select → `?select` (replace).
**Tests:** Grow `…/explore/local/index.v2.test.tsx` (MSW): sort/facet write URL, `?select` replace+restore, Load-More accumulates without touching URL. `npm test`.
**Commit:** `feat(explore-local): URL-sync sort/facets/search/select; cursor stays in component state`.

---

## Phase 5 — Local Models: table via shared CatalogTable (descending-only); remove result bar; count below list

**Modify `LocalDiscoveryScreen.tsx`:**
- `COLUMNS: CatalogColumn<Model>[]` — `num`; `repo` (org/repo/verified/multimodal + tags from `LocalRow`, no sort); `downloads` (`sort:'downloads'`, right, `compact()`); `likes` (`sort:'likes'`); `last_modified` (`sort:'last_modified'`, `fmtDate`). `created_at`/`trending` stay sidebar "Browse" presets.
- Render `<CatalogTable<Model> … descendingOnly sort={sort} onSort={onSort} rowTestId={m=>'ld-row-'+m.namespace+'-'+m.repo} />`. The shared `ColSort descendingOnly` reproduces the old `SortHeader` (active → `arrow-down`, never flips).
- **Delete `.ld-resultbar`** (`:332-337`).
- **Move "Showing N"** below the table: `<div className="ld-listfoot"><span data-testid="ld-count">Showing {rows.length}</span>{showLoadMore && <button data-testid="ld-load-more">…</button>}</div>`.

**Modify `LocalDiscoveryRow.tsx`:** reduce to a `LocalRepoCell` (org/repo/tags markup) + `compact`; delete the `LocalRow` wrapper div + `SortHeader`.
**Modify `local-discovery.css`:** drop `.ld-resultbar`/`.ld-listhead`/`.ld-row`/`.ld-num`; add `.ld-listfoot`; keep `.ld-name`/`.ld-tags`/`.ld-stat*` (reused inside `<td>`).

**Verify (Chrome):** Table renders; headers `arrow-down` when active, never flip; no result bar; `Showing N` next to Load more; Load more appends + count grows; arrow keys navigate.
**Tests:** Update `LocalDiscoveryPage.mjs` (remove resultbar selector, repoint sort selectors to `ld-sort-${col}` via `testIdPrefix='ld-sort'`, add `ld-count`). Grow `local-discovery.spec.mjs` (table renders, no result bar, count below list, descending-only). E2E + `npm test`.
**Commit:** `feat(explore-local): table via shared CatalogTable; remove result bar; count below list`.

---

## Phase 6 — Local Models: toolbar reset + key-nav verify + E2E completion

**Modify `LocalDiscoveryScreen.tsx`:** compute `resetMode` (`hasActiveFacets` → filters, `committedSearch` → query, else none); render shared `<ResetButton mode={resetMode} testId="ld-clear-all" />` in `.m-toolbar` (`onReset` clears facets via `navigate(facetsToSearch({}))` else clears `q`). Keep sidebar per-group "Clear" links.
**Modify `local-discovery.spec.mjs`:** `test.step`s — reset waterfall, arrow up/down opens rail, `?select` replace+reload+one-Back-skips, descending-only sort persists in URL.
**Verify (Chrome):** Toolbar reset clears facets → query → disables; arrow nav; selection round-trips.
**Tests:** Full `local-discovery.spec.mjs` + `npm test`.
**Commit:** `feat(explore-local): toolbar 3-state reset; complete E2E + key-nav verification`.

---

## Cross-cutting risks

- **`useListKeyNav` table compatibility (highest).** It queries `.l-scroll`→`.l-listrow`, focuses/clicks each row's `.l-rowlink` (`useListKeyNav.ts:55-74`). `CatalogTable`/`CatalogRow` MUST keep `<tr className="l-listrow cat-row">` + `LinkRow` in the first `<td>` — exactly as the shipped Explore tables do, so this is already proven against a `<tr>` list. Phase 0 just must not drop the classes; verify with a Chrome step + an E2E arrow-nav `test.step` in Phases 3 and 6.
- **View-transition races.** `select()` wraps `navigate(replace)` in `withViewTransition`. Tests set `reducedMotion:'reduce'`, wait for SPA + list settle before asserting `?select`. Keep the dedup guard.
- **Heterogeneous-column derivation (My Models).** `aliasProvider`/`aliasBaseUrl` switch on `isApiAlias`/`isModelRouterAlias`/`isUserAlias` (router → `targets[0].alias`/`.model`). All fields exist on the response types. Sort is page-local — documented.
- **Descending-only (Local).** No `?order` in schema; `ColSort descendingOnly`; `resolveSortPreference` with `naturalOrder=()=>'desc'`. Prevents any `order=asc` (HF rejects).
- **Cursor vs URL (Local).** Cursor + `extraPages` in `useState`; a `useEffect` resets them when the non-cursor request slice changes. Reload restores filters, not accumulated pages — accepted.
- **Refactor regression (Phase 0).** Existing E2E + `index.v2.test.tsx` are the guard; preserve every `data-testid` via `testIdPrefix`.

## Verification summary (end-to-end)

1. Per phase: `make app.run.live`, exercise the page in Chrome per the phase's "Verify" steps (URL params on filter/search/page/select, back/forward, reload-restore, table render, slider, reset, arrow nav).
2. Component: `cd crates/bodhi && npm test` (the `index.v2.test.tsx` files).
3. E2E: `make test.e2e` from `crates/lib_bodhiserver/tests-js` — grown `all-models-v2.spec.mjs` + `local-discovery.spec.mjs`, plus the unchanged `api-explore.spec.mjs` + `providers.spec.mjs` as the Phase-0 regression guard. Black-box only; `reducedMotion:'reduce'`.
4. `make format` before each commit; commit per phase.

No backend change required (My Models derived-column sort is client-side).

## Optional follow-up (out of scope)

Server-side sort for the derived My Models columns: extend `sort_aliases` in `crates/routes_app/src/models/routes_models.rs:242` with `base_url`/`api_format`/`model-count` cases + `get_alias_base_url`/`get_alias_provider` helpers. Makes Provider/Base-URL sort consistent across pages; defaulting to client-side avoids the OpenAPI→ts-client regen cycle. Clean future PR.
