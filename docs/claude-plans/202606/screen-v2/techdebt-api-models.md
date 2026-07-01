# Tech Debt & Gap Analysis — Explore · API Models / API Providers

**Status:** re-verified against source; only **pending** items retained. **Date:** 2026-07-01 (orig. 2026-06-23).
**Scope:** the two shipped Explore pages (`/models/explore/api`, `/models/explore/providers`), the Configure-in-Bodhi create flow, and the reference-api catalog backend that feeds them.

## How this was produced

Original doc was a side-by-side comparison of the **design mock** (`design/models/*.jsx`) against the live implementation, plus a source read of the **reference-api** catalog backend (`BodhiSearch/api-getbodhi-app`). On 2026-07-01 every item was re-verified against current source; resolved items were dropped and the remaining pending items had their evidence/line refs refreshed.

---

## Pending priorities (user-flagged)

Of the five originally flagged, **#2 (relevance default) and #4 (numbered pagination) are done**. Still open:

1. **Provider search must be a real backend search API (R4).** Model search now uses FTS5 bm25, but **provider search is still an in-memory `${name} ${slug}`.includes(q)`** with no `providers_fts` table — searching "claude"/"llama" won't surface Anthropic/Meta-serving providers.
2. **Provider relevance ranking (R2).** Default provider `rank` is still raw `model_count DESC`, burying OpenAI/Anthropic/Google under gateway aggregators. Needs a curated-rank / `tier` / `is_canonical` signal.
3. **Create API models by selecting providers — provider-first flow (F7).** Today the only path is the per-*model* "Configure in Bodhi" bridge. Design supports pick-a-provider → prefill `api_format`+`base_url` + "Prefilled from {provider}" banner + auto-name `<provider>-api`.

### Cross-cutting: version-bump sequencing (still gates R2/R7 residual)

Any new wire field (provider `tier`/`is_canonical`, `featured`/`curated` on `ModelLite`) lands in `@bodhiapp/reference-api-types` first → minor bump + publish → bump the **pinned version** dep in BodhiApp (never `file:`) → regen. The worker's `schemas/catalog.ts` zod enums have **compile-time parity guards** — change them in lockstep. Redefine `rank`'s meaning additively rather than renaming (a rename is breaking).

---

## Layer 1 — Reference API (`BodhiSearch/api-getbodhi-app`)

The API-models catalog is D1-backed via `worker/services/real/RealCatalogReader.ts` + `catalogMap.ts` (NOTE: `RealCatalog.ts` + `schemas/query.ts` are the *separate* HuggingFace local-models system — not this surface).

### R2 — [CRITICAL] Default provider sort ranks aggregators above canonical providers
- **Now:** `sort='rank'` still orders by `model_count DESC` (labRank first) at `RealCatalogReader.ts:230`; `ProviderSummary.rank` is just the 1-based page position (`:248`). No `curated_rank`/`provider_tier`/`is_canonical` column on the `providers` table (`packages/infra/src/db/schema.ts:30-52`); no migration adds one (`drizzle/0001`–`0009`). So NanoGPT/Kilo/OpenRouter dominate; OpenAI/Anthropic sink.
- **Fix:** add `curated_rank` (INTEGER nullable, lower = more prominent) and/or `provider_tier`/`is_canonical` to `providers` (new drizzle migration + `catalog-ddl.ts` e2e DDL). Seed from a checked-in allowlist in `normalize.ts`/`ingest.ts` (models.dev has no popularity signal — curation is BodhiApp-owned). A partial signal already exists: `provider_shape` (`native`/`anthropic`/`openai` = first-party vs `openai-compatible`/`openrouter` = gateway) via `deriveShape` (`packages/infra/src/catalog/normalize.ts`) but is unused for ordering. Redefine `sort='rank'` → `ORDER BY curated_rank ASC NULLS LAST, model_count DESC, slug ASC`; keep `sort='model_count'` as the raw escape hatch. Expose `tier` in `ProviderSummary`.

### R4 — [HIGH] Provider search has no FTS and no model/family signal
- **Now:** `listProviders` still does `${p.name} ${p.slug}`.includes(q)` only (`RealCatalogReader.ts:206`). No `providers_fts` table exists (grep over `drizzle/*.sql`). Model FTS is used only as an indirect tier-1 hint. Searching "claude"/"llama" won't surface Anthropic/Meta-serving providers.
- **Fix:** add a `providers_fts` FTS5 table (name, slug, + denormalized served-family bag) with triggers (new migration + `catalog-ddl.ts` + `dropCatalogSchema`), OR derive provider hits from `models_fts` (DISTINCT provider_slug whose models match) UNION name/slug matches, bm25-ranked.

### R5 — [MEDIUM] `logicalModels` picks the cheapest provider as the representative row (aggregator wins)
- **Now:** primary serving row = `min(input_per_m)` then model_count desc then slug (`packages/infra/src/catalog/normalize.ts:205-206`) — a cheap aggregator becomes a model's representative provider over the native one.
- **Fix:** lead the primary-row selection with `tier` (prefer tier-0 native), then price. (Depends on the `tier` signal from R2.)

### R7 (residual) — [MEDIUM] `@bodhiapp/reference-api-types` still missing the ranking/featured fields
- **Now:** package at **0.0.13**; `relevance` present in both sort enums (`packages/api-types/src/index.ts`). But `ProviderSummary` has **no `tier`/`is_canonical`** (`index.ts:292-309`) and `ModelLite` has **no `featured`/`curated`** (`index.ts:373-390`).
- **Fix:** add `tier`/`is_canonical` to provider types and `featured`/`curated` to `ModelLite` (all additive). Mirror in `worker/schemas/catalog.ts` zod (compile-time guards enforce parity). Minor bump → publish → bump BodhiApp pinned dep → regen. **Gates R2/R8's wire exposure.**

### R8 — [LOW] `providerSummary.rank` is page-position, not a stable signal
- **Now:** still recomputed per request as `(page - 1) * pageSize + i + 1` (`RealCatalogReader.ts:248`).
- **Fix:** populate from `curated_rank` (stable) once R2 adds it.

---

## Layer 2 — BodhiApp Backend (`crates/`)

No pending catalog work. The two pages read the reference-api anonymously from the browser (B1, confirmed); `ApiAliasResponse` already exposes `api_format`/`base_url`/`has_api_key` for the connection-state join (B2, fed to F6); `BODHI_REFERENCE_API_URL` is env-aware (B3). The only remaining "backend" work is the **frontend-local** connection-state join (per-provider "Connected" derived from the user's own aliases) — tracked under F6.

---

## Layer 3 — BodhiApp Frontend (`crates/bodhi`)

### F1 — [HIGH] Search input has no debounce / min-length / reset-on-change
- **Now:** the API/providers search input fires `onChange` immediately (commits on Enter); no debounce, no min-length, no in-flight handling, no reset-to-page-1 on query change (`ExploreApiScreen.tsx` search input ~`:248`; range sliders are debounced in `ExploreApiSidebar.tsx` but text search is not). No debounce tests in `explore-api-search.test.ts`.
- **Fix:** debounce + min-length + in-flight handling, reset to page 1 on query change, distinct **search-no-results** state (see F9).
- **Files:** `ExploreApiScreen.tsx`, `ExploreProvidersScreen.tsx`, `hooks/reference/useCatalog.ts`, `useCatalogScreenState.ts`.

### F4 (residual) — [HIGH] API-model detail rail missing "Configured in your API models" + Configure CTA
- **Now:** the rail already renders multi-section content — Capabilities, Specs (cost/limits/modalities/meta), and **Served by** with per-provider logo/name/base_url/`$in`/`$out` pricing rows (`ExploreApiRail.tsx`). What's **missing**: a **"Configured in your API models"** section listing the user's aliases exposing this model (connected/no-key), and a dedicated **Configure CTA** — today each served-by row only offers an "Add" link (`ExploreApiRail.tsx:138-146`).
- **Fix:** add the "Configured in your API models" section (depends on the F6 alias join) + an "Add an API model for {name}" prefill link when none, and a Configure CTA deep-linking to create with provider+model prefill.
- **Files:** `ExploreApiRail.tsx`, `-shared/catalog-format.ts`.

### F6 — [HIGH] Providers page: Connected status, STATUS facet, logos, "API models using this provider", Manage CTA
- **Now:** `ExploreProvidersSidebar.tsx` has only Browse/Labs, Capability, API-format, Pricing facets (`:72-127`) — **no STATUS facet**. Rail shows **no Connected badge**. Logos are always monogram (`ExploreProvidersRail.tsx:27`); `logo_url` is available in the payload but not consumed on rows/rail. Cross-link to the API screen pre-applying the `provider` facet is wired (`ExploreProvidersRail.tsx:76-82`), and an "Add API Model" CTA exists (`:85-92`), but there's **no "Manage Connection" state** and **no "API models using this provider" backlink**.
- **Fix (per design `models-rows.jsx:135`, `models-detail.jsx:320-383`):**
  - **Connected badge** on rows + rail header (compute from user aliases — fields already on `ApiAliasResponse`).
  - **STATUS facet** single-select "Connected" in `ExploreProvidersSidebar.tsx`.
  - **"API models using this provider"** rail section → LinkRows to My Models.
  - **Manage Connection** rail footer state when connected (the create CTA already exists for the not-connected case).
  - **Logos:** consume `provider.logo_url` with `<img onError>` → monogram fallback.

### F7 — [HIGH] Provider-first create flow + "Prefilled from {provider}" banner
- **Now:** create is per-model only. `routes/models/api/new/index.tsx` accepts `api_format`, `base_url`, `model`, `name` prefill params (`:12-17`); the form prefills from them (`useApiModelForm.ts:94-107`). There is **no `provider` param**, no provider-first entry, and no "Prefilled from {provider}" banner. The Providers page "Add API Model" link passes the provider `name` (`ExploreProvidersRail.tsx:57`), not a structured `provider`.
- **Fix (per design `api-model-form.jsx:183-257`):** extend the new-api-model search schema + `ApiModelPrefill` to accept `provider`; render a "Prefilled from {provider}. Add your API key to finish." banner and auto-name `<provider>-api`. Add a provider picker/combobox on the create form for users landing without `?provider=`. **Decide:** does selecting a provider lock/hide `api_format`? Reuse the catalog `bridge` (not the legacy `components/api-models/providers/constants.ts` chip table — stale; mine it for labels or retire it).

### F9 — [MEDIUM] Filtered empty-state has no "Clear filters" action
- **Now:** `EmptyState` renders `title="No models found"` / `sub="Try a different search or filters."` with **no action button** (`ExploreApiScreen.tsx:270-276`); `EmptyState.tsx` has no built-in action support.
- **Fix:** titled empty state + a Clear-filters button resetting facets (design `models-main.jsx:176-182`). Also serves the F1 search-no-results state.

### F12 (residual) — [LOW] `aria-sort` missing on sortable column headers
- **Now:** the numbered pager has `aria-label="Pagination"` + `aria-current="page"` (`ShellPagination.tsx:69,87`) ✓ and `FacetCombobox` has `role="combobox"` ✓. But the sortable `ColSort` header button has **no `aria-sort`** despite tracking the active sort (`-shared/catalog-table.tsx:57-66`).
- **Fix:** add `aria-sort` (ascending/descending/none) to sortable headers.

---

## Suggested execution sequence

Ordered so each step unblocks the next (types/version bump gates downstream wire fields):

1. **Reference-api data model + ranking (R2, R5):** add `curated_rank`/`tier` columns + ingest seeding + redefined default provider sort. New drizzle migration + `catalog-ddl.ts`/`dropCatalogSchema` + seed.
2. **Reference-api provider search (R4):** add `providers_fts` (or model-derived provider hits), bm25-ranked. MATCH escaping + tests.
3. **Reference-api types + version bump (R7 residual, R8):** add `tier`/`is_canonical` (provider) + `featured`/`curated` (`ModelLite`); populate `rank` from `curated_rank`. Publish minor → bump BodhiApp pinned dep → regen. **Gates FE work needing new fields.**
4. **Frontend search hardening (F1) + empty-state (F9) + a11y (F12).** Highest user-visible, no backend dependency.
5. **Frontend provider-first create (F7) + Providers connection features (F6) + the alias join.**
6. **Frontend rail residual (F4 "Configured in your API models" + Configure CTA)** — depends on the F6 alias join.

---

## Testing (per testing-depth + blackbox-e2e conventions)

- **Reference-api (vitest-pool-workers):** provider-search matching served families + MATCH-injection escaping; `curated_rank`/`tier` provider ordering; `providers_fts` added to `catalog-ddl.ts` + `dropCatalogSchema`.
- **Frontend (Vitest + MSW):** server-driven search debounce / min-length / no-results / reset-to-page-1; provider-first prefill (`?provider=` banner + name seed); Connected badge/STATUS facet from mocked aliases; rail "Configured in your API models" rendering; `aria-sort` on headers.
- **Playwright E2E (stubbed catalog):** search→provider-first-create happy path; provider Connected badge + STATUS facet; served-by→provider and view-all cross-link round-trips.
- **States:** search-no-results, API/network-error, page-out-of-range clamp.

## Reference paths
- **Design source:** `design/models/{models-main,models-rows,models-detail,models-filters,bodhi-models-app,bodhi-models-data,api-model-form,model-access-picker}.jsx`, `design/shared/bodhi-list.jsx`.
- **Live FE:** `crates/bodhi/src/routes/models/explore/{api,providers}/-components/{ExploreApiScreen,ExploreApiRail,ExploreApiSidebar,ExploreProvidersScreen,ExploreProvidersRail,ExploreProvidersSidebar}.tsx`, `routes/models/explore/-shared/{catalog-format.ts,FacetCombobox.tsx,catalog-table.tsx}`, `hooks/reference/{useCatalog.ts,useReferenceApi.ts}`, `hooks/.../useCatalogScreenState.ts`, `components/api-models/{ApiModelForm.tsx,hooks/useApiModelForm.ts}`, `components/shell/ShellPagination.tsx`, `routes/models/api/new/index.tsx`.
- **Reference-api:** `apps/api-getbodhi-app/worker/{routes/catalog.*.ts,schemas/catalog.ts,services/real/RealCatalogReader.ts,services/real/catalogMap.ts,drizzle/*.sql}`, `apps/api-getbodhi-app/e2e-cf/helpers/catalog-ddl.ts`, `packages/infra/src/catalog/normalize.ts`, `packages/infra/src/db/schema.ts`, `packages/api-types/src/index.ts`.
</content>
</invoke>
