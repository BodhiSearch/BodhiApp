# Adapt Explore·API UI to reference-api-types 0.0.11 (facets → value arrays)

## Context

The `api-getbodhi-app` catalog backend was optimized (commit `f02a887`: materialized
`logical_models` + KV-cached lists) and shipped as **`@bodhiapp/reference-api-types@0.0.11`**
(published, `latest`). The optimization changed the wire shape of `/api/v1/catalog/models` and
`/api/v1/catalog/providers` in two ways, with **no backwards-compat shim** — the old shape is gone.
The frontend currently depends on `^0.0.10` and reads the old shape, so it will type-break and
mis-render once bumped.

**Breaking change 1 — facets are now value arrays, not count maps.** Previously each facet dimension
was a `FacetBucket = Record<string, number>` (value → post-filter count). Now each dimension is a
`FacetValues = string[]` (the global set of available filter values, no counts). `FacetBucket` is
removed. Facet arrays are **global** (precomputed per catalog sync), so they do **not** change as
filters are applied and an empty result set (`total: 0`) still returns the full global arrays.
- `ModelFacets`: `{ capability, modality, status, provider, family, open_weights }` — each `string[]`.
- `ProviderListResponse.facets`: `{ capability, api_format, is_lab }` — each `string[]` (`is_lab` is `["lab"]`).

**Breaking change 2 — default `page_size` 20 → 50.** *No frontend impact:* both Explore screens
already send an explicit `page_size: PAGE_SIZE` (30) on every request (`explore-api-search.ts:57`,
`explore-providers-search.ts:40`), and no code relies on the server default or hardcodes 20. The MSW
`paginate()` helper also defaults to 30 and the query always sets it. **No code change for this** —
documented here only so it isn't re-investigated.

**Intended outcome:** bump to 0.0.11, drop all count-based UI (badges + zero-count disabling), and
re-base chip availability on array membership. The user's chosen behavior: **a fixed-enum chip is
enabled only if its value is present in the facet array** (selected values stay clearable even if
absent). High-cardinality comboboxes (provider, family) render their options straight from the array.

## Changes

### 1. Bump the dependency
`crates/bodhi/package.json:20` — `"@bodhiapp/reference-api-types": "^0.0.11"`, then
`cd crates/bodhi && npm install` to refresh the lockfile. Confirm the installed `dist/index.d.ts`
has `FacetValues` and no `FacetBucket`.

### 2. `FacetCombobox.tsx` — drop counts from the helper + UI
`src/routes/models/explore/-shared/FacetCombobox.tsx`
- Rewrite `facetOptions()` to take `FacetValues | undefined` (a `string[]`) and map each value to
  `{ value }` (preserve array order — backend already orders them; no count to sort by):
  ```ts
  export function facetOptions(values: string[] | undefined): FacetOption[] {
    return (values ?? []).map((value) => ({ value }));
  }
  ```
- Remove the `count?` field from `FacetOption` and the `{o.count != null && <span className="cat-facet-count">…}` render (line 98). Update the doc comments that say "highest count first" / "value:count map".

### 3. `ExploreApiSidebar.tsx` — array membership instead of counts
`src/routes/models/explore/api/-components/ExploreApiSidebar.tsx`
- Import `ModelFacets` stays (now arrays). Replace the three `?? {}` count maps with availability
  **sets** derived from the facet arrays:
  ```ts
  const capAvail = new Set(facetCounts?.capability ?? []);
  const modAvail = new Set(facetCounts?.modality ?? []);
  const statusAvail = new Set(facetCounts?.status ?? []);
  const owAvail = new Set(facetCounts?.open_weights ?? []);
  ```
- `providerOptions`/`familyOptions` — pass `facetCounts?.provider` / `facetCounts?.family` (now
  `string[]`) into the rewritten `facetOptions`.
- Change `FacetPill` from a count-driven component to an **available**-driven one: replace `count`
  prop with `available: boolean`. Render no badge; `disabled = !available && !active` (selected stays
  clearable). At each call site pass `available={capAvail.has(c)}` etc. Synthetic chips (Free) pass
  `available` (always enabled) since they aren't real facet values.
- `open_weights` pills: `available={owAvail.has(w)}` (values are `'open'`/`'closed'`).
- Remove the `cat-facet-count` `<span>` and the `n > 0` logic from `FacetPill`.

### 4. `ExploreProvidersSidebar.tsx` — same treatment
`src/routes/models/explore/providers/-components/ExploreProvidersSidebar.tsx`
- Drop the `FacetBucket` import. Change the two props from `capabilityCounts/apiFormatCounts:
  FacetBucket` to `capabilityValues/apiFormatValues: FacetValues` (`string[]`).
- `apiFormatKeys`: build from the array directly, still unioned with any selected value and still
  filtering out `'openai_responses'`:
  ```ts
  const apiFormatKeys = Array.from(
    new Set<ApiFormatHint>([...(apiFormatValues as ApiFormatHint[]), ...(facets.api_format ?? [])])
  ).filter((f) => f !== 'openai_responses');
  ```
- `FacetPill` → `available`-driven (mirror change in §3): `available={new Set(capabilityValues).has(c)}`,
  `available={new Set(apiFormatValues).has(f)}`. No badges. Labs-only / Free / Paid stay synthetic
  (always enabled).
- Update `ExploreProvidersScreen.tsx:262-263` to pass `capabilityValues={data?.facets.capability ?? []}`
  / `apiFormatValues={data?.facets.api_format ?? []}` and the matching `useMemo` deps (267).

### 5. Test fixtures — facets become arrays
- `src/test-fixtures/catalog-models.ts:40-47` — replace count maps with arrays, e.g.
  `capability: ['reasoning','tool_call','structured_output','attachment','vision']`,
  `modality: ['text','audio','image','video','pdf']`, `status: ['stable','alpha','beta','deprecated']`,
  `provider: ['nano-gpt','kilo','openrouter','vercel','anthropic','openai']`,
  `family: ['claude','gpt','gemini','llama','deepseek-v3']`, `open_weights: ['open','closed']`.
  Update the doc comment ("page-recomputed counts" → "global available values").
- `src/test-fixtures/catalog-providers.ts:42-46` — `capability: [...]`, `api_format: ['openai','anthropic','gemini','other']`,
  `is_lab: ['lab']`. Update the doc comment.
- `src/test-utils/msw-v2/handlers/reference-catalog.ts` — passes fixture facets through; **no change**
  beyond confirming it still type-checks against `ModelsListResponse`/`ProviderListResponse`.

### 6. Component tests — rewrite count assertions
- `src/routes/models/explore/api/index.v2.test.tsx` — the test **"renders live facet counts and
  disables zero-count buckets"** (≈line 600) is the one structural rewrite. Re-author its intent to
  the new contract: build a fixture whose facet arrays **omit** some values (e.g. `capability` without
  `structured_output`, `modality` without `audio`, `status` without `beta`) and assert those chips are
  **disabled** while present ones are **enabled** — and that **no `cat-facet-count` badge renders**.
  Rename to "disables facet values absent from the global arrays". Other tests in this file that only
  toggle facets and assert query params are unaffected.
- `src/routes/models/explore/providers/index.v2.test.tsx` — the test "capability + api_format facets
  send repeated-key params and counts render" (≈line 460): drop the `toHaveTextContent('80')` count
  assertion; keep the repeated-key param assertions and the `openai_responses`/`anthropic_oauth`
  exclusion + `anthropic` enabled checks (now driven by array membership).
- `explore-api-search.test.ts` / `explore-providers-search.test.ts` — only reference `PAGE_SIZE`
  (unchanged 30); **no change** unless a fixture import drags in the old facet shape.

### 7. CSS / dead code
- `.cat-facet-count` is now unrendered. Grep `catalog.css` / `models.css` for `cat-facet-count` and
  remove the rule if it has no other consumer (verify with a repo-wide grep before deleting).

## Verification

1. **Types/build:** `cd crates/bodhi && npm install && npx tsc --noEmit` — must be clean (catches any
   remaining `Record<string,number>` facet access and the removed `FacetBucket` import).
2. **Lint/format:** `cd crates/bodhi && npm run lint && npm run format`.
3. **Unit tests:** `cd crates/bodhi && npm test -- explore` (covers `api/index.v2`, `providers/index.v2`,
   both `*-search.test.ts`). All green, including the rewritten availability test.
4. **Live smoke (Chrome):** `make app.run.live`, open `/ui/models/explore/api/` and
   `/ui/models/explore/providers/`:
   - Sidebar chips render with **no count badges**.
   - Chips whose value is in the global facet set are clickable; absent ones are disabled.
   - Provider/Family comboboxes list values from the arrays; selecting one sends the repeated query
     param and narrows the list.
   - Pager still shows 30/page; an empty search keeps the full sidebar (global facets).
5. **E2E:** only if a Playwright spec under `lib_bodhiserver/tests-js` exercises these screens'
   facets — run `make test.e2e` for the relevant spec; otherwise not touched.

## Notes / out of scope
- `page_size` default 20→50: **no change** (explicit 30 everywhere). Documented above so it isn't
  re-investigated.
- Local-models / HF-discovery facets (`ModelSidebarFacets.tsx`, `LocalDiscoverySidebar.tsx`,
  `useModels.ts`) are a **different** API and unaffected — do not touch.
- This is a frontend-only change; no Rust/OpenAPI regeneration. Commit straight to `main` per
  trunk-based workflow after all gate checks pass.
