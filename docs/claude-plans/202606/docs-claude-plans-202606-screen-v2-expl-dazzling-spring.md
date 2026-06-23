# Explore · API Models + Explore · API Providers — frontend plan

## Context

BodhiApp's Explore section currently has one page — **Explore · Local Models** (screen-v2, shipped) — a search-driven catalog of downloadable GGUF repos served by the external Reference API. We are adding two sibling pages that together are "models.dev inside Bodhi", both reading a **new, already-deployed** catalog backend (`/api/v1/catalog/*` on `api.getbodhi.app`, currently `dev-api.getbodhi.app`):

- **(B) Explore · API Providers** — provider-first: a list of API providers/gateways (NanoGPT, Kilo, OpenRouter, Vercel…), a detail rail with connection metadata (env vars, base URL, docs, npm) + that provider's models, and cross-links into page A.
- **(A) Explore · API Models** — models-first: a list of individual catalog models with a faceted sidebar (counts), sortable columns, and a per-model detail rail (spec grid + "Served by" providers + a "Configure in Bodhi" CTA that prefills the New API Model form).

**Build order (per user): Providers (B) first, then Models (A).** Work proceeds **phase-wise, iterative, incremental, test-driven** — each phase ships a thin vertical slice, is verified in Chrome, gets component + E2E tests, and is committed before the next phase begins.

**Why now:** the catalog backend shipped, and the published types package `@bodhiapp/reference-api-types@0.0.6` now carries all catalog wire types. These pages turn the catalog into a browse-and-configure experience inside the app.

## Data contract (live API — source of truth)

Base URL is `AppInfo.reference_api_url` (env `BODHI_REFERENCE_API_URL`), surfaced via `useAnonymousReferenceApi()` (catalog is **publicly readable — no `id_token`**). Endpoints:

| Endpoint | Returns | Key query params |
|---|---|---|
| `GET /api/v1/catalog/providers` | `ProviderListResponse` `{items: ProviderSummary[], facets:{capability,api_format}, page,page_size,total}` | `q`, `capability[]`, `api_format[]`, `pricing_max`, `sort('rank'\|'name'\|'model_count')`, `page`, `page_size` |
| `GET /api/v1/catalog/providers/{slug}` | `ProviderDetailResponse` (`env[]`, `npm`, `doc_url`, `api_base_url`, `bridge`) | — |
| `GET /api/v1/catalog/providers/{slug}/models` | `ProviderModelsResponse` `{items: ProviderModelRow[], total}` | `sort('name'\|'context'\|'price')` |
| `GET /api/v1/catalog/models` | `ModelsListResponse` `{items: ModelLite[], facets: ModelFacets, page,page_size,total}` | `q`, `capability[]`, `modality[]`, `pricing_max`, `pricing_band`, `context_min`, `status[]`, `provider[]`, `open_weights('open'\|'closed')`, `sort('updated'\|'context'\|'price'\|'name'\|'providers')`, `page`, `page_size` |
| `GET /api/v1/catalog/models/{slug}/{model_id}` | `ModelDetailResponse` (`served_by: ServedBy[]`, `bridge: ConfigureBridge`, full `Pricing`, capability booleans) | — |
| `GET /api/v1/catalog/logos/{slug}.svg` | provider logo | **currently 404 → monogram fallback** |

All types import from `@bodhiapp/reference-api-types` (never hand-roll). Field shapes are **flat** (`context_limit`, `pricing.input_per_m`, `caps[]`, `modalities_in/out[]`, `status: ModelStatus|null`) — the design-prompt mock's nested models.dev names (`limit.context`, `cost.input`) are illustrative only; **the live API wins**.

**Verified facts (curl'd against dev-api):**
- **Facet counts are recomputed per filtered query** (e.g. `?capability=reasoning` drops total 2565→1292 and shrinks every other count). The sidebar renders counts that update on each settle; render zero-count options as **disabled** (not hidden) so a selected/zeroed option can still be deselected.
- **Status facet uses key `stable`** for null-status rows — so the synthetic "Stable" chip sends `status=stable` (null is not a query value).
- `ConfigureBridge.base_url` can be `null` (and `base_url_requires_substitution` can be `true`, e.g. Bedrock's AWS_REGION placeholder). When null, **omit** base_url from prefill (fall back to the form preset) — never pass `null` into a text field.
- `BridgeApiFormat` (`openai|openai_responses|anthropic|anthropic_oauth|gemini`) maps **1:1** to the form's `ApiFormat` — no provider→format guessing needed; the catalog's `bridge` payload is authoritative.

## Architecture & conventions (mirror existing, do not invent)

The two pages are **exact siblings of Explore · Local Models** and reuse its full pattern:

- **Routes/screens** mirror `crates/bodhi/src/routes/models/explore/local/` — `index.tsx` (wraps screen in `<AppInitializer allowedStatus="ready" authenticated>` + the `MultiTenantGuard` copied verbatim from `local/index.tsx:19-30`) + `-components/{Screen,Sidebar,Rail}.tsx` + a screen `.css`. Screens publish chrome via `useShellChrome({breadcrumb, sidebar, rail, railHeader, railDefaultOpen:false})`, use `useListKeyNav`, `ShellSearch`, `ShellIcon`, `LinkRow`, and the `.l-page/.l-listrow/.l-scroll` shell-list classes. Row select sets `selectedKey`+`railMode` **wrapped in `useViewTransition()`**; the detail is a separate query gated on selection.
- **Hooks** go in a **new** `crates/bodhi/src/hooks/reference/useCatalog.ts`, extending the existing `referenceKeys` factory + endpoint consts in `hooks/reference/constants.ts`. Clone `buildModelsQuery` from `useDiscoverModels.ts` as `buildCatalogModelsQuery` (its repeated-key array serialization is exactly what `capability[]`/`modality[]`/`status[]`/`provider[]` need). All catalog hooks use `useAnonymousReferenceApi()` + `keepPreviousData`.
- **Nav** — add two subPages to the Models item in `crates/bodhi/src/components/shell/shell-nav-config.tsx` (`SHELL_NAV`), alongside `explore-local`. New route constants in `crates/bodhi/src/lib/constants.ts`. (These pages are catalog-browse only — set `hideInMultiTenant: true` to match `explore-local`, and the `MultiTenantGuard` enforces it; also keeps the E2E specs standalone-only since `multi_tenant` excludes `specs/models/`.)
- **No feature flag** (net-new, shipped-only V2).
- **Strict TS, no `any`.** Comment policy: only non-obvious *why*.

### Pagination divergence (page-based, not Local's cursor)

Local uses cursor/keyset (`next_cursor`) with no total and **disables Load-more during search**. The catalog is **page-based** (`{page, page_size, total}`) with a real total. Use a **page-increment "Load more"**: accumulate items, fetch `page+1`, append, dedup by `slug/model_id`; `showLoadMore = accumulated.length < total`; resultbar shows "Showing X of TOTAL". **Reset the accumulator + page synchronously on any param change** (search/sort/facets) — not in a post-settle effect — to avoid `keepPreviousData` appending a stale page-2 onto a new filter's page-1. **Load-more stays available during search** (no cursor-disable) — the inverse of Local; the most likely copy-paste bug.

### Configure-in-Bodhi bridge (prefill seam)

`/models/api/new` has **no `validateSearch` today**. Wire it minimally:
1. Add `validateSearch` (zod) to `routes/models/api/new/index.tsx` for `{ api_format?, base_url?, model? }`.
2. In `NewApiModelContent`, read `Route.useSearch()`, build a typed `prefill: { api_format, base_url?, model? } | undefined` only when params exist, pass as one prop to `ApiModelForm`.
3. Thread `prefill` onto `ApiModelFormProps` → `UseApiModelFormProps`; in `useApiModelForm`, **spread prefill over the existing create-mode `defaultValues`** (the literal at `useApiModelForm.ts:82-95`) — keep the hardcoded OpenAI defaults as fallback, override only when prefill exists. `api_key` is **never** prefilled. Omit `base_url` when the bridge gives `null`. Document that a later format toggle (`handleApiFormatChange`) intentionally resets base_url to the preset.

Do **not** overload edit-only `initialData` (it's `ApiAliasResponse`-shaped). A dedicated `prefill` type is clearer.

### Testing convention (every phase)

Two complementary layers, per house style:
- **Vitest + MSW (precision layer — owns exactness):** component tests using a **new** `test-utils/msw-v2/handlers/reference-catalog.ts` (mirror `reference-models.ts` — **same default origin** as `mockAppInfoReady`'s `reference_api_url`, with the `onRequest` auth-capture callback) + **new** fixture factories in `test-fixtures/catalog-*.ts` (typed by the published types, `Partial` overrides). Mock `useViewTransition` to a pass-through (per `local/index.v2.test.tsx:19`). Assert exact query params (repeated-key arrays), facet counts, empty/error, and — load-bearing — **no `Authorization` header** on catalog reads.
- **Playwright E2E (wiring layer):** one growing spec per page under `specs/models/` (standalone-only) with many `test.step`s, a reusable Page Object extending `BasePage`, `emulateMedia({reducedMotion:'reduce'})` before navigation.
  - **DELIBERATE DIVERGENCE from Local (per user instruction): STUB the reference API.** Local's E2E hits live dev-api with reachability-resilient, shape-not-exact assertions. The user wants the catalog **stubbed** so E2E is deterministic. Use `page.route('**/api/v1/catalog/**', route => route.fulfill({ json, headers:{'access-control-allow-origin':'*'} }))` (precedent: `ChatPage.mjs:443` uses `page.route`). Centralize the stub in the Page Object (`stubCatalog(fixtures)`), routing each catalog endpoint to a fixture mirroring the Vitest fixtures. Determinism lets E2E assert exact facet counts, served-by entries, and **exact** Configure prefill values.
  - Each phase also re-runs the existing `/ui/models/` spec (My Models, `specs/models/`) to guard the shared nav/shell.

### Per-phase loop (exact steps)

For **every** phase below:
1. Implement the slice + write its **Vitest component/hook tests** (red→green).
2. `ports kill 1135 3000`
3. `make app.run.live` → validate the slice in **Claude-in-Chrome** (against live dev-api).
4. Write/extend the **Playwright spec** following convention: **stub the catalog via `page.route`**, grow the page's Page Object, add the phase's `test.step`(s). Run the new spec **and** the `/ui/models/` spec.
5. Run all gate checks (`make format`, `cd crates/bodhi && npm test`, `make test.e2e`).
6. **Commit** the slice. Then continue to the next phase.

## Phase 0 — Dependency + plumbing (no UI)

> Note: `npm install` is a state change and runs as the first build step (outside plan mode).

- Bump `@bodhiapp/reference-api-types` `^0.0.5` → `^0.0.6` in `crates/bodhi/package.json`; `npm install`. (0.0.6 ships all catalog types — verified.)
- Extend `hooks/reference/constants.ts`: add `referenceKeys.catalog*` keys (providers/list, provider/detail, provider/models, models/list, model/detail keyed on serialized params) + catalog endpoint path consts.
- New `hooks/reference/useCatalog.ts`: `buildCatalogModelsQuery` (clone of `buildModelsQuery`), `useCatalogProviders`, `useCatalogProviderDetail`, `useCatalogProviderModels`, `useCatalogModels`, `useCatalogModelDetail` — all anonymous + `keepPreviousData`. Export from `hooks/reference/index.ts`.
- New MSW `test-utils/msw-v2/handlers/reference-catalog.ts` (same origin pin + `onRequest` capture). New `test-fixtures/catalog-providers.ts` + `catalog-models.ts` factories.
- **Vitest:** hook tests — array params serialize as repeated keys; **no Authorization** header on catalog calls.
- Files: `package.json`, `hooks/reference/{constants.ts,useCatalog.ts,index.ts}`, `test-utils/msw-v2/handlers/reference-catalog.ts`, `test-fixtures/catalog-{providers,models}.ts`.

## Page B — Explore · API Providers (built first)

### Phase B1 — Basic page rendering the providers API result
- `routes/models/explore/api-providers/index.tsx` (AppInitializer + MultiTenantGuard) + `-components/ExploreProvidersScreen.tsx` + `.css`. List of `ProviderSummary` rows (name, monogram/logo, model_count, rank, capabilities_summary, min pricing). Resultbar "Showing X of TOTAL". Page-increment Load-more. Empty + error states. Add `SHELL_NAV` subPage `explore-api-providers` + route const.
- Chrome: nav shows "Explore · API Providers"; rows render from live dev-api; Load-more appends, no dups.
- E2E (new `pages/ApiProvidersPage.mjs` + `specs/models/api-providers.spec.mjs`, **catalog stubbed**): step "Open Explore · API Providers and list renders"; "Load more appends a page". Also run `/ui/models/` spec.
- Vitest: rows render from fixture; "Showing X of N" uses total; Load-more append+dedup; empty/error; anonymous read.

### Phase B2 — Detail side-panel (provider rail)
- `-components/ExploreProvidersRail.tsx` + header. Row-select (via `useViewTransition`) → `useCatalogProviderDetail` + `useCatalogProviderModels` (gated on selection). Rail shows `env[]`, `api_base_url`, `doc_url`, `npm`, `provider_shape`, and the provider's models (sortable `name|context|price`), logo→monogram fallback.
- Chrome: clicking a provider opens rail with connection meta + model rows.
- E2E: step "Provider rail shows env/base_url/doc and the provider's models"; close removes panel.
- Vitest: detail + provider-models fetch gated on selection; env/base_url/doc/npm render; provider-models sort param; monogram fallback.

### Phase B3 — Search (+ sort + facets)
- `ShellSearch` → `q` (reset page+accumulator synchronously); sort control (`rank|name|model_count`); facet sidebar `-components/ExploreProvidersSidebar.tsx` from `ProviderListResponse.facets` (capability + api_format multi-select → repeated-key params); clear-all.
- Chrome: typing narrows; sort re-queries; facet filters; clear-all resets.
- E2E: step "Search narrows the provider list"; "Sort by model_count + capability facet filters" (exact, since stubbed).
- Vitest: `q` resets page; sort param; capability/api_format repeated-key params; clear-all empties params; facet counts render.

## Page A — Explore · API Models (built second)

### Phase A1 — Basic page rendering the models API result
- `routes/models/explore/api/index.tsx` (+ guard) + `-components/ExploreApiScreen.tsx` + `.css`. `ModelLite` rows: rank `#`, name **bold** + `family` sub-line, `context_limit` (200K/1M), input/output `$/M` (`Free` when both 0), capability chips (only true caps), `provider_count`. Resultbar "Showing X of TOTAL"; page-increment Load-more. Empty/error. `SHELL_NAV` subPage `explore-api` + route const.
- Chrome: nav shows "Explore · API Models"; rows render; Load-more appends.
- E2E (new `pages/ApiExplorePage.mjs` + `specs/models/api-explore.spec.mjs`, **catalog stubbed**): "Open Explore · API Models and rows render + Showing N of total"; "Load more appends". Run `/ui/models/` spec.
- Vitest: rows render; total in resultbar; Load-more append+dedup; `Free` pricing; empty/error; anonymous read.

### Phase A2 — Detail rail (spec grid + Served by + Configure CTA)
- `-components/ExploreApiRail.tsx`. Row-select (`useViewTransition`) → `useCatalogModelDetail` (gated). Rail: spec grid (context/output limits, full pricing, caps, modalities in/out, status, release/updated, knowledge cutoff — **omit absent fields**, synthesize "Stable" when `status==null`); "Served by" list from `served_by[]` (each provider + its price; deep-link into page B `?select=<slug>`); logo→monogram. **Configure-in-Bodhi CTA** → `/models/api/new?api_format&base_url&model` from `bridge` (omit base_url when null). Wire the bridge prefill seam (validateSearch + prefill prop + `useApiModelForm` spread) described above.
- Chrome: click row → rail with specs + served-by; Configure → New API Model form with api_format+base_url+model preselected, api_key empty.
- E2E: "Opening a model shows the rail with specs + Served-by"; "Configure prefills the create form" — with the stub, assert **exact** prefilled api_format/base_url/model + empty api_key.
- Vitest: select→detail fetch gated; spec grid omit-if-absent + "Stable" synthesis; served_by renders; logo 404→monogram; `validateSearch` parses; create form seeds from prefill; null base_url→preset; api_key never prefilled; BridgeApiFormat→ApiFormat 1:1.

### Phase A3 — Search + facets + sort
- `ShellSearch` → `q`; sortable column headers (`updated|context|price|name|providers`) with active `data-test-state`; `-components/ExploreApiSidebar.tsx` from `ModelFacets`: multi-select capability/modality/status/provider (repeated-key arrays; "Stable"→`status=stable`), tri-state `open_weights` (unset→open→closed→unset), debounced `pricing_max`/`context_min` sliders (single commit on release), `pricing_band` shortcuts that set `pricing_max`; per-option counts (zero-count → disabled, not hidden); clear-all resets arrays + ranges + tri-state. **Load-more stays available during search.**
- Chrome: typing narrows; sort marks active; facet counts show + update on settle; slider fires one request on release; clear-all resets; Load-more present while searching.
- E2E: "Search narrows + Load-more stays present"; "Capability + provider facet narrows and counts render"; "Sort by Context/Price marks active"; "Clear all resets" (exact, stubbed).
- Vitest (owns exactness): `q` resets page, keeps Load-more; each sort header sends correct `sort`; each facet group → repeated-key params; tri-state cycle; sliders set `pricing_max`/`context_min` debounced; "Stable"→`status=stable`; clear-all empties every param; counts render + disabled-when-zero.

## Verification

- **Per phase:** the per-phase loop (Vitest → `ports kill 1135 3000` → `make app.run.live` → Claude-in-Chrome → Playwright spec (catalog stubbed) + `/ui/models/` spec → gates → commit).
- **End-to-end manual walk (after A2/A3 and B2/B3):** from nav, open both Explore pages; on Providers, open a provider rail, follow "View all models from X →" into Models pre-filtered by provider; on Models, open a model rail, click a "Served by" provider into the Providers page, then click "Configure in Bodhi" and confirm the New API Model form is correctly prefilled (api_format, base_url, model) with an empty API key.
- **Gates before every commit:** `make format`; `cd crates/bodhi && npm test`; `make test.e2e` (from `crates/lib_bodhiserver`, dev-server + Vite HMR — no ui-rebuild needed). Never `test.skip` for missing env — throw in `beforeAll`.

## Critical files

- Mirror template: `crates/bodhi/src/routes/models/explore/local/-components/LocalDiscoveryScreen.tsx` + `local/index.tsx` (guard) + `LocalDiscoverySidebar.tsx` + `LocalDiscoveryRail.tsx` + `local-discovery.css`.
- Hooks: clone from `crates/bodhi/src/hooks/reference/useDiscoverModels.ts`; extend `hooks/reference/constants.ts`; new `hooks/reference/useCatalog.ts`.
- Bridge: `crates/bodhi/src/routes/models/api/new/index.tsx`, `components/api-models/ApiModelForm.tsx`, `components/api-models/hooks/useApiModelForm.ts` (create defaults at `:82-95`).
- Nav/consts: `crates/bodhi/src/components/shell/shell-nav-config.tsx`, `crates/bodhi/src/lib/constants.ts`.
- Test precedent: `test-utils/msw-v2/handlers/reference-models.ts`, `test-fixtures/discover-models.ts`, `routes/models/explore/local/index.v2.test.tsx`; E2E `crates/lib_bodhiserver/tests-js/specs/models/local-discovery.spec.mjs` + `pages/LocalDiscoveryPage.mjs` + `pages/ChatPage.mjs` (page.route stub precedent).
- Types: `@bodhiapp/reference-api-types@0.0.6` — `ModelLite`, `ModelDetailResponse`, `ModelFacets`, `ProviderSummary`, `ProviderDetailResponse`, `ProviderModelsResponse`, `ServedBy`, `ConfigureBridge`, `ListCatalogModelsQuery`, `ListProvidersQuery`.
