# Kick-off — Cloudflare reference API (`api.getbodhi.app` model catalog)

> Bootstrap brief for a **new repo** implementing the model-catalog reference API on Cloudflare. This
> states **what to build and the constraints that matter** — it deliberately does **not** prescribe
> the framework, file layout, or a step-by-step build order. Researching the current Cloudflare
> platform + the HuggingFace API and **recommending** the design is your job; where this brief names a
> candidate library or approach, treat it as a hint to evaluate, not a mandate.
>
> **Read first:** the HTTP **contract** — the source of truth for every request/response shape:
> `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/docs/claude-plans/202606/screen-v2/models-local-discovery-reference-api.md`
> (sits next to this prompt file). The contract is still developing; if you find a more
> convention-compliant or cleaner shape, propose the change **in that doc first**, then build to it
> (upstream is unbuilt, so the contract leads).
>
> ## Reference files (absolute paths — all on this machine, readable)
> | File | Use |
> |---|---|
> | `…/BodhiApp/docs/claude-plans/202606/screen-v2/models-local-discovery-reference-api.md` | **The contract** (read first; the conformance target). Full path above. |
> | `…/BodhiApp/docs/claude-plans/202606/screen-v2/explore-local-models-feature-enhancement-brief.md` | The UI design brief (what the frontend renders + which fields feed it). |
> | `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/spike-api-getbodhi-app` | **Data-feasibility spike only** — mine for *what HF data exists*, NOT for API/architecture (see below). |
> | `/Users/amir36/Documents/workspace/src/github.com/anagri/cf-exps/cf-exp-vite-react-hono` | **Reference architecture** for the CF-services-behind-a-mockable-interface + the two-layer test setup (see below). |
>
> (`…/BodhiApp/` = `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/`.)

## What you are building (functional)

An HTTP API serving BodhiApp's **Models discovery** screens a **catalog of downloadable models** the
user can pull and run locally. For v1 that means **GGUF models from HuggingFace** — so the service is,
openly, **an enriching proxy in front of the HuggingFace API**: it adds the fields the UI needs
(parsed quant table, derived capabilities/specialisation, normalized facet vocabularies) and trims
what it doesn't. BodhiApp's frontend calls this API **directly from the browser**, sending the user's
Keycloak OIDC `id_token` as `Authorization: Bearer`.

Functional requirements (the contract has the exact shapes):
1. **List/search/filter/sort** models — `GET /api/v1/models` with `type`/`source` (v1: `gguf`/
   `huggingface`; other values ⇒ 422), facet filters (capability, specialisation, **`quant_bits`** +
   **`quant_method`** [two separate axes], **size range** `size_min_bytes`/`size_max_bytes`, **context
   range** `max_context`, license, language, tags, **`namespace`/org (repeatable, free-text, OR across
   multiple orgs)**, **`curated`**),
   free-text **search** (typo-tolerant), keyset pagination, and sorts = `downloads | likes |
   last_modified | trending | created_at` (**no benchmark/score sort** — no source).
2. **Single model** — `GET /api/v1/models/{source}/{namespace}/{repo}`, returning the model + a parsed
   **`quants:[{name,size,bits?,recommended?}]`** table, with **additive `?include=files,readme`**
   expansion (same object, plus the requested sections — no separate sub-paths).
3. **Taxonomy** — `GET /api/v1/taxonomy` enumerating facet values so the UI never hardcodes them.
4. **Orgs autocomplete** — `GET /api/v1/orgs?q=<query>` returning `{ id, label, model_count, verified }`
   to populate the publisher filter (which also accepts free text; orgs are NOT a closed enum).
5. **Auth** — validate the `id_token` (Keycloak JWKS; `iss`/`exp`/signature; `aud` gated on a config
   binding, off until this API is a registered client). **Public read-through** (anonymous reads
   allowed; invalid token ⇒ 401). **No rate limiting in v1** (future requirement — do not build it,
   but don't design something that precludes adding a per-`sub` limiter later).
6. **CORS: allow all origins (`*`)** — BodhiApp installs anywhere (desktop, self-hosted, PWA); there
   is no fixed origin.
7. **Enrichment the proxy must compute** — GGUF **quant parsing** from filenames, **split into two
   orthogonal axes** (do NOT merge them — a label like `Q4_K_M` is bit-width `4` + method `K_M`): the
   per-quant `quants:[{name,size,bits?,method?,recommended?}]` (detail; `recommended` = suggested
   default, ≤1/repo) and the projected `quant_bits[]` + `quant_methods[]` + `quant_count` +
   **`max_quant_size_bytes`** + **`total_size_bytes`** (on the **list** item, so a row renders
   "N quants · up to <size>" with no per-row fetch; `total_size_bytes` powers the **size range**
   filter `size_min_bytes`/`size_max_bytes`, not buckets); **`architecture`** (from HF
   `gguf.architecture`/card data); **`capabilities`** +
   **`specialisation`** (v1 rules: `small` params<2B, `long-ctx` ctx>64k, `vision` image-text pipeline,
   `coding`/`reasoning` by tag/name; `agentic`/classifier deferred) — **repo-level, NOT per-quant**
   (quantization changes precision, not capability); **`trending_score`** (HF `trendingScore` field —
   ingest + sort on it yourself; HF rejects it as a sort key) + **`created_at`** (HF `createdAt`);
   **`curated`** (your editorial flag) + **`owner_verified`** (a **self-curated org allow-list** — HF
   exposes no verification signal). **No hardware-fit field** (that's BodhiApp-local: host RAM/VRAM).
   - **DO NOT build any human-eval / benchmark / "HUMAN score" field, column, sort, or ingestion.**
     No source exists (HF API verified — no `model-index`/leaderboard data). It is **parked**; omit it
     entirely from this API.

## Data feasibility reference (NOT an architecture to copy)

`/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/spike-api-getbodhi-app` is a
**throw-away spike** that already proved out *what HF data
is available and reachable* (the model list/detail/file-tree/README endpoints, the `gguf` metadata
field, that quant labels live in filenames, that `trendingScore`/`createdAt` exist). **Use it only to
shortcut that data research.** Do **not** adopt its API schema, endpoint design, storage model, or
job architecture — this is a clean-room build against the contract + your own Cloudflare research.
Independently verify HF's **current** API (params, fields, filenaming) — don't trust the spike's
regex or field assumptions; confirm against live HF repos and current HF API docs.

**HF API facts verified live (2026-06-21) — build to these:**
- **List** `GET https://huggingface.co/api/models?filter=gguf&pipeline_tag=text-generation` accepts
  `sort=downloads|likes|lastModified|createdAt`, `order=asc|desc`, `author=<org>`, `search=<q>`,
  `limit`. List items carry `id, downloads, likes, trendingScore, createdAt, tags, pipeline_tag,
  library_name` (NOT `gguf`/`siblings`).
- **`sort=trending`/`sort=trendingScore` are REJECTED (400).** `trendingScore` is a *field* only →
  ingest it and sort on your own column.
- **Detail** `GET /api/models/{id}` carries `downloads, likes, createdAt, lastModified, tags, cardData,
  siblings, gguf{total(bytes), architecture, context_length}, author, gated`. → `architecture` =
  `gguf.architecture`; `context_max` = `gguf.context_length`. **`params_b` is NOT in `gguf`** (only
  `gguf.total` bytes) — derive from `safetensors.parameters.<dtype>` if present, else a repo-name regex.
- **File tree** `GET /api/models/{id}/tree/main?recursive=true&expand=true` → `{path, type, size,
  oid, lfs:{oid,size}}`. Quant labels are in filenames; real repos (e.g. `bartowski/*-GGUF`) show
  `Q2_K, Q3_K_{S,M,L,XL}, Q4_0, Q4_K_{S,M,L}, Q5_K_{S,M,L}, Q6_K{,_L}, Q8_0, IQ2_{XXS,XS,S,M},
  IQ3_{XS,M}, IQ4_XS` — confirms the bits+method split and that **`K_L` is real**.
- **README** = `GET https://huggingface.co/{id}/raw/main/README.md` (raw markdown).
- **`owner_verified`** has **no HF source** → maintain a self-curated org allow-list.
- **Human-eval/benchmark score** has **no HF source** (no `model-index` data, no Leaderboard API) →
  not built (parked).

## Cloudflare design — research-and-recommend, with these fixed constraints

**Research the current Cloudflare platform before deciding** (the docs move fast — prefer the
Cloudflare docs MCP / current docs over prior knowledge). Then recommend the stack. Hints to
evaluate, not mandates:

- **API layer** — needs **type-safe routes that also generate the OpenAPI doc** from the route
  definitions (so the emitted spec stays in lockstep with the handlers). A reasonable candidate is
  **Hono + `@hono/zod-openapi`**, but you may recommend an alternative if it serves type-safety +
  OpenAPI-from-routes better on Workers.
- **Storage + search** — the functional need is catalog rows + facet filtering + typo-tolerant search
  + the parsed quants. **Start with D1 (SQLite) + D1 FTS5** for search (use **Drizzle** to manage the
  D1 schema/migrations if you go D1). FTS5 is the *starting* choice — **design so the search provider
  can be swapped** if/when search grows beyond what FTS5 does well (see the ingest seam below). Decide
  cache-vs-live-proxy (e.g. KV for hot responses, on-demand HF fetch for cold repos) **after**
  researching CF limits + the read pattern; the contract is agnostic to it.
- **Ingestion** — keeping the HF catalog fresh (an active-set refresh + on-demand cold fetch) suits
  Cloudflare **Workflows** (durable, retriable) triggered by **Cron**. Recommend the concrete shape
  after reviewing the Workflows/Cron docs + limits.

### Required: the ingest seam (so search is swappable)

This is a **fixed architectural requirement**, not a hint. Catalog writes flow through a **decoupled
ingest pipeline**: producers **enqueue** change events (Cloudflare **Queue**), a **consumer worker**
ingests them into the **D1-FTS** store. Because routes read through an abstracted store interface and
ingestion is queue-driven, **swapping the search/storage provider later means replacing one
implementation**, not touching the API routes. Consider whether the FTS store warrants its own
worker/D1 to keep that seam clean.

### Required: Cloudflare services behind a mockable interface

Also a **fixed requirement**. **All access to Cloudflare bindings** (Workflows, Queues, D1, KV) sits
**behind a service interface** with a **real** implementation and a **fake** implementation selected
by an env var — so routes, the queue consumer, and tests never touch a binding directly, and the fake
replaces the whole surface for tests that must not hit async CF services.

**Reference example — `/Users/amir36/Documents/workspace/src/github.com/anagri/cf-exps/cf-exp-vite-react-hono`**
(study and mirror this pattern):
- `worker/services/Service.ts` defines a `Service` interface — *"the single boundary to Cloudflare
  bindings. Routes, the queue consumer, and the UI all go through it; the fake variant replaces the
  whole surface."* `getService(env)` returns `new FakeService()` when `env.USE_FAKE_JOBS === "true"`,
  else `new RealService(env)`.
- `worker/services/{real,fake}/…` hold the paired implementations; `worker/services/DigestJobs.ts`
  shows a sub-component contract where *"ALL access to env.DIGEST / env.DIGEST_QUEUE happens behind
  this interface … the fake replaces it wholesale for deterministic UI e2e."*
- `worker/index.ts` shows the queue consumer going through `getService(env).…` so the binding stays
  encapsulated.

## Testing — test-driven, real Cloudflare services (the big focus)

The headline expectation: **TDD with broad e2e coverage running against *actual* Cloudflare services
via the Workers vitest pool.** Mirror the reference example's two-layer split:

- **Workers-pool layer** (`@cloudflare/vitest-pool-workers`, `cloudflare:test`) — runs **against real
  CF services** (real D1, real Queue, real Workflow). The reference uses `introspectWorkflowInstance` +
  `disableSleeps()` / `mockStepResult()` to drive workflows deterministically while still exercising
  the real runtime. This is where workflow/queue/D1 logic is proven. (See the `cloudflare-workflow-e2e`
  skill for the introspection patterns.) The reference's `vitest.cf-config.ts` wires the
  `cloudflareTest` plugin to `wrangler.jsonc`; tests live in `e2e-cf/`.
- **UI/HTTP e2e layer** (Playwright) — the reference drives a real `wrangler dev` started in
  `globalSetup` with **`--var USE_FAKE_JOBS:true`** (the fake service, so no async CF work runs), then
  exercises it via the Playwright library (`vitest.config.ts`, tests in `e2e/`).

**Deviation to explore for *this* project:** where the reference uses the fake-service switch for the
Playwright layer, **investigate running Playwright against Miniflare with the *real* Cloudflare
services** instead (so the HTTP/e2e layer also exercises real D1/Queue/Workflow), and recommend
whether that's viable here or whether the fake-switch path is the pragmatic choice. State the tradeoff.

Every endpoint and enrichment rule should have tests; the contract (the
`models-local-discovery-reference-api.md` path above) is the conformance target the BodhiApp MSW stub
also mirrors.

## Coordination with BodhiApp

- BodhiApp's **Batch 3-6 Phase 2** builds the discovery UI against an **MSW stub of the contract**, so
  the two repos proceed in parallel and the UI is validated before this API deploys.
- **The contract is the single source of truth and is still fluid.** If implementation reveals a
  better/more-compliant shape (or something HF can't actually provide), change the contract file (path
  above) first; the MSW stub and any BodhiApp wiring follow the contract. Keep the API's emitted
  OpenAPI doc aligned with it.
- Auth specifics (issuer, JWKS, the eventual registered-client `aud`) come from BodhiApp's Keycloak
  realm `bodhi`; coordinate the `aud`-enforcement switch when this API is registered as a client.
