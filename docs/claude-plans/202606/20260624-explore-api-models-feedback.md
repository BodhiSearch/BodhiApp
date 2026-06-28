# Explore · API Models — feedback iteration (2026-06-24)

Acts on screenshot feedback for `/ui/models/explore/api/`. **Almost entirely frontend.** The
catalog backend already implements the four-param pricing model + all sort keys; the only
backend-repo work is **republishing `@bodhiapp/reference-api-types`** so BodhiApp can consume the
renamed pricing params.

## Key findings (already true — no work)

- `api-getbodhi-app` Worker + zod schema + `RealCatalogReader` already support `pricing_in_min/max`,
  `pricing_out_min/max`, `pricing=free|paid`, and sorts `updated|name|family|providers|context|price|price_out|relevance`.
  `endpoints.md` already documents the four-param contract. **No Worker logic change.**
- `ModelLite` already returns `last_updated` + `release_date`; `ServedBy` returns `base_url` + `pricing`.
- **Gap:** published `@bodhiapp/reference-api-types@0.0.8` (what BodhiApp installs) still ships the OLD
  `pricing_min/max` + `pricing_out_max`. The package `src/` is ahead (already four-param) but unbuilt/unpublished.

## Decisions (from interview)

1. **Per-provider "Add" link** → assume `api_format=openai` for every served-by provider; use that
   provider's own `base_url` + the model id. No `served_by[]` schema change.
2. **Types publish** → I bump + build api-types and prep publish; **user runs `just publish-api-types`**;
   I bump BodhiApp's `package.json` dep to the new version.
3. **Delivery** → phased vertical slices, commit per phase, verify each in Chrome, grow the one E2E
   spec (`api-explore.spec.mjs`) via `test.step`.

---

## Phase 0 — Types republish (api-getbodhi-app repo, gates the frontend)

- Confirm `packages/api-types/src/index.ts` is final (four-param pricing — already done).
- `just build-api-types` to verify it builds clean.
- **User action:** `just publish-api-types` (auto next-patch → likely `0.0.9`, builds, `npm publish`, pushes `api/v0.0.9` tag).
- In BodhiApp: bump `crates/bodhi/package.json` `@bodhiapp/reference-api-types` → `^0.0.9`, `npm install`.
- **Gate:** new `dist/index.d.ts` shows `pricing_in_min/max`, `pricing_out_min/max`, `pricing?: 'free'|'paid'`.

Commit (api-getbodhi-app): `chore(api-types): publish 0.0.9 (four-param pricing)`.
Commit (BodhiApp): folded into Phase 3.

---

## Phase 1 — Table columns: Family + Date columns, sortable headers (`ExploreApiScreen.tsx` + `catalog.css`)

Restructure the row grid into a **column model** so the column-picker (Phase 4) can hide/show columns
and the grid template stays in sync.

- Introduce a `COLUMNS` descriptor: `{ key, label, sort?: ModelSort, width, render }`. Columns:
  `#`, `MODEL` (name + capabilities subheading), `FAMILY` (sortable, `sort:'family'`), `CONTEXT`
  (`context`), `INPUT $` (`price`), `OUTPUT $` (`price_out`), `UPDATED` (sortable date, `sort:'updated'`),
  `PROVIDERS` (`providers`).
- **Family** → own column, value `model.family`, header is a `ColSort` (`col="family"`).
- **Updated** → new column rendering `model.last_updated` as human-readable relative/absolute (add
  `fmtDate(iso)` to `catalog-format.ts`, e.g. "Jun 2026" / "3 mo ago"); header `ColSort col="updated"`.
- **Name header** → make MODEL header a `ColSort col="name"` (currently a plain `<div>`).
- **Capabilities** → MOVE out of its own column into a **subheading line below the model name**
  (small cap-chips under `cat-model-name`, like the family line is today). Remove the standalone
  `CAPABILITIES` header + `cat-caps` column.
- **PROVIDERS cell** → show only the number; drop the `PROVIDERS` sub-label text from the cell
  (`cat-score-lbl`). The column header already says PROVIDERS.
- **Header overflow** → headers single-line with **center ellipsis**: add a `.cat-colhead-ellipsis`
  rule (`overflow:hidden; white-space:nowrap; text-overflow:ellipsis`) — center-ellipsis via
  `direction:rtl; text-align:left` trick or simple end-ellipsis if center proves brittle (note in CSS).
- **Dynamic grid** → `.cat-model-grid` can no longer be a static `grid-template-columns`. Compute it
  inline from visible columns (`style={{ gridTemplateColumns }}`) on both `cat-listhead` and each row.
- Keep `NATURAL_ORDER` for the new sortable cols (`family:asc`, `updated:desc` already present).

**Verify in Chrome:** Family + Updated columns sort on header click (asc/desc toggle); capabilities
render under the name; providers cell shows bare number; headers ellipsize when narrowed.

Commit: `feat(ui): Explore API Models — Family + Updated columns, sortable headers, caps-as-subheading`.

---

## Phase 2 — Remove redundant sort UI (`ExploreApiScreen.tsx`)

Now that Name/Family/Newest sorting lives on the column headers:

- Remove the `cat-sortbar` button group (the `Newest / Name / Family` buttons to the right of search).
- Remove the `cat-resultbar` "sorted by **X** (desc)" text; keep the "Showing X of Y" count.
- Search-as-you-type still auto-switches to Relevance sort internally (no visible Relevance button
  needed; the header chevrons + active state convey current sort). On clearing search, revert to
  `updated` as today.

**Verify in Chrome:** no sort buttons next to search; no "sorted by" line; header chevrons reflect
the active sort including the auto-Relevance-on-search behavior.

Commit: `feat(ui): Explore API Models — drop redundant sortbar + "sorted by" line (headers convey sort)`.

---

## Phase 3 — Pricing filters: four-param model + Free sets output (`ExploreApiSidebar.tsx`, dep bump)

Consume the republished types (Phase 0). Rework `ModelFacetsState` + sidebar + `modelFacetsToQuery`:

- Replace `{ pricing_min, pricing_max, pricing_out_max }` with the four params
  `{ pricing_in_min, pricing_in_max, pricing_out_min, pricing_out_max }` + a `pricing?: 'free'|'paid'`.
- **Two dual-range sliders** (Input price, Output price), each `[min,max]`. Frontend owns the ceilings
  (hardcoded, e.g. input `$30/1M`, output `$60/1M` — pick sensible per-axis ceilings; note rationale).
  Send a bound only when moved off its default (absent = no constraint), matching the docs/`recipes.md`.
- **Free** → a `pricing=free` toggle (not a `pricing_max=0` hack). Per feedback "main cost is output",
  the Free chip sets `pricing:'free'` (backend pins both axes to $0). When Free is active, **grey out /
  disable both price sliders** (frontend nicety; backend ignores redundant bounds) and ignore their values.
- Update `hasActiveModelFacets` + Clear-all for the new shape.
- Bump `crates/bodhi/package.json` dep → `^0.0.9`, `npm install`.

**Verify in Chrome:** input & output dual sliders each filter independently; Free greys the sliders
and returns only $0/$0 models; network requests carry `pricing_in_*`/`pricing_out_*`/`pricing=free`.

Commit: `feat(ui): Explore API Models — four-param pricing sliders + Free toggle (output-aware)`.

---

## Phase 4 — Column picker (`ExploreApiScreen.tsx` + small popover)

- Add a "Columns" control (icon button near the search/right of the header area) opening a popover
  (cmdk/Popover, reuse `FacetCombobox` shell or a simple checkbox list) listing the optional columns
  (Family, Context, Input $, Output $, Updated, Providers — `#`/MODEL always shown).
- Visible-columns state in component state (default = all on); could persist later (out of scope).
- Hidden columns drop from both `COLUMNS`-derived header and rows; `gridTemplateColumns` recomputes.
- `data-testid` per toggle (`cat-model-col-<key>`) for E2E.

**Verify in Chrome:** toggling a column hides/shows it in header + all rows; grid stays aligned.

Commit: `feat(ui): Explore API Models — column picker (show/hide columns)`.

---

## Phase 5 — Provider rail behavior + per-provider "Add" link (`ExploreApiRail.tsx`)

Two changes to the right detail panel's "Served by" list:

1. **Provider click no longer navigates** to the Providers page. With many providers the providers
   list may not even contain the clicked one. Replace each served-by row's
   `<Link to={ROUTE_MODELS_EXPLORE_API_PROVIDERS} select=…>` with a non-navigating row that shows the
   **provider detail inline in the right rail** (selected-provider sub-view within the same rail; e.g.
   expand the row or swap the rail body to provider specs using `useCatalogProviderDetail(slug)`).
   Remove the cross-page navigation entirely from this surface.
   - Audit `ExploreProvidersScreen`'s `select` param consumer — it stays for any other entry points,
     but this rail no longer produces it. (Don't remove the providers page; just stop linking to it here.)
2. **Add icon per provider** — in the last cell of each served-by row (after pricing), render an
   "Add"/plus icon button (`ShellIcon name="plus"` or `circle-plus`) linking to:
   `/ui/models/api/new/?api_format=openai&base_url=<urlenc(provider.base_url)>&model=<urlenc(model.model_id)>`
   - Per decision: `api_format` is hardcoded `openai` for every provider.
   - Omit `base_url` when `provider.base_url` is null (form falls back to preset), keep `model`.
   - `data-testid="cat-model-servedby-add-<slug>"`.
   - The existing top-level "Configure in Bodhi" CTA stays (uses `detail.bridge`).

**Verify in Chrome:** clicking a served-by provider shows its detail inline (no route change to
Providers); the per-row Add icon lands on the New API Model form prefilled with `openai` + that
provider's base_url + the model id.

Commit: `feat(ui): Explore API Models — inline provider detail in rail + per-provider Add-model link`.

---

## Phase 6 — Tests + docs

- **Component tests** (`crates/bodhi`, Vitest+MSW): cover header sort for Family/Updated, caps-as-
  subheading render, providers-bare-number, column-picker hide/show, four-param pricing query
  emission, Free disables sliders, per-provider Add href, provider-click-stays-on-page.
- **E2E** (`api-explore.spec.mjs` + `ApiExplorePage.mjs`): grow the single test with `test.step`s:
  sort by Family + Updated headers; toggle a column off/on; Free toggle greys sliders & filters;
  served-by provider opens inline detail (no URL change to providers); Add icon → New API Model URL
  with `api_format=openai&base_url=…&model=…`. Add page-object selectors/methods. Update `stubCatalog`
  fixtures for `last_updated` + provider `base_url`.
- **Docs:** note in BodhiApp crate docs if needed; `endpoints.md`/`recipes.md` already document the
  four-param contract (no change). Add CHANGELOG entry in api-getbodhi-app for the types publish.

Commit: `test(ui): Explore API Models feedback — component + E2E coverage`.

---

## Gate checks before each commit

`cd crates/bodhi && npm run lint && npm run format && npm test` for FE phases; `make test.e2e` (from
`lib_bodhiserver/tests-js`, dev-server + Vite HMR — no ui-rebuild) once Phase 5/6 land. Rebase onto
`origin/main` before pushing (trunk-based, linear history).

## Open risk

- Center-ellipsis on headers is finicky cross-browser; if the rtl trick misbehaves, fall back to
  end-ellipsis and flag it (documented in the CSS). Low impact.
