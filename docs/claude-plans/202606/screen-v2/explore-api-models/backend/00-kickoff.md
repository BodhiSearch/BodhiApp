# Explore API Models — Backend Kickoff (Plan of Record)

**Read this first.** This is the index and plan-of-record for building the
models.dev + Portkey catalog feature inside the `api-getbodhi-app` Cloudflare
Worker. The detail lives in the companion docs linked below; this page is
navigational.

## Goal

Bring the models.dev catalog (provider/model/pricing data) *inside* Bodhi so the
new Explore experience renders two pages — a **Providers** browse page and a
**Models** search/facet page — entirely from our own backend instead of hitting
a third party at request time. This backend **ingests** the models.dev dataset
(enriched with Portkey provider metadata) into D1 on a schedule, stores it in
first-class relational + FTS5 tables, and **serves** it through a small read API
plus a configure-bridge payload the Bodhi app uses to wire a provider. The
existing HuggingFace GGUF proxy stays as-is; this is an additive, parallel
catalog surface.

## Companion docs

- [`../data-analysis.md`](../data-analysis.md) — the data contract: the shape of
  the models.dev and Portkey sources, the merged catalog model, field-level
  provenance, and **§9 the risk register** (the early decisions called out
  below). Start here for "what does the data actually look like".
- [`../design-prompt.md`](../design-prompt.md) — the UI being fed by this
  backend (the two Explore pages, their facets, sort, and the
  configure-from-catalog action). Read to understand the consumer.
- [`01-d1-schema.md`](01-d1-schema.md) — drizzle schema (`sync_state`,
  `slug_alias`, `providers`, `models`, `model_pricing`) for migration `0001`,
  hand-authored raw-SQL `0002` (FTS5 external-content `models_fts` + 3 sync
  triggers + facet indexes), column→source mapping with models.dev-wins conflict
  rules, and D1-limits notes.
- [`02-ingestion-pipeline.md`](02-ingestion-pipeline.md) — diff-since-last-sync
  ingestion design (GitHub compare-API + raw-fetch + SHA cursor, Workflow/Cron
  wrangler deltas, 10-step `IngestWorkflow` pseudocode, idempotency/resume,
  slug-alias seed, Real/Fake + vitest-pool-workers test strategy).
- [`03-catalog-api-contract.md`](03-catalog-api-contract.md) — all 5 endpoints
  with zod-openapi request/response schemas, facet-count + numbered-pager shapes,
  the configure-bridge payload + provider→(api_format, base_url) mapping, the
  exact `@bodhiapp/reference-api-types` additions, and the KV→D1 cache-aside plan.
- [`../samples/`](../samples/) — raw models.dev + Portkey sample files captured
  for reference. Use these as the fixtures for Phase 1 seed and as the
  ground-truth inputs for transform tests.

## Current state of `api-getbodhi-app` (honest summary)

`api-getbodhi-app` is today a **v1 live HuggingFace GGUF proxy**: a single
Cloudflare Worker (pnpm + Turborepo monorepo) serving a React SPA under `/ui/*`
and a Hono + zod-openapi API under `/api/*`. The list endpoint is a stateless
passthrough to HuggingFace; the single-model endpoint is cache-aside over a
generic key/value blob table (`model_cache`) backed by KV (hot) + D1 (warm).
Only **two** Cloudflare bindings exist — `DB` (D1) and `KV` — and the
architecture is well-seamed for evolution (a `getService(env)` Real/Fake DI
factory keyed on `USE_FAKE_JOBS`, a typed `Catalog` interface, a reusable
two-tier `CacheStore`, pure HF transform functions, and a published types-only
package `@bodhiapp/reference-api-types` that is the single source of truth, wired
to the zod schemas via `satisfies` guards). **The headline gap:** there are no
catalog/domain tables (`providers`/`models`/`model_pricing`), no FTS5, no
ingestion/git-diff pipeline, and **no Workflows, no Queues, no Cron triggers, no
R2** (verified verbatim in `dist/api_getbodhi_app/wrangler.json`). The single D1
migration (`0000`) is the cache table only, and the cf test pool does **not**
auto-apply migrations (tests must `CREATE TABLE` in `beforeEach`).

## Phased implementation plan (thin vertical slices, commit per phase)

Each phase is an independently verifiable slice. Build it, prove it in a test +
a demo, then commit before moving on.

### Phase 1 — Schema + seed-from-fixture + read paths (fixtures only, no live ingest)

- **Scope:** Land migration `0001` (drizzle: `sync_state`, `slug_alias`,
  `providers`, `models`, `model_pricing`) and `0002` (raw-SQL FTS5 + triggers +
  indexes). Add a one-shot seed that loads the `../samples/` fixtures into D1.
  Add `GET /api/v1/providers` and `GET /api/v1/models` (catalog) read paths that
  query D1 directly. No models.dev fetch, no cron — fixtures are the source.
- **Files:** `worker/db/schema.ts`, `drizzle/0001_*.sql`, `drizzle/0002_fts.sql`,
  new `worker/catalog/` (read service behind the `Service`/`Catalog` seam, with a
  Fake twin), `worker/routes/providers.list.ts`, `worker/routes/catalog.models.list.ts`,
  `worker/schemas/catalog.ts`, fixtures wired from `../samples/`.
- **Test (vitest / `@cloudflare/vitest-pool-workers`):** `CREATE TABLE` the new
  schema in `beforeEach`, seed from fixtures, assert list/filter/sort/pagination
  rows come back correctly. Fake-service unit tests for deterministic shape.
- **Demo:** `wrangler dev` → `curl /api/v1/providers` and `/api/v1/models`
  return seeded rows; the two Explore pages render against local data.

### Phase 2 — models.dev ingest via GitHub compare-API + cron

- **Scope:** Add the `workflows` binding + `IngestWorkflow` (WorkflowEntrypoint),
  `triggers.crons`, and a `scheduled` handler. Implement diff-since-last-sync:
  read SHA cursor from `sync_state`, GitHub compare-API to find changed files,
  raw-fetch them, transform, models.dev-wins upsert into `providers`/`models`/
  `model_pricing`. Idempotent/resumable per `02-ingestion-pipeline.md`.
- **Files:** `wrangler.jsonc` (both envs — bindings are NOT inherited),
  `worker/index.ts` (add `scheduled`/workflow export), `worker/ingest/IngestWorkflow.ts`,
  `worker/ingest/modelsdev.ts` (source client + transform), `worker/ingest/sync_state.ts`,
  Real/Fake ingest seam (`env.WORKFLOW` fake under `USE_FAKE_JOBS`).
- **Test:** vitest-pool-workers with `introspectWorkflowInstance`
  (`disableSleeps`/`mockStepResult`) per the `cloudflare-workflow-e2e` skill;
  Real/Fake source client; assert first-run full ingest then no-op on unchanged SHA.
- **Demo:** trigger the workflow locally; D1 row counts grow; re-run is a no-op.

### Phase 3 — Portkey enrichment join

- **Scope:** Add Portkey provider metadata as a second source; join/enrich
  `providers` (and any model-level fields) with models.dev-wins conflict rules.
- **Files:** `worker/ingest/portkey.ts`, extend `IngestWorkflow` with the
  enrichment step, extend transform + provenance handling.
- **Test:** transform/merge unit tests over `../samples/` Portkey fixtures
  asserting the documented conflict resolution; pool test for the joined upsert.
- **Demo:** providers show Portkey-sourced fields where models.dev is silent.

### Phase 4 — FTS5 search + facets

- **Scope:** Wire `models_fts` MATCH search into `GET /api/v1/models` (`q`
  param) and implement facet counts (provider/modality/etc.) + the numbered pager.
- **Files:** `worker/catalog/` query layer (FTS5 MATCH + facet aggregation),
  `worker/schemas/catalog.ts` (facet-count + pager response shapes).
- **Test:** pool tests asserting MATCH results, facet counts, and pager math
  against seeded data; trigger-sync correctness (insert/update/delete reflected
  in FTS).
- **Demo:** Explore Models page search box + facet chips filter live against D1.

### Phase 5 — configure-bridge payload + types-package publish

- **Scope:** Add the configure-bridge endpoint/payload (provider→`(api_format,
  base_url)` mapping from contract §8) the Bodhi app consumes to wire a provider.
  Add the new types to `@bodhiapp/reference-api-types` and extend the zod
  `satisfies` guards; cut an `api/v*` publish.
- **Files:** `worker/routes/catalog.configure.ts`, `worker/schemas/catalog.ts`
  (guards), `packages/api-types/src/index.ts` (hand-written interfaces), publish
  via `just publish-api-types`.
- **Test:** contract test that the payload matches the published types
  (compile-time guard) + endpoint test for the mapping table.
- **Demo:** Bodhi app configures a provider end-to-end from the catalog.

## Top risks to decide early (from data-analysis §9)

1. **Ingest resolved `api.json` vs raw TOML.** models.dev ships both a built
   `api.json` and per-provider TOML sources. Resolved JSON is simpler to ingest
   but couples us to their build output; raw TOML is canonical but means parsing
   + assembling ourselves. Decide the source-of-record for the diff pipeline.
2. **Slug-alias map.** models.dev, Portkey, and Bodhi may name the same provider
   differently. The `slug_alias` table is the join key — decide the seed list and
   the policy for unmapped slugs (drop vs. surface vs. fail ingest).
3. **No native git in Workers.** There is no `git clone`/diff in workerd, hence
   the GitHub compare-API + raw-fetch + SHA-cursor approach. Confirm this is the
   accepted change-detection mechanism (rate limits, auth token, full-resync
   fallback when the cursor is stale).
4. **Per-provider pricing vs. flat cost.** Pricing can vary per provider for the
   same model (and by token type / tier). `model_pricing` is modeled as a child
   table for this reason — confirm the granularity the UI needs vs. what we store.

## Open decisions for the user

- Source-of-record: resolved `api.json` or raw TOML (risk 1)?
- Cron cadence + whether a manual "force resync" trigger is in scope.
- Should the catalog read paths reuse the existing two-tier `CacheStore` (KV hot
  / D1 warm) as a read accelerator, or query D1 directly and treat KV as optional?
- Generalize the existing GGUF `Catalog` interface/route surface vs. add a
  parallel catalog service (the plan assumes parallel/additive).
- Whether Portkey enrichment (Phase 3) is required for first ship or can land
  after the models.dev-only catalog is live.
