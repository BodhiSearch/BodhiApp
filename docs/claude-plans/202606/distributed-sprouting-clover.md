# Plan — Batch 3-6 Phase 2: Wire "Explore · Local Models" to the live Reference API

## Context

We are migrating BodhiApp's Models screens to screen-v2. The **Explore · Local Models**
discovery view does **not exist yet** — this batch is its first implementation. It is a
catalog of downloadable GGUF models with a faceted sidebar, a sortable repository list, and a
detail rail showing a **Download-options (quants)** table and a **Pull** action.

The catalog is served by an **external Reference API** (`dev-api.getbodhi.app`, prod
`api.getbodhi.app`) that BodhiApp calls directly from the browser with the user's OAuth
`id_token`. Phase 1 (the Cloudflare API) is built in a **separate repo** and is already
deployed on dev. This plan is **Phase 2 only** — the BodhiApp UI.

**Why now / why a re-plan.** The earlier plan docs in `docs/claude-plans/202606/screen-v2/`
(`batch-3-6-local-discovery-plan.md`, `models-local-discovery-reference-api.md`, the
enhancement brief) were written against a *richer* contract than what actually shipped. The
authoritative current truth is the **shipped API + its functional docs**:
`…/api-getbodhi-app/docs/functional/{README,endpoints,recipes,openapi.json}.md` and the
integration kickoff `…/api-getbodhi-app/docs/functional/BODHIAPP-INTEGRATION-KICKOFF.md`.
I verified every delta below against the **live dev API** on 2026-06-21. Where the shipped API
differs from the old contract, **we follow the shipped API** and reconcile the contract doc.

**Reference only (do NOT copy from):** `design/Bodhi Models Local.html` and its modularized
`design/models/*.jsx` were refreshed to match the lean API — they convey **UX intent** only.
We build to `crates/bodhi/` conventions (TanStack Router routes, `src/hooks/<domain>/`,
`@bodhiapp/ts-client` types, shell components, MSW v2), not the prototype's structure or code.

### Scope decisions (locked with the user)

1. **Dropped facets → omit entirely.** The updated design mock already removed them. Ship only
   the facets v1 backs (below). No "coming soon" placeholders for capability / GB-size slider /
   context slider / quant bit-width / quant method / Staff-Picks.
2. **README tab → drop.** The API does not return README in v1 (verified: not in the OpenAPI
   `Model` schema; `?include=readme` is a no-op live). Detail rail ships **Overview +
   Download-options** tabs only.
3. **Host-fit → defer entirely.** No "Fit / Partial Fit / Too large" pills and no "will it run?"
   badge. Host RAM/VRAM detection is BodhiApp-local with no API field and no backend endpoint
   yet — a separate workstream. Download-options rows show **name · size · recommended · Pull**.
4. **Nav sub-page, NO flag.** It's net-new (not replacing a V1 screen), so ship it on by default
   as a Models sub-page. Do **not** add a `UiV2Screen` flag entry.

## The shipped API (verified live 2026-06-21)

Base URL from `AppInfo.reference_api_url` (already wired). Auth: `Authorization: Bearer
<id_token>` (already attached by `referenceApiClient`). Public read-through (anonymous → 200);
present-but-invalid token → 401. CORS `*`.

**Two endpoints only** (no `taxonomy`, no `orgs`):

- `GET /api/v1/models` — list/search/filter/sort, keyset pagination.
- `GET /api/v1/models/{source}/{namespace}/{repo}` — single model + `quants[]`.

**List query params we use** (all verified): `q`, `author` (repeatable; **renamed from
`namespace`**), `pipeline_tag` (Task: `text-generation` default | `image-text-to-text`),
`specialisation` (`coding|reasoning|vision`, repeatable AND), `tag` (repeatable), `language`
(repeatable), `license` (repeatable), `library`, `sort`
(`downloads|likes|last_modified|trending|created_at`, default `downloads`), `order`
(`asc|desc`), `params_min`/`params_max` (billions; **page-local** — applied within the fetched
page only), `cursor`, `limit` (1–200, default 50). `type=gguf`/`source=huggingface` are the v1
defaults; anything else → 422.

**Response envelope:** `{ items: Model[], next_cursor: string|null, total_estimate: null }`.
- `total_estimate` is **always null** → render "Showing N", never a total count.
- Keyset `cursor`: pass `next_cursor` back with identical filters/sort; `null` = last page.
- **Setting `q` disables the cursor** (`next_cursor` always null when searching) → raise `limit`,
  no paging during search. `sort=trending` **does** return a cursor (paginates fine).

**Model DTO (key fields):** `source, type, namespace, repo, pipeline_tag, library, license,
languages[], tags[], capabilities[], specialisation[], quant_count, quant_bits[],
quant_methods[], max_quant_size_bytes, total_size_bytes, architecture, context_max, params_b
(STRING), provider, downloads, likes, trending_score, created_at, last_modified, curated,
owner_verified, fetched_at`. On the single-model response, add `quants: [{ name, size, bits,
method, recommended }]`.

**Critical row-render constraints (verified):**
- **`max_quant_size_bytes`, `total_size_bytes`, `context_max`, `architecture` are typically
  null on LIST rows** — populated only on the single-model detail (it reads the file tree). So
  the "up to <size>" and context **cannot** render from the list; show them in the detail rail.
- `quant_count`, `quant_bits[]`, `quant_methods[]` summaries **are** present on list rows → use
  for the "N quants" row meta.
- `params_b` is a **string** (e.g. `"7"`) — `Number(...)` before comparing/formatting.
- Sizes are **bytes**. Some quant `size` values are null even on the detail (split/edge repos) —
  render those gracefully.
- `pipeline_tag` filtering is best-effort upstream (an `image-text-to-text` request can include a
  `text-generation` row) — treat the Task facet as a soft filter, not a hard partition.

**Errors:** `400` validation · `401` unauthorized · `404` `{"error":"not_found"}` · `422`
`{"error":"unsupported_source"|"unsupported_type"}` · `500` internal.

## Types: `@bodhiapp/reference-api-types`

The reference API publishes its wire types to npm as **`@bodhiapp/reference-api-types`** (the
single source of truth — the API worker imports the same types). We consume it directly; see Work
item 1. Source of truth for shapes:
`…/api-getbodhi-app/packages/api-types/src/index.ts` (and the live `openapi.json`).

## What already exists (reuse — do not rebuild)

- **`src/lib/referenceApiClient.ts`** — `createReferenceApiClient(baseUrl, idToken)`; `fetch`
  wrapper, attaches Bearer, throws `ReferenceApiError(status, body)`. MSW-interceptable.
- **`src/hooks/reference/useReferenceApi.ts`** — `useReferenceApi(): ReferenceApiClient | null`;
  composes base URL from `AppInfo.reference_api_url` + id_token from `useGetUser`. Returns null
  until ready → gate downstream queries on `enabled: !!client`.
- **`src/hooks/reference/constants.ts`** — `referenceKeys.all = ['reference']` (extend here).
- **`src/routes/models/-components/ModelsScreenV2.tsx`** — the proven list+sidebar+rail template:
  `useShellChrome({ breadcrumb, sidebar, rail, railHeader, railDefaultOpen })`, `useListKeyNav()`,
  `useViewTransition()`. Clone its structure; `ModelSidebarFacets.tsx` + `ModelDetailRail.tsx`
  are facet/rail templates.
- **`src/hooks/models/useDownloads.ts`** — `usePullModel({onSuccess,onError})` → `POST
  /bodhi/v1/models/files/pull` with `NewDownloadRequest = { repo, filename }`; `useListDownloads(
  page, pageSize, { enablePolling })` polls every 1s. Reuse verbatim for Pull.
- **Shell components** in `src/components/shell/`: `ShellFilterGroup`, `ShellSearch`, `ShellIcon`,
  `LinkRow`, `useListKeyNav`, `useShellChrome`.
- **MSW v2** in `src/test-utils/msw-v2/`: `typedHttp`/`setupServer`; `handlers/info.ts`
  `mockAppInfo({ reference_api_url })` already lets tests set the external base URL.
- **Deployment mode:** `appInfo.deployment === 'multi_tenant'` (read like `AppInitializer.tsx`).

## Building blocks (referenced by the phases below)

> These are the component-level specs. **Execution is phased** — see "## Phased execution" after
> this section. Each phase implements a thin vertical slice from these blocks, verifies it live in
> Claude-in-Chrome, extends one growing E2E spec, and commits before the next phase starts.

### 1. Reference-API types — use the published `@bodhiapp/reference-api-types`

**Do NOT hand-roll types.** The reference API publishes its wire types as
**`@bodhiapp/reference-api-types`** (npmjs; types-only, no runtime/fetch — it is the *single
source of truth*, imported by the API worker itself so types and server cannot drift). Add it as a
dependency in `crates/bodhi/package.json` and import from it everywhere instead of defining local
shapes.

- **Dependency:** the reference API is a **separate repo**, so depend on the **published npm
  package** with a version range (e.g. `"@bodhiapp/reference-api-types": "^0.0.2"`) — NOT a local
  `file:` path (that's only how the in-repo `ts-client` is wired). Confirm the exact published
  version to pin (npm currently shows `0.0.1`; the repo's package.json is `0.0.2` — pin to the
  latest published one; bump when the API ships new fields). Run `npm install` after adding.
- **Exports to use** (verified from the package): `Model`, `Quant`, `ListModelsQuery`,
  `ListModelsResponse`, `GetModelQuery`, `GetModelResponse`, `ErrorResponse`, `ErrorCode`, and the
  enums `ModelType`, `ModelSource`, `Specialisation`, `SortKey`, `SortOrder`.
- **Why this matters:** `ListModelsQuery` declares **only the params the API actually supports** —
  building the request object from it makes any dropped filter (capability, size, context, quant
  bits/method, curated) a **compile-time error** rather than a silent no-op. `Model.quants?` is
  optional (detail-only); the nullability of list-only fields is encoded in the types.
- Use these types directly in the discovery hooks/components; no `discoverTypes.ts` wrapper needed
  (a tiny local alias is fine only if a component wants a narrower view).

### 2. Discovery hooks — `src/hooks/reference/useDiscoverModels.ts`

- `useDiscoverModels(params: ListModelsQuery)` — wraps
  `useReferenceApi().get<ListModelsResponse>(...)` for `GET /api/v1/models`. `enabled: !!client`,
  `placeholderData: keepPreviousData` (3-1 carry-forward: no empty-flash on facet/sort change).
  Build the query string from `params` (typed by `ListModelsQuery`, so only supported params
  compile); **omit empty values**; serialize repeatables (`author`, `specialisation`, `tag`,
  `language`, `license`) as repeated keys.
- `useModelDetail(source, namespace, repo)` — `GET /api/v1/models/{source}/{namespace}/{repo}`;
  `enabled: !!client && !!selected`. Returns `GetModelResponse` (a `Model` with `quants[]`
  populated). (No `?include=` needed: README is dropped; quant sizes come back on the base detail
  response.)
- Pagination: expose `next_cursor` so the view can do "Load more" (append pages). When `q` is set,
  the hook should not surface a cursor (API returns null) — the view raises `limit` instead.
- Extend `referenceKeys` with a `discover` factory: `discoverList(paramsKey)` and
  `discoverDetail(source, ns, repo)` (stable key from a normalized params object, like
  `modelKeys.list`'s `filterKey`).

### 3. Route + view — `src/routes/models/explore/local/`

Clone `ModelsScreenV2.tsx` into a new route component (TanStack file route
`/models/explore/local/`, `validateSearch` with a Zod schema for the filter/sort/search state so
deep-links and back/forward work). Publish chrome via `useShellChrome({ breadcrumb: Models ›
Explore · Local Models, sidebar, rail, railHeader, railDefaultOpen: false })`. `useListKeyNav()`
for arrow nav; `useViewTransition()` for selection transitions. **One `useShellChrome` publisher
per screen** (3-4 carry-forward); publish `sidebar` memoized on filter state.

### 4. Sidebar facets (hardcoded enum sets — no taxonomy endpoint)

There is no `/api/v1/taxonomy`; hardcode the small known sets client-side (kickoff §6). Render
with `ShellFilterGroup`. Facets → query params (matching the updated design mock exactly):

- **Source** — static "HuggingFace" (display only; always `source=huggingface`).
- **Specialisation** — All / Coding / Reasoning / Vision → `specialisation` (repeatable AND).
- **Task** — Text Generation (default) / Image-Text-to-Text → `pipeline_tag` (soft filter).
- **Tag (advanced)** — tool-use, conversational, thinking, moe, embedding → `tag` (repeatable).
- **Publisher** — **free-text** input (org or author), maps to `author` (repeatable). **No
  autocomplete dropdown** (no `/orgs` endpoint); accept free text, support multiple as chips.
- **Language** — en/zh/es/fr/de/ja/ko → `language` (repeatable).
- **License** — Apache-2/MIT/Llama/Gemma/DeepSeek → `license` (repeatable).
- **Browse** — Trending (`sort=trending`) / New (`sort=created_at`) chips.
- Active-filter chip row + clear-all (mirror the prototype's affordance, our components).

**Explicitly NOT built:** capability facet, GB-size slider, context slider, quant bit-width,
quant method, Staff-Picks/curated filter (all dropped in v1; the updated mock omits them).
`params_min`/`params_max` exist in the API but are page-local — **out of scope** this batch
(would mislead as a "filter"); revisit if needed.

### 5. List

Repository rows: `namespace/repo` (org clickable → sets `author` filter), tag chips, an `arch`
chip when present, `owner_verified` ✓ and `trending` flame badges, `multimodal` marker when
`pipeline_tag === 'image-text-to-text'`. **Columns = Downloads + Likes** (sortable; show/hide
menu), default **sort = Downloads desc** (API default; the mock shows Trending — either is fine,
pick Downloads to match the API default and keep cursor paging predictable). `#` rank = **client
list ordinal**, not a server field.

Row meta uses **only list-available fields**: `quant_count` ("N quants"), `license`. **Do NOT**
render "up to <size>" or context on the row (null on list rows). Result bar: "Showing N" +
"sorted by <label> · <dir>" (no total — `total_estimate` is null). "Load more" appends the next
`next_cursor` page; when `q` is set, hide "Load more" and rely on a higher `limit`. Search box =
`q` (placeholder "Search HuggingFace repos — ⌘K"), submit-on-Enter, clear-resets (3-1 pattern).
Empty state when `items.length === 0`.

### 6. Detail rail — Overview + Download-options (two tabs; no README)

Fetch via `useModelDetail` on selection. **Header:** `namespace/repo` (org clickable), copy-id,
chips (params from `params_b`, architecture, format GGUF), stat row (downloads, likes, relative
`last_modified`, `trending_score`). **Overview tab:** capability badges (`capabilities[]`), specs
(Context=`context_max`, Architecture=`architecture`, Parameters=`params_b`, License=`license`,
Created=`created_at`). **Download-options tab:** render `quants[]` straight from the detail
response — per row: quant `name`, `size` (format bytes → GB; "—" when null), a **Recommended**
badge when `recommended`, and a **Pull** button. Footer: **Pull recommended quant** (the
`quants.find(q => q.recommended)` or first) + **Add to Bodhi**. **No host-fit pills. No README
tab.** ("More from <org>" cross-discovery is optional/nice-to-have; if included, it's just a
preset `author` filter — no extra endpoint.)

### 7. Pull wiring (the bridge into 3-4/3-5)

Reuse `usePullModel()` → `POST /bodhi/v1/models/files/pull` with `{ repo: ` `${namespace}/${repo}` `,
filename: <chosen quant's filename> }`; poll `useListDownloads(…, { enablePolling: true })` (1s)
for progress, surface inline. **Quant → filename:** the API gives `quant.name` (e.g. `Q4_K_M`),
not the literal `.gguf` filename. Resolve the filename from the detail's file list if exposed, or
construct/confirm the convention used by the existing pull flow — **verify against a real Pull in
GATE B** (this is the one spot where quant-name→filename mapping must be exact). **Hide Pull in
MultiTenant** (HubService rejects downloads there): gate on `appInfo.deployment === 'multi_tenant'`
(and/or `power_user` scope) — show browse-only catalog there.

### 8. Nav (no flag)

Add an **Explore · Local Models** sub-page under Models in
`src/components/shell/shell-nav-config.tsx` pointing at `/models/explore/local/`. **Do not** add
a `UiV2Screen` flag entry (per decision 4). Add a `ROUTE_*` constant in `src/lib/constants.ts`.

### 9. MSW external-origin stub — `src/test-utils/msw-v2/handlers/reference-models.ts`

Stub the external origin (from `mockAppInfo({ reference_api_url })`) so the batch is
self-sufficient and never blocked on the live API:
- `GET <base>/api/v1/models` — honor `q`/`author`/`sort`/`specialisation`/`tag`/`language`/
  `license`/`pipeline_tag`/`cursor`/`limit`; return `{ items, next_cursor, total_estimate: null }`
  with **list-shaped rows** (null `max_quant_size_bytes`/`context_max`/`architecture`, present
  `quant_count`/`quant_bits`/`quant_methods`, string `params_b`). Provide a small fixture set that
  exercises trending vs downloads sort, search (cursor null), and the multimodal row.
- `GET <base>/api/v1/models/huggingface/{ns}/{repo}` — full DTO **with `quants[]`** (incl. one
  `recommended`, one null-size quant) and populated detail-only fields.
- **Assert `Authorization: Bearer <id_token>`** is present (id_token from the logged-in-user mock).
- Error fixtures: 401 (invalid token), 404 `{"error":"not_found"}`, 422
  `{"error":"unsupported_source"}`.
Add a `src/test-fixtures/discover-models.ts` factory (typed by `@bodhiapp/reference-api-types`'s
`Model`/`Quant`, `Partial<T>` overrides) for list rows + detail. Typing the fixtures by the
published `Model` keeps the stub honest against the real wire shape. Keep the stub aligned to the
shipped API; if the contract shifts (and a new `@bodhiapp/reference-api-types` version ships),
bump the dep and follow it.

### 10. Reconcile the contract docs

Update `docs/claude-plans/202606/screen-v2/models-local-discovery-reference-api.md` and
`batch-3-6-local-discovery-plan.md` with the shipped deltas (drop taxonomy/orgs endpoints,
`namespace`→`author` query param, dropped filters, README not returned, null list-row
sizes/context, `total_estimate` null, `params_b` string). Mark the
`…/api-getbodhi-app/docs/functional/` set as the source of truth. Update
`docs/claude-plans/202606/screen-v2/tracker.md` Batch 3-6 status.

## Phased execution

**Build incrementally, not in one big change.** Each phase = a thin vertical slice → **verify
live in Claude-in-Chrome** → extend the ONE growing E2E spec → run gate checks → **commit** →
plan/start the next phase. Do not move to the next phase until the current one is verified +
committed.

### The dev loop (every phase)

- Most work is frontend → **Vite HMR reloads automatically** when `make app.run.live` is already
  running. If the server isn't running or HMR wedges: `ports kill 1135 3000` then `make
  app.run.live` (memory: ports-cli; cap housekeeping ~30s). 1135 = bodhiserver_dev, 3000 = Vite.
- **Verify each slice in Claude-in-Chrome** against the new sub-page (real dev API
  `dev-api.getbodhi.app` for the live walk; MSW stub backs the tests). Check light+dark+responsive
  and a clean console as the slice grows.
- **One growing E2E spec** at `crates/lib_bodhiserver/tests-js/specs/models/local-discovery.spec.mjs`
  (+ a page object under `pages/`). **Few specs, many `test.step()`** — E2E is expensive, so each
  phase *adds `test.step` blocks to the same test*, not new test files. **Load the `playwright`
  skill before writing/extending the E2E** (data-testid / data-test-state, page-object, test.step
  structure). Black-box only — no `page.evaluate`/context fetch (memory: blackbox-e2e);
  `reducedMotion:'reduce'` for the V2 rail (memory: view-transition races); **never silently
  skip** — throw in `beforeAll` if a required stub/env is absent (memory: no-skip-for-missing-env).
- **RTL (`cd crates/bodhi && npm test`)** for the slice's logic (hooks/query-params/components)
  against the MSW stub, alongside the E2E step.
- **Gate before each commit:** `npm run format` + lint + typecheck + RTL + the E2E spec (memory:
  run-all-gate-checks). No backend Rust change expected (Pull endpoint exists) → no ts-client
  regen. Feature rollout → **commit per phase** (memory: layered-refactors).

### Phase 1 — Minimal search-only list (vertical slice, no filters)

Smallest end-to-end slice: the route renders a searchable, sortable list from the live API.
- Add `@bodhiapp/reference-api-types` dep (block 1). Build `useDiscoverModels` (block 2, list
  only — no detail yet) + the `referenceKeys.discoverList` factory. Add the route + nav sub-page
  (blocks 3, 8) and the cloned shell with breadcrumb + list + search box only (no sidebar facets,
  no rail yet). List rows + "Showing N" + Downloads/Likes columns + default sort + "Load more"
  (block 5). MSW stub for `GET /api/v1/models` only (block 9, list handler + `Authorization`
  assertion).
- **Verify in Chrome:** navigate to the sub-page, see real models, type a query → list updates,
  toggle sort, "Load more" appends. "Showing N" (no total).
- **E2E (new spec):** `test.step('browse + search local models')` — load page, assert rows render,
  search narrows, sort toggles, load-more appends. **RTL:** hook builds correct query string
  (`q`, `sort`, `order`, `limit`, `cursor`; `q` set ⇒ no cursor); MSW asserts Bearer header.
- **Commit:** "feat(ui-v2): Explore·Local Models — search-only discovery list".

### Phase 2 — Filters, in batches

Add the sidebar facets (block 4) **a few groups at a time**, verifying each batch in Chrome and
extending the SAME E2E spec with new `test.step`s. Suggested batches:
- **2a:** Browse (Trending/New) + Specialisation + Task (`pipeline_tag`).
- **2b:** Tag + Language + License (repeatable → repeated query keys).
- **2c:** Publisher free-text (`author`, multi-chip) + active-filter chip row + clear-all.
- Each batch: **verify in Chrome** (facet → list changes), add a `test.step('filter by <group>')`
  to the spec, RTL for the new params. **Commit per batch** (or per phase 2 if small).
- Assert the exact param mapping (esp. `author` **not** `namespace`; repeatables as repeated
  keys; specialisation/tag AND vs language/license OR per the contract).

### Phase 3 — Detail rail (model detail API)

Add selection → `useModelDetail` (block 2) + the detail rail with Overview + Download-options
tabs (block 6) and MSW single-model handler (block 9, detail handler). **No README tab, no fit
pills.**
- **Verify in Chrome:** select a row → rail opens, Overview specs populate from the detail fetch
  (context/architecture that were null on the row now appear), Download-options lists real
  `quants[]` with sizes + a Recommended badge.
- **E2E:** `test.step('open model detail + quants')` — select row, assert rail header, Overview
  specs, quant rows. **RTL:** detail hook query-key/enabled gating; quants render incl. null-size
  "—".
- **Commit:** "feat(ui-v2): Local Models discovery — detail rail + quants".

### Phase 4 — Pull wiring + MultiTenant gating

Wire per-quant Pull + footer Pull-recommended via `usePullModel` + progress poll (block 7); hide
Pull in MultiTenant.
- **Verify in Chrome (the riskiest seam):** Pull a quant → **real download starts and
  progresses** — this validates the quant-name→filename mapping. Confirm Pull is hidden when
  `deployment === 'multi_tenant'`.
- **E2E:** `test.step('pull a quant')` (where a real/stubbed Pull target exists) + a step
  asserting MultiTenant hides Pull. **RTL:** mutation fires `{ repo, filename }`; MultiTenant
  branch.
- **Commit:** "feat(ui-v2): Local Models discovery — Pull wiring + MultiTenant gate".

### Phase 5 — Polish + docs reconcile + final GATE B

Error states (401/404/422 via `ReferenceApiError`), empty state, a11y/responsive pass, console-
clean. Reconcile the contract docs (block 10) + update the tracker.
- **Final GATE B (live):** full walk against the real dev API — browse/sort/search/all facets,
  open a model, real Pull + progress, light+dark+responsive, console-clean.
- **E2E:** add error/empty `test.step`s to the same spec. **Commit:** "feat(ui-v2): Local Models
  discovery — polish + docs".

> If a phase turns out larger than expected, split it further rather than batching — keep each
> commit a verified, working slice.

## Out of scope (tracked, not built here)

- **Host-fit / "will it run?"** — needs a backend host-memory endpoint (separate workstream).
- **README tab** — needs the API to return README.
- **Capability / size-GB / context / quant-bits / quant-method / Staff-Picks facets** — need API
  filter support (and a taxonomy endpoint to avoid hardcoding).
- **Publisher autocomplete** — needs `/api/v1/orgs`.
- `params_min`/`params_max` UI — page-local only; misleading as a filter today.
- These feed later batches (3-4 New Local Model reuses the `quants[]` contract; 3-7 API Models
  discovery reuses the reference-API consumer shape).
