# Plan — Enrich Explore · API Models / Providers search + migrate Load More → pagination

## Context

BodhiApp ships two Explore catalog pages — **API Models** (`/models/explore/api`) and **API Providers**
(`/models/explore/api-providers`) — backed by the `api-getbodhi-app` catalog API
(`https://dev-api.getbodhi.app/api/v1/catalog/*`). The backend has now **fully implemented and deployed**
an enriched search surface; the frontend consumes only a fraction of it. This plan brings the frontend up
to the deployed contract, with filter controls matched to data cardinality, surfaces the cross-provider
cost comparison in the rail, and **migrates Load More → numbered pagination**.

Full analysis: `docs/claude-plans/202606/screen-v2/explore-api-models/feature-enrichment-analysis.md`.
Authoritative API contract: `api-getbodhi-app/docs/functional/{endpoints.md,recipes.md,openapi.json}`.

### Deployed backend contract (verified live on `dev-api` 2026-06-24)

`GET /api/v1/catalog/models` — query: `q` (FTS), `capability` (AND), `modality` (OR), `family` (OR,
ci-exact), `pricing_min` / `pricing_max` / `pricing_out_max` (USD/1M; **`pricing_max=0` = free models**),
`context_min`, `status` (OR; `stable` = absent), `provider` (OR), `open_weights` (open|closed),
`sort` (`relevance|updated|context|price|price_out|name|family|providers`) + `order` (`asc|desc`),
`page`/`page_size` (default 20). Response `facets` = `{ capability, modality, status, provider, family,
open_weights }`, each a `{value:count}` map over the **fully-filtered** set; `provider` and `family` are
high-cardinality (one key per slug/family) — backend docs recommend driving **autocomplete/typeahead** off
these maps. `pricing_band` has been **removed** entirely.

`GET /api/v1/catalog/providers` — `q` (provider name/slug + served-model names), `capability` (AND),
`api_format` (OR), `pricing` (`free|paid`), `pricing_max`, `sort`
(`rank|name|model_count|api_format|pricing`) + `order`, `page`/`page_size`. Facets: `capability`,
`api_format`.

`GET /api/v1/catalog/providers/{slug}/models` — **unpaginated**; `sort` (`name|context|price`) + `order`.

Search: prefix-first + **typo/substring tolerant** (trigram FTS, now live: `q=clade`→Claude works).
Search-as-you-type = debounce ~150–250 ms + `sort=relevance`. No server price/context histogram — drive
range sliders from known maxima. `served_by[]` on model detail is cheapest-first (cross-provider cost).

### Decisions (locked with the user)
- **Scope:** everything incl. rails — delivered **phase-wise, iterative, TDD, commit between every phase**
  (build a slice → verify in Chrome → grow one Playwright spec with many `test.step` → run all gate checks
  → commit).
- **Types:** the backend will **publish `@bodhiapp/reference-api-types@0.0.8`** (with the `family` facet +
  `pricing_band` removed) to npm **first**. Frontend Phase 1 bumps to `^0.0.8`. *(npm `0.0.7`, published
  06-23, predates the `f7de38f` family-facet / drop-`pricing_band` commit — it still carries `pricing_band`
  and lacks `facets.family`. The in-tree dist has the new contract; it just isn't on npm yet.)*
- **Pricing UX:** migrate from the old `pricing_band` (high/med/low/free) to **numeric min/max range
  filters** (`pricing_min` + `pricing_max` input range, `pricing_out_max` output ceiling) **plus a "Free"
  chip** that sets `pricing_max=0` (pins the range max to 0; toggling off restores the range).
- **Provider + family filter:** **autocomplete + removable chip tags**, options/counts driven from the
  **`facets.provider` / `facets.family`** maps (live counts under current filters); fetch the provider list
  once for slug→display-name/logo. No per-keystroke `?q=` round-trips.
- **Typo search is live** — ship search-as-you-type now (debounce + `sort=relevance`); no backend gate.

### Constraints
1. ⚠️ **Phase 1 is a hard prerequisite** and is gated on the backend publishing `0.0.8` to npm. Installed
   `0.0.6` lacks the new params; npm `0.0.7` lacks `family` facet + still has `pricing_band`.
2. ⚠️ **`/api/v1/namespaces` is HuggingFace-only** — not for catalog providers. Provider typeahead uses
   `facets.provider` + the provider list, per the decision above.

---

## Reuse surface (all confirmed present — do NOT rebuild)

| Need | Reuse | File |
|---|---|---|
| Numbered pager | `ShellPagination` (canonical usage: `tokens/index.tsx`) | `crates/bodhi/src/components/shell/ShellPagination.tsx` |
| Dual-handle range | `Slider` accepts `value={[min,max]}` out of the box | `crates/bodhi/src/components/ui/slider.tsx` |
| Provider/family multi-select autocomplete | pattern from `AliasCombobox` + `Command`/cmdk | `crates/bodhi/src/routes/models/router/-components/AliasCombobox.tsx`, `components/ui/command.tsx` |
| Facet chips/group | existing hand-rolled `FacetPill`/`FacetGroup` (has a count-badge slot) | `.../explore/api/-components/ExploreApiSidebar.tsx` |
| Rail slots | `useShellChrome({ rail, railHeader })` | `components/shell/ShellSlotsContext.tsx` |
| Query string build | `buildCatalogModelsQuery` (skips empties, repeats array keys — generic walk) | `crates/bodhi/src/hooks/reference/useCatalog.ts` (37–54) |
| Configure prefill | `ApiModelForm` `prefill={{api_format,base_url,model}}` (already wired from the rail CTA) | `components/api-models/ApiModelForm.tsx`; route `models/api/new/index.tsx` |
| Format helpers | `fmtPrice`, `fmtContext`, `isFree`, `statusLabel`, `CAP_LABELS` | `.../explore/-shared/catalog-format.ts` |

**Data-layer facts:** catalog hooks in `hooks/reference/useCatalog.ts` (`useCatalogModels`,
`useCatalogProviders`, `useCatalogProviderModels(slug, params?)`, `useCatalogModelDetail`,
`useCatalogProviderDetail`) use the anonymous client + `keepPreviousData`. Query-key factory
(`hooks/reference/constants.ts`) keys on a serialized `paramsKey` — **no change needed** for new params.
Screens hold all state (`page`, `accumulated[]`, `search`, `sort`, `facets`). The model rail
(`ExploreApiRail.tsx` 100–127) **already renders `served_by[]`**; the provider rail calls
`useCatalogProviderModels(slug)` with **no params** today.

---

## Phases (one commit per phase; TDD; gate checks before each commit)

> Each phase = a thin vertical slice: extend the Vitest `*.v2.test.tsx` (red → green, asserting the request
> query string via the MSW `onRequest` hook), verify in Chrome via `make app.run.live` against `dev-api`,
> grow the one Playwright spec per page with a new `test.step` (`reducedMotion:'reduce'` for rail/VT
> races), run `make format` + `cd crates/bodhi && npm run test` + the touched Playwright spec, then commit
> to `main` (trunk-based).

### Phase 1 — Types bump to `^0.0.8` + query-builder params (no UI change; unblocks all) — *gated on backend 0.0.8 on npm*
- Bump `@bodhiapp/reference-api-types` → `^0.0.8` in `crates/bodhi/package.json`; `npm install`; confirm
  the installed `.d.ts` has `pricing_min`/`pricing_out_max`/`family` query fields, `facets.family`, and
  **no** `pricing_band`.
- Extend the typed param state + facet→query mappers to pass the new params: `ModelFacetsState` +
  `modelFacetsToQuery` (`ExploreApiSidebar.tsx`) gain `pricing_min`, `pricing_out_max`, `family`; the
  providers params builder (`ExploreProvidersScreen.tsx`) gains `pricing_max`, `pricing`; thread `order`
  into both screens' params. `buildCatalogModelsQuery` already serializes them.
- **Remove all references to the old `pricing_band` / high-med-low chip notion** anywhere in the catalog
  UI/state (the sidebar currently only sends raw `pricing_max`, so this is mostly type-state cleanup).
- **Tests:** unit-assert the request query string includes the new params when state is set.

### Phase 2 — Load More → `ShellPagination` (both catalog pages) — *the explicit ask*
- In `ExploreApiScreen.tsx` and `ExploreProvidersScreen.tsx`: remove `accumulated[]` + dedup-merge +
  `loadMore` + the `cat-loadmore` button; render only `data.items` for the current page; render
  `<ShellPagination total={data.total} page={page} onPage={setPage} pageSize={PAGE_SIZE} unit="models|providers" />`.
- Keep `resetPaging` (→ page 1) on search/sort/facet change; keep `keepPreviousData`.
- Models page = full numbered pager; Providers page = `minimal`. **Local Models stays on Load More**
  (cursor-based, out of scope). Drop now-dead `.cat-loadmore` CSS if unused.
- **Tests:** replace the "Load-more without duplicates" assertions with pager assertions (click page 2 →
  `page=2`; changing a filter resets to page 1). MSW handlers already honor `page`/`page_size`. Playwright:
  a `test.step` paging to page 2 and back.

### Phase 3 — Sort: asc/desc `order` + relevance-on-search + new sorts
- Models page: column headers toggle `order` (asc/desc) instead of descending-only; add `price_out` and
  `family` sort options; when `q` is non-empty auto-apply `sort=relevance` (fall back to `updated` when
  cleared). Thread `order` into params.
- Providers page: add `pricing`, `api_format`, `model_count`, `name` sorts + `order` toggle.
- **Tests:** header click toggles `order=asc|desc`; `q` set → `sort=relevance`; new sort keys sent.

### Phase 4 — Pricing range (min/max + Free chip) + output price + context
- Replace the single-handle `pricing_max` `RangeControl` with a **dual-handle** input-price range
  (`Slider value={[min,max]}`, `onValueCommit`) sending `pricing_min` + `pricing_max` (range ~0–100, step
  0.5). Add a **"Free" chip** beside it that sets `pricing_max=0` (and pins the displayed max to 0); toggle
  off restores the prior range. Reflect "Free" label when `pricing_max===0`.
- Add an **output-price** range (`pricing_out_max`, ceiling only; ~0–150).
- Keep **context** as min-only (`context_min`) with stepped marks (8k/32k/128k/200k/1M).
- **Tests:** dual price handles send both `pricing_min`+`pricing_max`; Free chip sends `pricing_max=0` and
  visually pins; output slider sends `pricing_out_max`.

### Phase 5 — Live facet counts (both pages)
- Render `data.facets` counts on every chip (models: capability/modality/status/open_weights; providers:
  capability/api_format) via the existing `FacetPill` count-badge slot; grey/disable zero-count values.
- **Tests:** chips render counts; a zero-count chip is disabled.

### Phase 6 — Provider + family autocomplete (facets-driven)
- **Provider filter** (models page): replace the top-12 chip list with an autocomplete-with-chip-tags
  control (pattern from `AliasCombobox` + `Command`); options + counts from `facets.provider`; fetch the
  provider list once (`useCatalogProviders`) for slug→display-name/logo; selected providers as removable
  chips; sends repeated `provider=`.
- **Family filter** (models page): same control, options/counts from `facets.family`; sends repeated
  `family=`.
- **Tests:** typing filters the facets-derived options; selecting adds a chip + sends `provider=`/`family=`;
  removing a chip drops it; counts shown.

### Phase 7 — Providers page filters polish
- Add provider **price filter** (`pricing_max` slider) + **free/paid** toggle (`pricing`) to
  `ExploreProvidersSidebar.tsx`; confirm capability filter (server-AND) with live counts; add a hint that
  `q` matches providers **and** the models they serve.
- **Tests:** price slider sends `pricing_max`; free/paid sends `pricing`; capability sends repeated `capability=`.

### Phase 8 — Rails: cross-provider cost table + provider-rail sort + Configure bridge
- **Models rail** (`ExploreApiRail.tsx`): enrich the existing `served_by[]` list with each provider's own
  `$in / $out` (+ cache prices when present), cheapest-first, keep the provider deep-link. (Cross-provider
  cost comparison.)
- **Provider rail** (`ExploreProvidersRail.tsx`): wire a sort toggle (name/context/price) into the
  served-models list by passing `params` to `useCatalogProviderModels(slug, { sort, order })` (the endpoint
  is unpaginated → full list available).
- **Configure bridge**: verify the rail CTA → `ApiModelForm` prefill; surface
  `bridge.base_url_requires_substitution` (e.g. `{AWS_REGION}`) as a hint rather than a dumb copy.
- **Tests:** rail shows per-provider prices; provider-rail sort sends `sort`/`order`; Configure CTA carries
  `api_format`/`base_url`/`model` (existing) + substitution hint when applicable.

### Phase 9 — Typo-tolerant search verification
- Typo tolerance is live — add one Playwright `test.step` (or MSW fixture mirroring it) asserting a typo
  query (`clade`) returns results with `sort=relevance`, and a debounce on search-as-you-type. No new
  product code beyond Phase 3.

### Phase 10 (optional) — Cross-cutting polish
- URL-synced filter/sort/page state (TanStack search params) for shareable searches (only `?select` read
  today); optionally consolidate the hand-rolled facet sidebars into one `ShellFilterGroup`-based kit.
  Defer unless requested.

---

## Critical files

- Screens/sidebars/rails:
  `crates/bodhi/src/routes/models/explore/api/-components/{ExploreApiScreen,ExploreApiSidebar,ExploreApiRail}.tsx`
  and `.../api-providers/-components/{ExploreProvidersScreen,ExploreProvidersSidebar,ExploreProvidersRail}.tsx`
- Data layer: `crates/bodhi/src/hooks/reference/useCatalog.ts`, `constants.ts`
- Shared: `crates/bodhi/src/routes/models/explore/-shared/{catalog-format.ts,catalog.css,breadcrumbs.ts}`
- Reuse: `components/shell/ShellPagination.tsx`, `components/ui/slider.tsx`, `components/ui/command.tsx`,
  `routes/models/router/-components/AliasCombobox.tsx`, `components/api-models/ApiModelForm.tsx`
- Tests: `.../explore/api/index.v2.test.tsx`, `.../api-providers/index.v2.test.tsx`,
  `test-utils/msw-v2/handlers/reference-catalog.ts`, `test-fixtures/catalog-{models,providers}.ts`
  (fixtures need a `facets.family` bucket added; MSW handlers need to honor the new query params for
  request-assertion tests)
- Playwright: one spec per page under `crates/lib_bodhiserver/tests-js/`
- Dep: `crates/bodhi/package.json` (`@bodhiapp/reference-api-types` → `^0.0.8`)

## Verification (per phase + final)

- **Unit:** `cd crates/bodhi && npm run test` for the touched page; assert the request query string carries
  the new params via the MSW `onRequest` hook (established pattern).
- **Chrome:** `make app.run.live` (Vite HMR; no UI rebuild) → exercise against `dev-api`; confirm pager,
  sliders + Free chip, provider/family autocomplete, facet counts, sort direction, typo search, and rail
  prices.
- **Playwright:** grow one spec per page (`reducedMotion:'reduce'`); `make test.e2e` from
  `crates/lib_bodhiserver/tests-js`.
- **Gate before each commit:** `make format`, unit tests, touched Playwright spec.

## Backend status (no frontend action — informational)

All three asks are **done, deployed, and live** on `dev-api` (commit `f7de38f`): typo search works,
`facets.family` is populated, `pricing_band` removed. The **only** open backend item is **publishing
`@bodhiapp/reference-api-types@0.0.8` to npm** so Phase 1 can pin it. The prior kickoff prompt at
`screen-v2/explore-api-models/backend/04-search-enrichment-asks-kickoff.md` is now **obsolete** (its asks
are implemented) and can be deleted — superseded by "publish 0.0.8".
