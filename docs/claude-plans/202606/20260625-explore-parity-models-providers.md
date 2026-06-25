# Plan: Explore · API Providers — parity with Explore · API Models + provider-specific changes

## Context

The **Explore · API Models** page (`/ui/models/explore/api/`) recently received URL-sync,
a 3-state toolbar reset, removal of the result bar, and stacked hover-reveal sliders. The
sibling **Explore · API Providers** page (`/ui/models/explore/api-providers/`) still lags
behind: all of its sort/filter/page state lives in `useState` (no URL sync, no Back/Forward),
its clear-all sits in the sidebar (layout shift), it shows a "Showing 30 of 144 · sorted by
Rank (desc)" result bar, and its pricing slider uses the old inline layout.

Beyond closing those gaps, this work also reshapes the Providers page around provider-specific
intent: the page should be about **which providers to use**, not a rank leaderboard. So we drop
"rank" and "cheapest" sorting entirely, add a **Labs** category filter (show only first-party
labs, not aggregators), add a sortable **API format** column, trim the api_format filter to only
values the search API actually returns, and finally **rename the route to `/ui/models/explore/providers/`**
(moving the files, not just the URL).

**Two cross-cutting requirements (apply to BOTH pages, not just Providers):**
1. **No-sort-on-first-load + localStorage-persisted sort preference** is NEW behavior — neither
   page has it today (Models currently hard-defaults `sort='updated'`). Build it once and apply it
   to **both** the Models and Providers pages.
2. **Share components and logic** between the two near-identical pages rather than copy-pasting.
   Extract the common mechanics into the existing `routes/models/explore/-shared/` directory and
   have both screens consume them.

The reference page is the source of truth for the shared mechanics (URL-as-state, mappers,
3-state reset, hover-reveal). Read its files and copy the discipline:
- `crates/bodhi/src/routes/models/explore/api/index.tsx` — `arrayParam` z.preprocess helper, search schema, defaults stripped from URL.
- `crates/bodhi/src/routes/models/explore/api/-components/ExploreApiScreen.tsx` — state derived from `useSearch()`, single read-only effect syncing `searchInput` DOWN, 3-state `resetMode` waterfall (lines ~418-507).
- `crates/bodhi/src/routes/models/explore/api/-components/explore-api-search.ts` — `searchToFacets` / `facetsToSearch` / `searchToParams`, `DEFAULT_SORT` / `DEFAULT_ORDER` / `PAGE_SIZE`.
- `crates/bodhi/src/routes/models/explore/api/index.v2.test.tsx` — real memory-router harness (`@/test-utils/router-harness`); the 3-state reset test (lines ~542-584) is the template.

Shared infra already fixed (verify, don't redo): `components/ui/slider.tsx` (one Thumb per value),
`components/shell/AppShell.tsx` (memoized `useShell()`), `explore/-shared/catalog.css` (`.l-scroll`,
`.cat-colsort`, `.cat-range-stack`/`.cat-range-val`/`.visible` hover-reveal classes).

## Decisions (locked with user)

- **Filter placement:** match Models — facets stay in the **left sidebar**; only the 3-state reset + sort controls live in the toolbar. No header-bar facets.
- **Labs filter:** a **toggle, OFF by default** (page shows all providers; user opts into labs-only). Uses the API's `is_lab=true` query param.
- **API-format column:** new sortable column inserted **after the provider name** column, before MODELS.
- **Rank:** removed as a SORT key. Keep the leading `#1, #2…` as a **plain per-page row index** (non-sortable).
- **Cheapest sort:** removed entirely.
- **Pricing facet:** keep **Free/Paid** pills; **remove the price-max slider** (so no Models-style range control needed here).
- **Default sort:** **no `sort` param on first load** → API natural order. Persist the user's chosen sort in **localStorage** and apply it on subsequent loads. Precedence on load: **URL `?sort=` > localStorage > none**. When localStorage supplies the sort, **apply it to the API request silently — do NOT write it to the URL** (URL stays clean until the user explicitly clicks a sort).
- **api_format filter options:** render **only the values present in the API's `facets.api_format` bucket** (`{openai, anthropic, gemini, other}` in current data). This auto-drops the frontend-introduced `openai_responses` / `anthropic_oauth` (and any future synthetic value) without a hardcoded denylist.
- **Route rename:** `/ui/models/explore/api-providers/` → `/ui/models/explore/providers/`, moving the directory too.
- **Sharing strategy:** extract common logic into `routes/models/explore/-shared/` (which already
  holds `catalog-format.ts`, `catalog.css`, `FacetCombobox.tsx`, `breadcrumbs.ts`). Both screens
  consume the shared primitives; each keeps only its page-specific facet/column/schema config.
- **localStorage convention:** follow the existing `bodhi.<key>.…` namespace used by `AppShell.tsx`
  (e.g. `bodhi.explore.api.sort`, `bodhi.explore.providers.sort`), keyed per page.

## Files to change

| Concern | Path |
|---|---|
| **NEW** shared sort hook | create `routes/models/explore/-shared/useSortPreference.ts` (+ test) |
| **NEW** shared reset | `routes/models/explore/-shared/` reset component/hook (factored from Models) |
| Models page (no-default-sort + persist) | `…/api/-components/explore-api-search.ts`, `…/api/-components/ExploreApiScreen.tsx`, `…/api/index.v2.test.tsx` |
| Route + search schema | move `routes/models/explore/api-providers/index.tsx` → `routes/models/explore/providers/index.tsx`; `createFileRoute('/models/explore/providers/')` |
| Screen | move `…/api-providers/-components/ExploreProvidersScreen.tsx` → `…/providers/-components/…` |
| Sidebar | move `…ExploreProvidersSidebar.tsx` |
| Rail | move `…ExploreProvidersRail.tsx` |
| **NEW** mappers | create `…/providers/-components/explore-providers-search.ts` |
| Component test | move + rewrite `…/providers/index.v2.test.tsx` |
| Route constant | `crates/bodhi/src/lib/constants.ts:30` — `ROUTE_MODELS_EXPLORE_API_PROVIDERS = '/models/explore/providers/'` |
| Nav entry | `crates/bodhi/src/components/shell/shell-nav-config.tsx:56` (`explore-api-providers` `to`) |
| Models→Providers cross-link | `…/api/-components/ExploreApiRail.tsx:174` (`to="/models/explore/providers/"`) |
| Models test asserting cross-link | `…/api/index.v2.test.tsx:289` |
| Shared sidebar import in test | `…/explore/-shared/catalog-query.test.ts:5` (update import path) |
| Generated route tree | `crates/bodhi/src/routeTree.gen.ts` — regenerated automatically by the TanStack Router Vite plugin on `npm run dev`/`build`; do not hand-edit |
| E2E spec + page object | `crates/lib_bodhiserver/tests-js/specs/models/api-providers.spec.mjs` + `pages/ApiProvidersPage.mjs` (rename to `providers.spec.mjs` / `ProvidersPage.mjs`, update URL) |

Data facts confirmed: `ListProvidersQuery` supports `is_lab?: 'true' | 'false'`, `sort?:
'rank'|'name'|'model_count'|'api_format'|'pricing'`, `q`, `capability[]`, `api_format[]`,
`pricing`, `pricing_max`, `page`, `page_size`. `ProviderSummary` has `is_lab: boolean`,
`api_format_hint: ApiFormatHint`, `rank`, `model_count`, `capabilities_summary`,
`pricing_summary`. Hook: `useCatalogProviders()` in `hooks/reference/useCatalog.ts`. Sample
`facets.api_format` = `{openai:96, anthropic:4, gemini:2, other:43}` (no `openai_responses`/
`anthropic_oauth`). Note: `is_lab` is NOT currently in the providers `facets` bucket — the Labs
toggle uses the query param, not a facet count.

## Implementation phases

Follow the existing screen-v2 cadence: thin slices, verify each in Chrome, grow ONE E2E spec,
commit per phase (this is a feature rollout — commit per phase, not deferred).

### Phase 0 — Shared sort-preference hook (NEW, used by BOTH pages)
Build the persisted-sort behavior once, in `-shared/`, before touching either page.
- Create `routes/models/explore/-shared/useSortPreference.ts` — a small hook/util that resolves the
  **effective sort** with precedence **URL `?sort=` > localStorage > none**, persists the user's
  explicit pick to localStorage, and tells the caller whether the active sort came from the URL
  (write to URL on user action) or from storage (apply to request only, keep URL clean).
  Generic over the page's sort-key union; takes a `storageKey` (`bodhi.explore.<page>.sort`),
  the valid sort keys, and the current URL `sort`/`order`. Returns `{ effectiveSort, effectiveOrder,
  persist(sort, order) }` where `effectiveSort` is `undefined` when nothing is set (→ API natural order).
  Pure/SSR-safe localStorage access (guard `typeof window`), and the read is done in a way that
  doesn't trigger the render loop (read once + on explicit change, never an effect that writes the URL).
- Unit-test it directly (`useSortPreference.test.ts`): URL wins over storage; storage applied when URL
  clean; none → undefined; `persist` writes storage; invalid stored key ignored.
- Commit: `feat(ui): shared sort-preference hook (URL>localStorage>none) for Explore pages`.

### Phase 0b — Apply no-default-sort + localStorage to the Models page
Bring the reference page to the new behavior (it currently hard-defaults `sort='updated'`).
- In `explore-api-search.ts`, stop forcing `DEFAULT_SORT` in `searchToParams` — emit `sort`/`order`
  only when an effective sort exists (mirror the Providers contract). Keep `DEFAULT_ORDER`/`PAGE_SIZE`.
- In `ExploreApiScreen.tsx`, derive the effective sort from `useSortPreference` (key
  `bodhi.explore.api.sort`); on a sort-header click, write to URL + persist; on clean-URL load with a
  saved pref, apply to the request silently (no navigate). First load with no URL sort and no saved
  pref → no `sort` param → API natural order (relevance).
- Update `api/index.v2.test.tsx`: the default-sort assertions change (clean first load sends no
  `sort`; a saved pref drives the request without a URL `?sort=`; explicit click writes URL + storage).
- Commit: `feat(ui): no-default-sort + persisted sort preference on Explore API Models`.

### Phase 1 — Route rename + file move (mechanical, isolating)
Do this first so every later phase edits the final paths.
- `git mv` the `api-providers/` directory to `providers/`; rename `ExploreProviders*` files in place (component names stay `ExploreProviders*` — only the directory/route path changes).
- Update `createFileRoute('/models/explore/providers/')` in the moved `index.tsx`.
- Update the 5 reference sites (constants, nav config, ExploreApiRail link, Models test assertion, catalog-query.test import).
- Let `npm run dev` regenerate `routeTree.gen.ts`; confirm the old route 404s and the new one loads.
- Commit: `refactor(ui): move Explore API Providers route to /models/explore/providers`.

### Phase 2 — URL sync + mappers (the big one)
Mirror Models exactly.
- **Schema** (`providers/index.tsx`): extend `providersSearchSchema` (keep `select`, `q`) with
  `q?`, `capability?: Capability[]`, `api_format?: ApiFormatHint[]`, `pricing?: 'free'|'paid'`,
  `is_lab?: 'true'`, `sort?: 'name'|'model_count'|'api_format'`, `order?: 'asc'|'desc'`,
  `page?` (positive int). Reuse the `arrayParam` z.preprocess helper from `api/index.tsx`.
  Note: **drop `rank` and `pricing` from the sort enum**, drop `pricing_max`. Defaults are
  `.optional()` and never written to the URL.
- **Mappers** (`explore-providers-search.ts`, pure + unit-tested): `searchToFacets`,
  `facetsToSearch` (reuse `providerFacetsToQuery` from the sidebar), `searchToParams` (apply
  `order`/`page`/`page_size` defaults; `PAGE_SIZE=30`). For sort, see Phase 5 — `searchToParams`
  emits `sort`/`order` only when the *effective* sort (URL or localStorage) is set; omit when none.
- **Screen**: replace `page/search/sort/order/facets` `useState` with values DERIVED from
  `getRouteApi('/models/explore/providers/').useSearch()`. Write changes via
  `navigate({ search: (prev) => next })`, stripping defaults and resetting `page` on any
  filter/sort/search change. Keep `searchInput`, `selectedSlug` local. Keep the `?select` rail-open
  effect and the `?q` seed reconciled with the committed `q` URL param. EXACTLY ONE effect, and it
  only writes LOCAL `searchInput` (URL→input) — never the URL.
- Commit: `feat(ui): URL filter sync + back/forward for Explore Providers`.

### Phase 3 — Toolbar 3-state reset (replace sidebar clear-all)
- Remove the in-sidebar clear-all button from `ExploreProvidersSidebar.tsx`; drop its `onClearAll` prop.
- Add the icon-only 3-state reset to the screen toolbar, copying the Models pattern verbatim:
  classes `cat-sort-btn cat-toolbar-icon-btn`, `ShellIcon name="rotate-ccw" size={13}`,
  `data-testid="cat-prov-clear-all"`, `data-test-state={resetMode}`, context-sensitive
  `aria-label`/`title`. `resetMode = hasFilters ? 'filters' : hasQuery ? 'query' : 'none'`
  (waterfall: 1st click clears facets keeping q+sort, 2nd clears q, then inert/disabled).
  `hasFilters` = `hasActiveProviderFacets(facets)` (now must include `is_lab`).
- Commit: `fix(ui): toolbar 3-state reset on Explore Providers`.

### Phase 4 — Sort changes: remove rank + cheapest, add API-format column, localStorage default
- **Sort keys:** delete `rank` and `pricing` from `SORT_LABELS`/`NATURAL_ORDER`; remaining =
  `name` (asc), `model_count` (desc), `api_format` (asc). Remove the `#` rank-sort handling but
  keep the `#{idx}` per-page row counter (non-sortable display only).
- **API-format column:** add a `<td>` after the provider-name column showing
  `provider.api_format_hint`; make its header sortable (`onSort('api_format')`) with the existing
  arrow indicator. Adjust `.cat-prov-grid` / table layout in the screen so the new column fits at
  narrow width (verify against `catalog.css` `.l-scroll`).
- **No-sort default + localStorage persistence:** consume the shared `useSortPreference` hook from
  Phase 0 (key `bodhi.explore.providers.sort`, valid keys `name`/`model_count`/`api_format`). Use
  its `effectiveSort`/`effectiveOrder` for `searchToParams`; on a sort-header click call `persist(...)`
  and write the URL. When effective sort is `undefined`, omit `sort`/`order` (API natural order).
  No bespoke localStorage code here — it lives in the shared hook.
- Commit: `feat(ui): API-format column + drop rank/cheapest sort, persist sort pref on Explore Providers`.

### Phase 5 — Labs category filter + trim api_format options + pricing facet
- **Labs toggle:** add a "Labs only" control under the sidebar **Browse** section (a single
  toggle pill, OFF by default). Active → `is_lab=true` facet → query param; reflected in
  `ProviderFacets`, `providerFacetsToQuery`, `hasActiveProviderFacets`, `searchToFacets`/`facetsToSearch`.
- **Trim api_format options:** in the sidebar, render api_format pills from
  `Object.keys(apiFormatCounts)` (the API's `facets.api_format` bucket) instead of the hardcoded
  `API_FORMAT_LABELS` key list. Keep `API_FORMAT_LABELS` only as a display-label lookup
  (fallback to the raw key). Drop `openai_responses` / `anthropic_oauth` entries from the label
  map. Result: synthetic/frontend-only formats never appear.
- **Pricing facet:** keep Free/Paid pills; **delete `ProviderRange` and the price-max slider**
  (and `pricing_max` from facets/query/schema). Remove now-unused `PRICE_MAX` constant.
- Commit: `feat(ui): Labs filter + trim api_format options + drop price slider on Explore Providers`.

### Phase 6 — Remove the result bar
- Delete the `cat-prov-resultbar` block ("Showing X of Y · sorted by …") from the screen. Keep
  `total` (feeds `ShellPagination`, which already shows the count in the pager). Remove
  `.cat-resultbar`/`.cat-count` from `catalog.css` ONLY if a repo-wide grep shows zero remaining
  users (Models already dropped its bar).
- Commit: `feat(ui): drop the result bar on Explore Providers`.

## Shared code (avoid copy-paste between the two pages)

Extract into `routes/models/explore/-shared/` and consume from both screens:
- `useSortPreference.ts` (Phase 0) — the URL>localStorage>none sort resolver + persistence. Used by
  both `ExploreApiScreen` and `ExploreProvidersScreen`.
- The 3-state reset is small but identical in shape on both pages — factor the toolbar reset
  (`resetMode` waterfall + the icon button markup) into a shared `CatalogReset` component (or a
  `useResetMode(facets, query)` hook) under `-shared/` so the precedence logic isn't duplicated.
  Keep each page's `data-testid` (`cat-model-clear-all` / `cat-prov-clear-all`) via a prop.
- Keep already-shared primitives as-is (`catalog-format.ts`, `catalog.css`, `FacetCombobox.tsx`,
  `breadcrumbs.ts`). The two `*-search.ts` mapper files stay per-page (different facet shapes) but
  should mirror each other's structure and reuse shared `PAGE_SIZE`/`DEFAULT_ORDER` constants —
  lift those into `-shared/` if it removes duplication cleanly.
Guardrail: don't over-abstract the screens themselves (columns, facets, and schemas differ enough
that a single mega-component would be worse). Share the *mechanics*, not the page layout.

## Testing (all layers — required)

- **Unit (Vitest)** — `useSortPreference.test.ts` (shared hook; see Phase 0).
  `explore-providers-search.test.ts`: round-trip `searchToFacets`/
  `facetsToSearch`/`searchToParams` (defaults applied, empties omitted, single→array coercion,
  `is_lab` mapping, sort omitted when none/url-vs-localStorage precedence).
- **Component (Vitest + MSW)** — migrate `providers/index.v2.test.tsx` to the real memory-router
  harness (`@/test-utils/router-harness`); drop the `vi.mock('@tanstack/react-router')` shim.
  Cover: deep-link `?capability=`/`?sort=` drives the request + active chip; sort/facet writes
  reset `page` and strip defaults; `router.history.back()` re-applies prior state; `?q=` seeds the
  box and requests `q`; **3-state reset in the toolbar** (filters→query→none via `data-test-state`,
  not in the sidebar); **API-format column header sorts** (`sort=api_format`); **Labs toggle**
  sends `is_lab=true`; **api_format pills render only API-returned buckets** (assert
  `openai_responses` pill absent); **no price slider**; **no result bar**; **localStorage default**
  (clean URL + saved sort → request carries that sort, URL has no `?sort=`).
- **E2E (Playwright, black-box, UI-only)** — rename to `providers.spec.mjs` + `ProvidersPage.mjs`,
  update the URL to `/ui/models/explore/providers/`. `test.step`s: filter/sort writes the URL,
  Back/Forward reverts/re-applies, toolbar clear-all, API-format column sort, Labs toggle filters
  the list, result bar removed, `?select`/`?q` cross-links from Models still work. Use
  `reducedMotion:'reduce'`; wait for mutation settle before asserting.

- **Component (Models, regression)** — `api/index.v2.test.tsx`: update default-sort expectations
  (clean first load sends NO `sort`; saved pref drives request without URL `?sort=`; explicit sort
  click writes both URL and storage). Confirm the shared reset still passes its 3-state test.

## Verification (manual, Chrome via claude-in-chrome)

Run `make app.run.live` (HMR, no rebuild) and confirm on BOTH
`/ui/models/explore/api/` and `/ui/models/explore/providers/`:
- First load with a clean URL → no `?sort=`, API natural order. Pick a sort → persists to
  localStorage; reload clean URL → that sort applied to results, URL still clean; a shared `?sort=`
  link overrides the saved pref. (Same behavior on both pages, different storage keys.)

Then on `/ui/models/explore/providers/` specifically confirm:
1. Old `/api-providers/` URL no longer resolves; nav + Models "View" cross-link land on `/providers/`.
2. Apply a Capability filter + change sort → URL gains `?capability=…&sort=…`; Back reverts list+URL; Forward re-applies.
3. Toolbar reset: 1st click clears facets (keeps search/sort), 2nd clears search, then inert; not in sidebar; no sidebar layout shift on toggle.
4. Sort bar has **Name / Models / Format** only (no Rank, no Cheapest). API-format column present, header sortable, arrow indicator correct.
5. Sidebar: **Labs only** toggle off by default; turning it on shows only labs (`is_lab=true`). api_format pills show only `openai/anthropic/gemini/other` (no OpenAI Responses / Anthropic OAuth). Pricing has Free/Paid only — no slider.
6. No "Showing X of TOTAL · sorted by …" bar; count shows in the bottom pager.

## Gate before each commit
`cd crates/bodhi && npm run test` (providers test green), `npm run lint`, `tsc --noEmit`. Run the
E2E spec if Docker/Postgres is up; otherwise verify the flow manually in Chrome and note E2E was
deferred to the user. Commit directly to `main` (trunk-based), one focused commit per phase.
