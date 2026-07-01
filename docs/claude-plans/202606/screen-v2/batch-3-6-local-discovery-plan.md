# Plan — Batch 3-6: Local Models Discovery (reference-API two-phase)

> Supersedes the assumptions in `batch-3-6-local-discovery-kickoff.md`. That kickoff was written
> as if the reference API had to be spec'd from scratch and handed to an API team. **It already
> exists** and already serves a GGUF model catalog. This plan is the result of cross-repo research
> (two research workflows over `BodhiApp` + `spike-api-getbodhi-app` + the HuggingFace API) and the user's
> decisions on score / quant-table ownership / auth.

## Context — why this, why now, why before 3-4

We are migrating the Models screens to V2. The **Local Models discovery** view (Explore · Local
Models) is a catalog of downloadable GGUF models with facets + a detail panel showing a
**quantization table** and a **Pull** action. It is powered by an **external reference API**
(`api.getbodhi.app`) the frontend calls directly with the user's OAuth `id_token`.

We start 3-6 **before** 3-4 (New Local Model form) deliberately: the discovery flow needs the
quant-file selection data (which quants exist for a repo + their sizes). Building that API surface
first lets 3-4's "select the quant file to download" component render against a real, stable
contract instead of being blocked on it. The design principle the user set: **let the reference API
own the complexity** (HF instability, caching, quant parsing, trending) and expose a **stable,
simplified contract** to BodhiApp.

### What research established (the reframe)

The spike `…/BodhiSearch/spike-api-getbodhi-app` (Hono + Postgres) **proved the data is reachable**:
HuggingFace exposes everything the discovery UI needs — a filterable model list, per-model detail, a
file tree (quant labels live in filenames), README, and `trendingScore`/`createdAt`. **The spike is a
throw-away data-feasibility probe; its API schema and architecture are NOT reused.** The production
reference API is designed fresh against our own contract (below) and rebuilt on Cloudflare.

The prototype's implied features, with the user's resolutions:

| Prototype feature | Reality | Resolution (user-decided) |
|---|---|---|
| "HUMAN" 0–100 score + sortable Score column | No source — HF has no benchmark | **Drop the fake score.** Sort by downloads; designs are indicative only |
| Quant table `{name,size}` per repo | HF file tree gives paths+sizes; quant label is in the filename | **Backend parses & returns** `quants:[{name,size}]` |
| Auth | needs CORS + identity | **CORS `*`** (installed anywhere) + **validate id_token** (Keycloak JWT). **No rate-limit in v1** |
| Specialisation / Quant / Trending facets | Derivable from HF data | Backable: quant **bit-width + method** (two axes parsed from filenames), trending/new (HF `trendingScore`/`createdAt`), specialisation (v1 rules) |
| `type` vs `source` | conflated in the prototype | **Orthogonal axes:** `type`=format (gguf), `source`=registry (huggingface); don't conflate |

## Phase 1 — Reference API: contract + kickoff for a NEW Cloudflare repo

**Pivot (2026-06-20):** the production reference API is a **new Cloudflare repo**; the spike is data-
feasibility-only. Phase 1's deliverable from BodhiApp's side is therefore **not code** but two authoring
artifacts that let the new repo be bootstrapped — **both rewritten 2026-06-21** per the user's
architecture/contract feedback:

1. **The API contract** — [`models-local-discovery-reference-api.md`](./models-local-discovery-reference-api.md)
   (this folder): the HTTP contract (endpoints, request/response, auth, facets, errors). **The source
   of truth** the BodhiApp UI codes against and the MSW stub mirrors. **Still fluid** — implementation
   may propose changes there (upstream is unbuilt, so the contract leads).
2. **The kickoff brief** — [`batch-3-6-cloudflare-reference-api-kickoff.md`](./batch-3-6-cloudflare-reference-api-kickoff.md)
   (this folder): **functional requirements + fixed constraints**, deliberately *not* a prescriptive
   architecture — the implementing agent researches Cloudflare + HF and **recommends** the stack.

See those two docs for the authoritative detail. In brief, the contract establishes:

- **Openly a HuggingFace proxy** (enrich + trim), not a hidden facade. v1 scope: `type=gguf`,
  `source=huggingface` only (other values ⇒ 422). `type` (format) and `source` (registry) are
  orthogonal, both filterable.
- **Endpoints** (all under `/api/v1`): `GET /api/v1/models` (list/search/filter/sort, keyset
  pagination), `GET /api/v1/models/{source}/{namespace}/{repo}` (single model + parsed `quants[]`,
  with **additive `?include=files,readme`**), `GET /api/v1/taxonomy` (facet enum values).
- **Enrichment**: parsed `quants:[{name,size,bits,method}]` + projected `quant_bits[]`/
  `quant_methods[]`/`quant_count` (bit-width & method are **two separate facets**), `total_size_bytes`
  (powers a **size range** slider, not buckets), derived `capabilities[]` + `specialisation[]` (v1
  rules; classifier deferred), `trending_score` + `created_at`. **No score / no benchmark sort.**
- **Auth**: validate the Keycloak `id_token` (JWKS; `iss`/`exp`/sig; `aud` gated on a config binding,
  off until registered); **public read-through**, invalid ⇒ 401. **CORS `*`. No rate-limit in v1.**

**Fixed Cloudflare constraints** (in the kickoff; everything else is the agent's call after research):
a **queue → consumer → D1-FTS** ingest seam so the search provider is swappable; **all CF bindings
behind a mockable service interface** (real/fake switch); **TDD with real CF services via
`@cloudflare/vitest-pool-workers`** + a Playwright HTTP layer. Reference pattern quoted from
`…/anagri/cf-exps/cf-exp-vite-react-hono`. Candidates *hinted* (Hono + `@hono/zod-openapi` for
type-safe-routes-with-OpenAPI; Drizzle for D1 migrations; D1-FTS5 as the starting search) but the
agent confirms/recommends.

**Gate (Phase 1, BodhiApp side):** the contract + kickoff are reviewed and approved; the Phase-2 MSW
stub matches the contract (so it's exercised before the Cloudflare repo exists). The new repo is built
**separately** against the kickoff; the migration is never blocked on it.

## Phase 2 — BodhiApp UI (`crates/bodhi`): the Local Models discovery view

Reuses the shipped Batch 3-1 "My Models" full-stack recipe wholesale. All the reference-API
scaffolding already exists (Batch-0): `src/lib/referenceApiClient.ts` (`createReferenceApiClient`,
sends `Authorization: Bearer <id_token>`) + `src/hooks/reference/useReferenceApi.ts` (composes base
URL from `AppInfo.reference_api_url` + id_token from `useGetUser`). The MSW info stub already returns
`reference_api_url`.

**Work items:**

1. **Reference-API discovery hooks** — new `src/hooks/reference/useDiscoverModels.ts` (+
   `useModelDetail`) wrapping `useReferenceApi()`: `GET /api/v1/models` (list),
   `GET /api/v1/models/{source}/{namespace}/{repo}` (single + `?include=files` when the Quants tab
   opens). Gate on non-null client (`enabled: !!client`), use `keepPreviousData` (3-1 carry-forward:
   no empty-flash on facet/page change), own query-key factory. A `useDiscoverTaxonomy()` hook reads
   `/api/v1/taxonomy` for facet chip values.

2. **Local Models view** — new route under `src/routes/models/` (e.g. `discover/local/`), cloning
   `routes/models/-components/ModelsScreenV2.tsx`: `useShellChrome({ breadcrumb, sidebar, rail })`
   (ONE publisher per screen — 3-4 carry-forward), `useListKeyNav()`, `ModelSidebarFacets` +
   `ModelDetailRail` as templates. Sidebar published memoized on filter state.

3. **Facets (taxonomy-driven, not hardcoded)** — Capability, License, **Quant bit-width**, **Quant
   method** (two separate chip groups), Specialisation enumerate from `/api/v1/taxonomy`; **Size** is a
   **two-thumb GB range** (`size_min_bytes`/`size_max_bytes`) and **Context** a **max-only** slider
   (`max_context`) — both derive bounds/step from the result set, no buckets. The list always sends
   `type=gguf&source=huggingface`. Browse = New (`sort=created_at`) / Trending (`sort=trending`).
   Active-filter chip row + clear-all mirror the prototype.

4. **List** — repository rows (`namespace/repo`, meta line, tag chips), columns = **Downloads + Likes**
   (sortable; default sort Downloads), keyset "Load more" pagination (raise `limit` when `q` is set —
   cursor is disabled during search). Search box = `q` (placeholder "Search HuggingFace repos"),
   submit-on-Enter + clear-resets (3-1 pattern).
   - **PARKED — "Human Evals" column NOT implemented (frontend or API).** The design mock shows a HUMAN
     score column + default sort, but **no upstream source exists** (HF API has no per-model benchmark/
     human-eval; verified live 2026-06-21). **Do not build** the column, its data, or any `score`/
     `human_eval` field/sort in this batch — render the list without it. Revisit only if a real benchmark
     source (e.g. a leaderboard/Arena ingestion) is later greenlit as its own workstream.

5. **Detail rail** — Overview (date=`created_at`, downloads, likes, capability badges, specs:
   Context=`context_max`, **Architecture=`architecture`**, License=`license`) + **Quants tab rendering
   `quants:[{name,size}]` straight from the single-model response** (no client-side filename parsing —
   the API owns it; the Quants tab fetches with `?include=files` if the raw file list is also shown).
   Per-quant Pull + footer Pull-best-quant / Add-to-Bodhi. **Row meta** ("N quants · up to <size> ·
   <license>") composes from `quant_count` + `max_quant_size_bytes` + `license` — all on the
   list item, no per-row fetch. **Rank `#` is the client list ordinal** (not a server field).

6. **Pull wiring (the bridge into 3-4/3-5)** — reuse `usePullModel()`
   (`src/hooks/models/useDownloads.ts`) → `POST /bodhi/v1/models/files/pull` `{ repo, filename }`;
   poll `useListDownloads()` (1 s) for progress. `repo = namespace/repo`, `filename` from the chosen
   quant. **Hide Pull in MultiTenant** (HubService rejects download in multi-tenant; gate on
   deployment mode / `power_user` scope) — show catalog browse-only there.

7. **Flag + nav** — add `'local-models-discover'` to the `UiV2Screen` union (`src/lib/uiV2Flags.ts`);
   add the sidebar MODEL-TYPE sub-page in `src/components/shell/shell-nav-config.tsx`.

8. **MSW external-origin stub** — handlers for `https://api.getbodhi.app/` `GET /api/v1/models`,
   `GET /api/v1/models/huggingface/{ns}/{repo}` (incl. `quants`, and `files`/`readme` under
   `?include=`), `GET /api/v1/taxonomy` via the `typedHttp` setup; assert the `Authorization: Bearer`
   header is present (id_token from the logged-in-user mock). This keeps the batch self-sufficient —
   **the migration is never blocked on the real reference API.** If the contract shifts during the
   Cloudflare build, update the stub to follow the contract.

**E2E reference-API handling (stated explicitly, per `reference-api.md`):** the Playwright suite has
no live reference API in CI. Drive the discovery view against the **MSW external-origin stub** (or a
tagged/gated spec) — never silently skip. Black-box only (no `page.evaluate`/context fetch),
`reducedMotion:'reduce'` for the V2 rail (memory carry-forwards).

**Gate (Phase 2):** RTL for the new view (list, taxonomy-driven facets→query-params, rail Overview +
Quants-from-DTO, Pull→mutation, MultiTenant-hides-Pull, MSW id_token-header assertion); typecheck +
lint clean; E2E spec; **GATE B live** walk (the discovery list renders from the stub and, where
available, the real API; Pull triggers a real download; light + dark + responsive; console-clean).

## Sequencing & dependencies
- **The Cloudflare reference API is built independently** (its own repo, against the kickoff). Phase
  2's MSW stub means BodhiApp work proceeds in parallel and is validated against the stub even before
  the API is deployed — the migration is never blocked on it.
- After 3-6: the `quants:[{name,size}]` contract feeds **3-4** (New Local Model — quant-file selection)
  and **3-5** (files/pull consolidation). Then **3-7** (API Models discovery) reuses the same
  reference-API consumer shape (its catalog source is scoped separately when that batch is planned).

## Decisions taken as defaults (noted, not blocking)
- **Trending = HF `trendingScore`** (real-time), not downloads/day delta. If UX later wants the delta
  signal, that's deferred snapshot-delta infra — out of scope here.
- **Specialisation is its own axis** (not folded into `capabilities`), so the UI shows two facet groups
  and the future classifier has a clean target.
- **`type` vs `source`** kept as orthogonal axes from the start (v1 only `gguf`/`huggingface`), so
  future `safetensor`/`mlx` formats and non-HF registries slot in without a contract break.

## Verification summary (end-to-end)
1. **Phase 1 (BodhiApp side)** = the contract + kickoff docs are reviewed/approved (no spike code runs;
   the API itself is the separate Cloudflare repo, verified there against its own TDD/`vitest-pool-
   workers` + Playwright gates per the kickoff).
2. **Phase 2**: `cd crates/bodhi && npm test` (RTL incl. MSW id_token-header assertion); `make
   build.ts-client` if any BodhiApp backend type changed (none expected — Pull endpoint already
   exists); `make test.e2e` for the discovery spec; **GATE B**: `make app.run.live`, toggle the
   `local-models-discover` flag, browse the catalog, open a model, Pull a quant, watch progress.
