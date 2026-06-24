# Explore · API Models / Providers — Search Feature Enrichment Analysis

**Date:** 2026-06-23
**Status:** Analysis + proposal. **No code changes.** Pick features → then switch to a plan.
**Scope:** Enrich the two existing Explore catalog pages to consume the backend's newly-enhanced
`/api/v1/catalog/*` search surface, and migrate from **Load More** to **numbered pagination**.

**Inputs cross-checked:**
- Backend source: `api-getbodhi-app` (`worker/schemas/catalog.ts`, `RealCatalogReader.ts`, `bridge.ts`) + `CHANGELOG.md` "Unreleased".
- **Live API probed directly** at `https://dev-api.getbodhi.app/api/v1/catalog/*` (real data, ground truth).
- Frontend source: `crates/bodhi/src/routes/models/explore/{api,api-providers,local}/`, `hooks/reference/useCatalog.ts`, `lib/referenceApiClient.ts`, shared shell primitives.
- Published types: `@bodhiapp/reference-api-types@0.0.7` (BodhiApp currently pins `^0.0.6`).
- Design mocks: `design-prompt.md`, `sample-responses.json`, the `design/models/*.jsx` mock source.

---

## 1. Executive summary

The two catalog pages **already exist and work**, but they consume only a fraction of what the
backend now offers. The backend was "enhanced quite a lot" — full sort/filter parity with the UI's
intent, FTS5 search, relevance ranking, ascending/descending order, and live facet counts — and the
frontend has not caught up. This is an **enrichment of existing pages**, not greenfield.

Three workstreams:

1. **Wire the now-functional backend params** the UI already wants but doesn't send (ascending sort,
   relevance, output-price filters, provider price filters, family filter, the new sorts).
2. **Upgrade the filter controls** to match data cardinality: dual-handle **min/max range sliders**
   for price (in + out) and context; **autocomplete** for the 144-provider and the large family sets
   (chips don't scale); live **facet counts** on every chip.
3. **Migrate Load More → numbered pagination** on both catalog pages (the `ShellPagination`
   component already exists, unused).

**Three hard constraints I verified on the live API (these bound what we can promise):**

- ⚠️ **Typo tolerance is NOT live on dev yet.** `q=clade` and `q=onnet` return **0** results today.
  Prefix FTS works (`q=clau` → 229). The trigram migration (`0004_catalog_fts_trigram.sql`) and its
  reader code exist in source but are not applied on `dev-api`. Treat typo-tolerant search-as-you-type
  as **"verify before relying on it"** — the code is merged, the index isn't live.
- ⚠️ **`pricing_band` `low|mid|high` are dead enum values.** Only `free` is implemented in the reader;
  `low/mid/high` are accepted by zod but are silent no-ops. **Do not build a "price band" chip set on
  them** — use the explicit numeric `pricing_min` / `pricing_max` / `pricing_out_max` ranges.
- ⚠️ **`/api/v1/namespaces` is HuggingFace-only**, not a catalog-provider autocomplete. For provider
  typeahead on the API catalog, use `GET /api/v1/catalog/providers?q=` (its `q` matches provider
  name/slug **and** served-model names). There is no dedicated provider/family autocomplete endpoint —
  but we don't need one (see §5).

---

## 2. What the backend now offers (the enhanced surface)

All confirmed in `worker/schemas/catalog.ts` + verified live unless flagged.

### 2.1 `GET /api/v1/catalog/models` — params

| Param | Type | Combine | Default | Live? | Notes |
|---|---|---|---|---|---|
| `q` | string | FTS5 | — | ✅ prefix | Matches name/id/provider/family/modalities + capability keyword bag incl. synthetic `vision`. Trigram typo fallback ⚠️ **not live on dev**. |
| `capability` | enum, repeatable | **AND** | — | ✅ | `reasoning\|tool_call\|structured_output\|attachment\|vision` |
| `modality` | enum, repeatable | **OR** | — | ✅ | `text\|audio\|image\|video\|pdf` (input ∪ output) |
| `status` | enum, repeatable | OR | — | ✅ | `stable\|alpha\|beta\|deprecated`; `stable` = status absent |
| `provider` | slug, repeatable | OR | — | ✅ | keeps model if any group member served by that provider |
| `family` | string, repeatable | OR (ci-exact) | — | ✅ **NEW** | exact family match, case-insensitive |
| `open_weights` | `open\|closed` | single | — | ✅ | |
| `pricing_max` | number | ≤ input $/M | — | ✅ | nulls excluded |
| `pricing_min` | number | ≥ input $/M | — | ✅ **NEW** | nulls excluded |
| `pricing_out_max` | number | ≤ output $/M | — | ✅ **NEW** | nulls excluded |
| `context_min` | int | ≥ tokens | — | ✅ | nulls excluded |
| `pricing_band` | `free\|low\|mid\|high` | — | — | ⚠️ only `free` works | **low/mid/high dead** |
| `sort` | enum | — | `updated` | ✅ | see §2.4 |
| `order` | `asc\|desc` | — | per-sort | ✅ **NEW** | overrides natural direction |
| `page` / `page_size` | int | — | `1` / **`20`** | ✅ | page_size max 100; default was 8 |

### 2.2 `GET /api/v1/catalog/providers` — params

| Param | Type | Combine | Live? | Notes |
|---|---|---|---|---|
| `q` | string | substring | ✅ | two-tier: provider name/slug, then **served model names** (lower-ranked) |
| `capability` | enum, repeatable | **AND** | ✅ **now functional** | was accepted-but-ignored |
| `api_format` | enum, repeatable | OR | ✅ | `openai\|openai_responses\|anthropic\|anthropic_oauth\|gemini\|other` |
| `pricing_max` | number | ≤ provider min input $/M | ✅ **now functional** | |
| `pricing` | `free\|paid` | — | ✅ **NEW** | `free` = min_in == 0 |
| `sort` | `rank\|name\|model_count\|api_format\|pricing` | — | ✅ **NEW sorts** | `pricing` = cheapest first |
| `order` | `asc\|desc` | — | ✅ **NEW** | |
| `page` / `page_size` | int | — | ✅ | default `1` / **`20`** (was 5) |

### 2.3 `GET /api/v1/catalog/providers/{slug}/models` — **now UNPAGINATED**

Returns `{ items, total }` only (no `page`/`page_size`). Sortable: `sort=name\|context\|price`
(default `context`), `order=asc\|desc`. The provider rail's served-model list can now offer its own
sort control. (Currently the UI calls this with **no params** at all.)

### 2.4 Sorts (with natural direction)

- **models:** `relevance` (bm25, needs `q`) · `updated` (desc, default) · `context` (desc) ·
  `price` (asc) · `price_out` (asc, **NEW**) · `name` (asc) · `family` (asc, **NEW**) · `providers` (desc).
- **providers:** `rank` (default) · `model_count` · `name` · `api_format` (**NEW**) · `pricing` (**NEW**).
- `order=asc\|desc` overrides any of them. **Today the frontend sends descending-only.**

### 2.5 Facets (live counts in every list response)

- **models** `facets`: `capability`, `modality`, `status`, `provider`, `open_weights` — value→count over
  the post-filter, pre-pagination grouped set. (Live example: capability `tool_call: 1762`,
  `reasoning: 1292`; status `stable: 2477`, `deprecated: 66`; **144 providers** in the provider bucket.)
- **providers** `facets`: `capability`, `api_format`.
- The frontend **fetches these but renders static chip labels** — it doesn't show the counts or grey
  out zero-count values.

### 2.6 Per-model cross-provider comparison (already in the contract)

`GET /catalog/models/{slug}/{model_id}` returns `served_by[]` — **every provider serving that logical
model, each with its own `{input_per_m, output_per_m, cache_read, cache_write}` and a `base_url`**,
cheapest-input first, plus an embedded `bridge` (`api_format`, `base_url`, `base_url_source`,
`base_url_requires_substitution`). This is the "compare cost across providers" surface the user wants —
the data is already there; the rail under-renders it.

---

## 3. What the frontend does today (and the gap)

### 3.1 Architecture (reuse surface — all confirmed present)

- Three sibling master-detail screens under `routes/models/explore/{api,api-providers,local}/`, each on
  the AppShell chrome (`useShellChrome`: sidebar facets + right detail rail), `useListKeyNav` arrow nav,
  breadcrumb. Catalog reads via `hooks/reference/useCatalog.ts`; thin `fetch` client
  `lib/referenceApiClient.ts` (anonymous, external origin via `AppInfo.reference_api_url`).
- Query strings built per-hook (`buildCatalogModelsQuery`): skips empty values, serializes arrays as
  repeated keys — **already correct** for the repeatable params; adding new params is a small change.
- **Ready-but-unused primitives:**
  - `components/shell/ShellPagination.tsx` — full numbered pager (ellipsis windowing, prev/next,
    rows-per-page, `minimal` mode). **Exists, zero Explore usages.** ← the migration target.
  - `components/ui/slider.tsx` (Radix) — used only on API Models, single-handle, for `pricing_max` +
    `context_min`. Needs a **dual-handle** wrapper for true min/max.
  - `components/Combobox.tsx` / `AutocompleteInput.tsx` / `components/ui/command.tsx` (cmdk) — full
    autocomplete stack, **unused by Explore** (search is a plain `ShellSearch` text input).
  - `ShellFilterGroup` — generic faceted group with counts; **Explore hand-rolls its own**
    `FacetGroup/FacetPill` markup instead (3 near-duplicates).

### 3.2 Pagination today = **Load More** (the thing to replace)

Both catalog pages: `page` state + `accumulated[]` state; `loadMore` appends `data.items` into
`accumulated` and increments `page`; `showLoadMore = rows.length < total`. (Local is cursor-based and
is **out of scope** — keep its Load More.)

### 3.3 The gap (current UI vs. enhanced backend)

| Area | Today | Enhanced backend offers | Gap |
|---|---|---|---|
| **Sort direction** | descending-only | `order=asc\|desc` | No asc toggle on any column |
| **Relevance** | not used | `sort=relevance` (bm25) | Search results not relevance-ranked |
| **New model sorts** | not used | `price_out`, `family` | No output-price / family sort |
| **Output price filter** | not used | `pricing_out_max` | Can't filter by output cost |
| **Min price filter** | not used | `pricing_min` | Price is ceiling-only, not a range |
| **Family filter** | not used | `family` (repeatable) | No family facet |
| **Price range control** | single-handle ≤ slider | min+max | Not a true range |
| **Provider facet** | top-12 chips, no overflow | 144 providers + counts | Can't reach providers past top-12 |
| **Facet counts** | static labels | live counts | No counts, no zero-greying |
| **Provider page price** | no control | `pricing_max`, `pricing` free/paid | Providers sidebar has no price filter |
| **Provider new sorts** | not used | `pricing`, `api_format` | No cheapest-provider sort |
| **Provider rail models** | no params | `sort=name\|context\|price` | Served list never sorted |
| **Cross-provider cost** | rail under-renders | `served_by[]` w/ per-provider $ | Comparison surface thin |
| **Pagination** | Load More | `page`/`page_size`/`total` | Migrate to numbered pager |
| **Types** | `^0.0.6` | `0.0.7` (has all new query types) | Bump dependency first |

---

## 4. Proposed feature set (pick from these)

Grouped by page, tagged **[wire]** (param already supported, just send it), **[control]** (new/upgraded
UI control), **[pagination]**, **[infra]**. Each is independently shippable as a thin vertical slice.

### Group A — Foundation (prerequisite for everything)

- **A1 [infra]** Bump `@bodhiapp/reference-api-types` `^0.0.6` → `0.0.7`; regenerate; confirm new query
  types (`pricing_min`, `pricing_out_max`, `family`, `order`, new `sort` unions, provider `pricing`).
- **A2 [infra]** Add the new params to `buildCatalogModelsQuery` / providers query builder + `useCatalog`
  hook signatures. (Serialization already handles repeated keys — low risk.)
- **A3 [pagination]** Replace Load More with `ShellPagination` on **both** catalog pages: drop
  `accumulated[]`, render `data.items` for the current page, wire `page`/`total`/`page_size`, reset to
  page 1 on any filter/sort/search change. *(This is the explicit "migrate search to pagination" ask.)*

### Group B — API Models page: filters & sort

- **B1 [control]** **Dual-handle price range** (input $/M) → `pricing_min` + `pricing_max`. Replaces the
  single ≤ slider. Suggested range 0–100, step 0.5, "Free" at 0.
- **B2 [control]** **Output price range** (output $/M) → `pricing_out_max` (ceiling; backend has no
  output floor). Range 0–150.
- **B3 [control]** **Context range** stays min-only (`context_min`) — backend has no `context_max`.
  Use stepped marks (8k/32k/128k/200k/1M) since it's log-distributed.
- **B4 [control]** **Family filter** → `family` (repeatable, OR). Cardinality is moderate; render as a
  **search-within-facet** chip group (small inline filter box) or a compact autocomplete.
- **B5 [control]** **Provider filter as autocomplete** → driven by `providers?q=` typeahead (144 is too
  many for chips). Multi-select chip tags below the input; sends repeated `provider=`.
- **B6 [wire]** **Ascending/descending column sort** → `order` toggle on each sortable header
  (Context, Input $, Output $, Providers, Name).
- **B7 [wire]** **`sort=relevance`** auto-applied when `q` is non-empty (search-as-you-type ranked by
  bm25); fall back to `updated` when `q` clears.
- **B8 [wire]** **`sort=price_out` and `sort=family`** as additional sort options.
- **B9 [control]** **Live facet counts** on capability/modality/status/provider/open_weights chips;
  grey/disable zero-count values. (Data already in `facets`.)

### Group C — API Providers page: filters & sort

- **C1 [control]** **Provider price filter** → `pricing_max` slider + a **free/paid** toggle (`pricing`).
- **C2 [wire]** **Provider sorts** → `pricing` (cheapest first), `api_format`, `model_count`, `name`,
  with `order` toggle.
- **C3 [wire]** **Functional capability filter** (now AND-combined server-side) + live facet counts.
- **C4 [control]** **`q` matches served model names** — surface a hint ("matches providers and the models
  they serve") so users know searching "claude" finds providers serving Claude.

### Group D — Detail rails (cross-provider comparison)

- **D1 [control]** **API Models rail: full "Served by (N)" cost table** — render every `served_by[]`
  provider with its own `$in / $out` (+ cache prices when present), sorted cheapest-first, each row
  deep-linking to the provider page. *(This is the cross-provider cost comparison.)*
- **D2 [wire]** **Provider rail: sortable served-models list** → `providers/{slug}/models?sort=` toggle
  (name/context/price). It's now unpaginated, so the full list is available client-side too.
- **D3 [control]** **Configure-in-Bodhi from the rail** consumes the embedded `bridge`
  (`api_format` + `base_url` + `base_url_requires_substitution`) to prefill `ApiModelForm` — including
  surfacing a substitution affordance for `{AWS_REGION}`-style base URLs. (Verify current prefill path.)

### Group E — Cross-cutting polish (optional)

- **E1** **URL-synced filter/sort/page state** (TanStack search params) so catalog searches are
  shareable/back-button-safe. Today only `?select` is read.
- **E2** **Consolidate the 3 hand-rolled facet sidebars** into one `ShellFilterGroup`-based primitive
  (counts + search-within-facet + overflow) so all three Explore pages share one facet kit.

---

## 5. Control-to-data-cardinality mapping (the "appropriate filters" ask)

| Filter | Cardinality | Recommended control | Backend param |
|---|---|---|---|
| Capability | 5 fixed | chips + counts | `capability` (AND) |
| Modality | 5 fixed | chips + counts | `modality` (OR) |
| Status | 4 fixed | chips + counts | `status` (OR) |
| Open weights | 2 | radio/single chip | `open_weights` |
| API format (providers) | 6 fixed | chips + counts | `api_format` (OR) |
| Input price | continuous | **dual-handle range slider** | `pricing_min` + `pricing_max` |
| Output price | continuous | **range slider (max only)** | `pricing_out_max` |
| Context | continuous (log) | **min slider, stepped marks** | `context_min` |
| **Provider** | **144** | **autocomplete + chip tags** | `provider` (OR), typeahead via `providers?q=` |
| **Family** | **many** | **search-within-facet / autocomplete** | `family` (OR) |
| Free/paid (providers) | 2 | toggle | `pricing` |

**Autocomplete sources:** provider typeahead = `GET /catalog/providers?q=` (returns matching providers).
Family has no dedicated endpoint; either derive the option list from the `facets.provider`-style counts
(no family facet exists yet — see §6) or use a client-side filtered list from the first page. **This is
the one place where a small backend addition would help** (see §6).

---

## 6. Backend gaps / asks (the "if any apis missing" ask)

Things the enriched frontend wants that the backend doesn't fully provide. Ordered by impact.

1. **🔴 Apply the trigram migration on dev (and prod).** Typo-tolerant search is built and typed but
   **`0004_catalog_fts_trigram.sql` is not live on `dev-api`** (`q=clade` → 0). Either it wasn't
   deployed or the index isn't populated. Without it, "typo-tolerant search-as-you-type" can't ship.
   *Lowest effort, highest user-visible payoff.*
2. **🟡 No `family` facet in the models response.** There's a `family` *filter* but no `facets.family`
   bucket, so the UI can't populate a family picker with counts. Add a `family` facet bucket (or a
   `GET /catalog/families` list) to drive B4 cleanly. Workaround: client-derive from the current page.
3. **🟡 `pricing_band` `low/mid/high` are dead.** Either implement them with sensible thresholds or drop
   them from the enum so the type doesn't advertise non-functional values. (We'll use numeric ranges
   regardless.)
4. **🟢 No `context_max` filter** (only `context_min`). Minor — a true context *range* would need it.
5. **🟢 No output-price floor** (`pricing_out_min`). Minor — output filter is ceiling-only.
6. **🟢 `anthropic_oauth` missing from the providers `api_format` facet seed** (valid enum value, omitted
   from the pre-seeded facet set). Cosmetic; the chip just won't show a 0 bucket.
7. **🟢 Provider/family autocomplete endpoints.** Not strictly needed (provider typeahead works via
   `providers?q=`), but a dedicated lightweight typeahead would be cleaner than reusing the full list
   endpoint. Optional.

---

## 7. Pagination migration (Load More → numbered) — specifics

- **Target component:** `components/shell/ShellPagination.tsx` (already built, unused in Explore).
- **Both catalog pages** (`api`, `api-providers`). **Local stays on Load More** (cursor-based, no `total`).
- **Mechanics:** remove `accumulated[]`; render only `data.items`; pass `page`, `total`, `page_size`
  (default 20, consider a rows-per-page select) to `ShellPagination`; `onPage(setPage)`; reset to page 1
  via the existing `resetPaging` on any filter/search/sort change (the reset wiring already exists).
- **Design note:** the mock uses a **full numbered pager** for the long Models catalog and a **minimal**
  pager for finite lists — `ShellPagination` supports both via its `minimal` prop.
- **Keep** `keepPreviousData` so page changes don't flash empty.
- **Tests to update:** the `index.v2.test.tsx` Load-More assertions + MSW handlers; one Playwright spec
  step per page exercising pager → page 2 → filter resets to page 1.

---

## 8. Suggested slice sequencing (for when we move to a plan)

Thin vertical slices, verify each in Chrome, grow one Playwright spec, commit per slice:

1. **A1+A2** types bump + query-builder params (no UI change; unblocks all).
2. **A3** Load More → `ShellPagination` on both pages (the explicit ask; isolated, testable).
3. **B6+B7+B8** sort: asc/desc `order`, relevance-on-search, new sorts.
4. **B1+B2+B3** range sliders (dual-handle price in/out + context marks).
5. **B9+C3** live facet counts on both pages.
6. **B5+B4** provider autocomplete + family filter.
7. **C1+C2+C4** providers page price filter + sorts + q hint.
8. **D1+D2** rail cross-provider cost table + provider rail sort.
9. **D3** Configure bridge prefill (verify/upgrade).
10. **E1/E2** (optional) URL state + facet-kit consolidation.

Backend asks #1 (trigram) and #2 (family facet) should be filed in `api-getbodhi-app` before B4/B7
depend on them.

---

## 8b. Decisions (locked 2026-06-23)

- **Scope:** **Everything incl. rails (Groups A–D)** — but delivered **phase-wise, iterative,
  incremental, test-driven, with a commit between every phase** (per the vertical-slice + TDD charter:
  build a slice, verify in Chrome, grow one Playwright spec with many `test.step`, run all gate checks,
  commit).
- **Typo search:** **Ship prefix-only now** (works today) with relevance ranking; **file the trigram ask**
  so typo tolerance lights up later with no frontend change.
- **Provider filter:** **Autocomplete + removable chip tags** (typeahead via `catalog/providers?q=`),
  scaling to all 144 providers.
- **Backend asks to file in `api-getbodhi-app` (parallel track):** **all three** — (1) apply the trigram
  migration `0004` on dev/prod, (2) add a `family` facet bucket to the models response, (3) drop or
  implement the dead `pricing_band` `low/mid/high` values.

These supersede the open questions below.

## 9. Open questions for the user

1. **Typo tolerance:** ship search-as-you-type now with **prefix-only** matching (works today), or block
   on getting trigram (#6.1) live on dev first?
2. **Provider filter UX:** autocomplete-with-chip-tags (recommended for 144 providers) vs. keep top-N
   chips + a "show all" expander?
3. **Pagination style:** full numbered pager for both, or numbered for Models + minimal for Providers
   (matching the mock)? Add a rows-per-page selector?
4. **Scope of D (rails):** include the cross-provider cost table + Configure-bridge prefill in this round,
   or keep this round to list/search/filter/sort/pagination only?
5. **Backend asks:** file the trigram + family-facet asks in `api-getbodhi-app` now (parallel track), or
   build the frontend against the workarounds and add them later?
