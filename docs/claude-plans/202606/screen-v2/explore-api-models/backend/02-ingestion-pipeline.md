# Backend 02 — Diff-Since-Last-Sync Ingestion Pipeline

**Audience:** the backend engineer building the catalog-ingestion pipeline inside `api-getbodhi-app`.
**Scope:** how the Cloudflare Worker re-ingests only the files that changed in `anomalyco/models.dev` and `Portkey-AI/models` since the last sync, normalizes the union into the D1 catalog (`providers` / `models` / `model_pricing` + `models_fts`), and advances a durable per-source SHA cursor — implementation-ready at the TS-pseudocode level.
**Grounding:** all data shapes are from `../data-analysis.md` §2.5 (base_model resolution), §4 (Portkey parse / cents-per-token / custom_pricing), §6 (pipeline). All Cloudflare mechanics are grounded in the **actual current repo state** — the repo today has **only** `DB` (D1) and `KV` bindings; there are **no** Workflows, Queues, Cron triggers, R2, or FTS5. Every binding this pipeline needs is spelled out as a `wrangler.jsonc` delta in §2.

> Companion docs: `01-*` (catalog schema / migrations) defines the target tables this pipeline writes; `../data-analysis.md` is the format contract. This doc owns **how data gets in**, not the table DDL.

---

## 0. TL;DR

- **No native git in Workers.** Use the **GitHub compare API** (`GET /repos/{owner}/{repo}/compare/{base}...{head}`) to get changed/removed file paths since a stored HEAD SHA, then **raw-fetch only the changed files**. Recommended over an external CI runner — fully self-contained, no second deploy surface. (§1)
- **wrangler.jsonc needs four additions per env** (bindings are NOT inherited across envs in this repo): a `workflows` binding, a `triggers.crons` entry, the source-repo `vars`, and a `GITHUB_TOKEN` secret. (§2)
- **One `IngestWorkflow` class, parameterized by source** (`models.dev` | `portkey`), mirroring `../data-analysis.md` §6.2 step-for-step, each step a durable `step.do(...)` checkpoint. (§3)
- **Cursor advances only on full success** of the parse→upsert→prune→reindex→logos batch; a partial failure re-diffs the same `base..HEAD` range next tick. Upserts are PK-keyed so re-runs are no-ops. (§4)
- **A seeded slug-alias map** reconciles `models.dev` slugs with Portkey slugs (`amazon-bedrock↔bedrock`, `xai↔x-ai`, `google↔google`+`vertex-ai`, …). (§5)
- **Tests reuse the existing `getService(env)` Real/Fake seam** (`USE_FAKE_JOBS`) plus `@cloudflare/vitest-pool-workers` with `introspectWorkflowInstance` for step-level mocking; fixtures are TOML/JSON inputs, assertions are D1 rows. (§6)

---

## 1. No-native-git Constraint and Source-Fetch Strategy

### 1.1 The constraint

Cloudflare Workers and Workflows run in `workerd` — no shell, no filesystem, no `git` binary. "Shallow clone + `git diff`" (the cadence `models.dev` itself uses internally per `../data-analysis.md` §6.2) cannot be executed in-Worker. We need a **diff primitive that the GitHub HTTP API gives us for free** and a **durable cursor** to anchor it.

### 1.2 Two viable shapes

| Shape | How | Pros | Cons |
|---|---|---|---|
| **(a) GitHub compare-API + raw-file fetch + stored HEAD SHA** | Worker/Workflow calls GitHub REST to resolve HEAD, diff `base...head`, then raw-fetches only changed files | Fully self-contained in the Worker; no second deploy surface; reuses the `USE_FAKE_JOBS` seam for tests; cursor lives in D1 next to the data it gates | Subject to GitHub REST rate limits; compare API truncates file lists >300 (handle pagination/fallback) |
| **(b) External CI runner (GitHub Actions) does real `git clone`+`diff`, POSTs normalized rows to an authenticated ingest endpoint** | Actions cron → clone → diff → normalize → `POST /api/internal/ingest` | Real git, no compare-API limits; heavy normalization off the Worker | A second moving part (Actions workflow + secrets + an authenticated write endpoint to harden); ingest logic split across two repos; harder to unit-test the seam |

### 1.3 Recommendation: **(a) GitHub compare-API + raw-file fetch**

Self-contained, testable through the existing service seam, and the cursor (a SHA) is the durable monotonic position the whole pipeline keys on. (b) is the documented fallback if compare-API limits or file-list truncation become painful.

### 1.4 Exact GitHub API calls (authenticated)

All calls send `Authorization: Bearer ${GITHUB_TOKEN}`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`, and a `User-Agent` (GitHub rejects no-UA requests).

1. **Resolve HEAD of the default branch** (Workflow step 2):
   ```
   GET https://api.github.com/repos/{owner}/{repo}/commits/{branch}
   -> body.sha   // the new HEAD SHA to diff toward and (on success) store as cursor
   ```
   `{owner}/{repo}` = `anomalyco/models.dev` or `Portkey-AI/models`; `{branch}` from `vars` (default `main`).

2. **Compare base..head** (Workflow step 3) — the diff primitive:
   ```
   GET https://api.github.com/repos/{owner}/{repo}/compare/{base_sha}...{head_sha}
   -> body.files[] = { filename, status, previous_filename? }
        status ∈ added | modified | removed | renamed | copied | changed | unchanged
   -> body.status, body.ahead_by, body.behind_by
   ```
   - `status: removed` (and the *old* path of a `renamed`) → **prune** that row.
   - `added | modified | renamed(new path) | changed` → **fetch + upsert**.
   - **Truncation guard:** the compare endpoint returns at most **300 files** in `files[]` and paginates the commit list, not the file list. If `body.files.length === 300` (or `body.files` is flagged truncated), treat the diff as **incomplete** and **fall back to a full snapshot ingest** (enumerate the tree, see step 3b) for that tick. Hourly diffs of these repos are far below 300 files in practice (`../data-analysis.md` §6.2 — models.dev emits small per-provider PRs hourly).

3. **First run / truncation fallback — enumerate the tree** (no base cursor yet):
   ```
   GET https://api.github.com/repos/{owner}/{repo}/git/trees/{head_sha}?recursive=1
   -> body.tree[] = { path, type }   // type 'blob'
   -> body.truncated  // if true, fall back to the source's own bulk artifact (see 1.5)
   ```
   Filter `path` to the source's relevant subtree (models.dev: `providers/**/*.toml`, `models/**/*.toml`; portkey: `pricing/*.json`, `general/*.json`).

4. **Raw-fetch a changed file** (Workflow step 4) — pin to the resolved HEAD SHA so the fetch is reproducible within the run:
   ```
   GET https://raw.githubusercontent.com/{owner}/{repo}/{head_sha}/{path}
   ```
   `raw.githubusercontent.com` does **not** count against the REST rate limit and needs no auth for public repos, but send the token anyway for consistency/abuse-detection headroom. Prefer raw over the contents API (`GET /repos/.../contents/{path}`) — raw avoids base64 round-trips and a per-file REST quota hit.

5. **Logos** (Workflow step 9) — not on GitHub raw; served by models.dev:
   ```
   GET https://models.dev/logos/{slug}.svg     // fallback default.svg on 404
   ```

### 1.5 Rate-limit notes (authenticated token)

- Authenticated REST: **5,000 requests/hour** per token (`X-RateLimit-Limit` / `-Remaining` / `-Reset` headers). Hourly cadence × 2 sources × (1 HEAD + 1 compare + N changed-file *raw* fetches). Raw fetches hit `raw.githubusercontent.com`, **not** counted against REST — so a tick costs ~2 REST calls/source plus raw fetches. Comfortably inside budget even on a large diff.
- **First-run cost:** the tree enumeration is 1 REST call; the bulk raw-fetch of every file is on `raw.*` (uncounted). If `git/trees` returns `truncated: true` for a huge tree, prefer the source's own bulk artifact (models.dev: pull the generated `api.json` once via raw; portkey: each provider file is already one JSON) rather than paginating trees.
- **Backoff:** on `403`/`429` with `X-RateLimit-Remaining: 0`, read `X-RateLimit-Reset`, **do not advance the cursor**, and let the next cron tick retry. On `403` with `Retry-After`, honor it. Use Workflow step retry config (`§4`) for transient `5xx`.
- **Token:** a fine-grained PAT with `Contents: read` on the two public repos, or a classic token with `public_repo`. Stored as the `GITHUB_TOKEN` secret (§2). Never log it.

---

## 2. wrangler.jsonc Deltas

**Critical:** in this repo, D1/KV bindings are declared **per-env and are NOT inherited** (dev and prod each repeat the block). Every addition below must be made in **both** the dev and prod env blocks, and the build bakes `CLOUDFLARE_ENV` at build time — deploy env must match build env. The current built deploy config (`dist/api_getbodhi_app/wrangler.json`) shows `"workflows":[]`, `"queues":{producers:[],consumers:[]}`, `"triggers":{}` — all empty. We add to those.

### 2.1 Workflow binding (chosen over Queue)

We use a **Workflow** (durable multi-step with per-step checkpointing) rather than a Queue + scheduled handler, because the pipeline is a linear 10-step durable job per source where each step must checkpoint and resume (§4) — exactly Workflows' sweet spot. (A Queue would force us to hand-roll step state and retries.)

```jsonc
// top-level (shared) OR repeated per env — workflows binding block
"workflows": [
  {
    "name": "ingest-workflow",
    "binding": "INGEST_WORKFLOW",
    "class_name": "IngestWorkflow"
  }
]
```

> If Workflows are not preferred in a given environment, the Queue alternative is: add `"queues": { "producers": [{ "binding": "INGEST_QUEUE", "queue": "ingest-queue" }], "consumers": [{ "queue": "ingest-queue", "max_batch_size": 1 }] }`, export a `queue(batch, env)` handler from `worker/index.ts`, and persist a `step` column in `sync_state` to emulate checkpoints. Workflows are the recommendation.

### 2.2 Cron trigger

```jsonc
// per env (triggers is currently {} in the built config)
"triggers": {
  "crons": ["0 * * * *"]   // hourly, mirrors models.dev sync cadence (data-analysis §6.2)
}
```

The cron fires a single `scheduled(event, env, ctx)` handler (new export from `worker/index.ts`) that creates **one Workflow instance per source**:

```ts
// worker/index.ts — extend the existing `export default { fetch } satisfies ExportedHandler<Env>`
export default {
  fetch: app.fetch,
  async scheduled(_event: ScheduledController, env: Env, ctx: ExecutionContext) {
    if (String(env.USE_FAKE_JOBS) === 'true') return // fake seam: no real ingestion in e2e
    for (const source of ['models.dev', 'portkey'] as const) {
      await env.INGEST_WORKFLOW.create({ id: `ingest-${source}-${_event.scheduledTime}`, params: { source } })
    }
  },
} satisfies ExportedHandler<Env>
```

### 2.3 Vars (source-repo coordinates)

```jsonc
// per env "vars" (alongside existing USE_FAKE_JOBS, KEYCLOAK_ISSUER)
"vars": {
  "USE_FAKE_JOBS": "false",
  "MODELSDEV_REPO": "anomalyco/models.dev",
  "MODELSDEV_BRANCH": "main",
  "PORTKEY_REPO": "Portkey-AI/models",
  "PORTKEY_BRANCH": "main",
  "MODELSDEV_LOGOS_BASE": "https://models.dev/logos"
}
```

### 2.4 Secret

```bash
# per env — NOT in wrangler.jsonc; injected via wrangler secret / .dev.vars (mirrors HUGGINGFACE_TOKEN)
wrangler secret put GITHUB_TOKEN            # prod
wrangler secret put GITHUB_TOKEN --env dev  # dev
# local: add GITHUB_TOKEN=... to .dev.vars
```

Resolve it through the existing `getSettings(env)` indirection (`worker/services/Settings.ts`) — add `githubToken()` next to `huggingfaceToken()`.

### 2.5 Type regen

After editing `wrangler.jsonc`, run `wrangler types` (cf-typegen) so `worker-configuration.d.ts` gains `INGEST_WORKFLOW: Workflow`, `GITHUB_TOKEN: string`, and the new vars on `Env`.

---

## 3. `IngestWorkflow` — Steps as Concrete TS Pseudocode

One class, parameterized by `source`. Each numbered step maps 1:1 to `../data-analysis.md` §6.2 and is wrapped in `step.do(...)` so it checkpoints (§4). Service access goes through the existing seam: `getService(env).ingest()` returns an `Ingestor` with a Real impl (touches `DB`/`KV`/GitHub) and a Fake twin (canned) selected by `USE_FAKE_JOBS`.

```ts
import { WorkflowEntrypoint, WorkflowStep, WorkflowEvent } from 'cloudflare:workers'

type Source = 'models.dev' | 'portkey'
interface IngestParams { source: Source }

export class IngestWorkflow extends WorkflowEntrypoint<Env, IngestParams> {
  async run(event: WorkflowEvent<IngestParams>, step: WorkflowStep) {
    const { source } = event.payload
    const svc = getService(this.env).ingest()          // Real or Fake via USE_FAKE_JOBS
    const repo = source === 'models.dev'
      ? { owner: 'anomalyco', name: 'models.dev', branch: this.env.MODELSDEV_BRANCH }
      : { owner: 'Portkey-AI', name: 'models',     branch: this.env.PORTKEY_BRANCH }

    // ── step 1: read cursor ────────────────────────────────────────────────
    const base = await step.do('read-cursor', async () =>
      svc.getCursor(source)                              // SELECT last_sha FROM sync_state WHERE source=?  (null on first run)
    )

    // ── step 2: resolve HEAD ───────────────────────────────────────────────
    const head = await step.do('resolve-head', { retries: { limit: 5, delay: '10 seconds', backoff: 'exponential' } }, async () =>
      svc.resolveHead(repo)                              // GET /repos/{o}/{r}/commits/{branch} -> .sha
    )
    if (base === head) {
      await step.do('noop-touch', async () => svc.touchSync(source, head, 'unchanged', 0))
      return                                             // nothing changed; idempotent early-out
    }

    // ── step 3: diff base..head ────────────────────────────────────────────
    const diff = await step.do('diff', { retries: { limit: 5, delay: '10 seconds', backoff: 'exponential' } }, async () => {
      if (!base) return svc.fullFileList(repo, head)     // first run: tree?recursive=1, filtered to relevant subtree
      const cmp = await svc.compare(repo, base, head)    // GET /compare/{base}...{head}
      if (cmp.truncated) return svc.fullFileList(repo, head)   // >300 files -> full snapshot fallback (§1.4)
      return cmp                                         // { changed: path[], removed: path[] }
    })

    // ── step 4: fetch changed files (raw, pinned to head sha) ──────────────
    const files = await step.do('fetch-files', { retries: { limit: 5, delay: '10 seconds', backoff: 'exponential' } }, async () =>
      svc.fetchRaw(repo, head, diff.changed)             // [{ path, body }]  via raw.githubusercontent.com/{o}/{r}/{head}/{path}
    )

    // ── step 5: parse + normalize ──────────────────────────────────────────
    const normalized = await step.do('normalize', async () =>
      source === 'models.dev'
        ? svc.normalizeModelsDev(files)                  // see §3.1
        : svc.normalizePortkey(files)                    // see §3.2
    )

    // ── step 6: upsert (models.dev wins on overlap) ────────────────────────
    await step.do('upsert', async () => svc.upsert(normalized))   // see §3.3 conflict policy

    // ── step 7: prune removed rows ─────────────────────────────────────────
    await step.do('prune', async () => svc.prune(source, diff.removed))   // delete rows whose source file was removed

    // ── step 8: reindex FTS for touched models only ────────────────────────
    await step.do('reindex-fts', async () => svc.reindexFts(normalized.touchedModelKeys))

    // ── step 9: logos -> KV (new/changed providers only) ───────────────────
    await step.do('logos', { retries: { limit: 3, delay: '5 seconds', backoff: 'exponential' } }, async () =>
      svc.syncLogos(normalized.touchedProviderSlugs)     // GET models.dev/logos/{slug}.svg (default.svg fallback) -> KV logo:{slug}
    )

    // ── step 10: advance cursor (LAST — only after 5-9 all succeeded) ──────
    await step.do('advance-cursor', async () =>
      svc.advanceCursor(source, head, 'ok', diff.changed.length)   // UPDATE sync_state SET last_sha=head, synced_at=now
    )
  }
}
```

### 3.1 `normalizeModelsDev(files)` — base_model resolution (data-analysis §2.5 + §3)

Two ingestion modes; **recommended hybrid** (data-analysis §9 risk 3): diff raw TOML to know *what* changed, but resolve each touched serving-model the same way `generate.ts` does.

```ts
function normalizeModelsDev(files: {path,body}[]): Normalized {
  const providers = [], models = [], pricing = []
  const touchedModelKeys = [], touchedProviderSlugs = []

  for (const f of files) {
    // provider record:  providers/<slug>/provider.toml
    if (/^providers\/[^/]+\/provider\.toml$/.test(f.path)) {
      const slug = f.path.split('/')[1]                 // id derives from DIR name, not file (data-analysis §2.2)
      const p = parseToml(f.body)                        // tolerant parse; unknown keys allowed
      providers.push({
        slug, name: p.name, doc_url: p.doc, npm: p.npm,
        env_vars: JSON.stringify(p.env),
        api_base_url: p.api ?? null,                     // null for native providers; literal AWS_REGION stored as-is for bedrock
        provider_shape: deriveShape(p),                  // from api-refinement rule (data-analysis §2.2)
        portkey_slug: SLUG_ALIAS[slug] ?? slug,          // §5
      })
      touchedProviderSlugs.push(slug)
      continue
    }

    // serving model:  providers/<slug>/models/<model>.toml  (the row we want; carries cost — data-analysis §2.1)
    const m = f.path.match(/^providers\/([^/]+)\/models\/(.+)\.toml$/)
    if (m) {
      const [, slug, idPath] = m
      const modelId = idPath                             // id derives from path, dots literal (data-analysis §2.5)
      let raw = parseToml(f.body)                        // tolerant: unknown cost keys kept, NOT thrown (data-analysis §2.4 open-Q)

      // logical-model grouping key (01 §1 / 03 §2.4): the base_model LAB/MODEL path id, else a normalized model_id.
      // Captured here, before `delete raw.base_model` below clears it.
      const canonicalId = raw.base_model ? raw.base_model : normalizeModelId(modelId)
      if (raw.base_model) {
        const donor = loadDonor(raw.base_model)          // models/<LAB>/<MODEL>.toml  (resolve target; missing => error/skip+log)
        raw = applyOmit(mergeDeep(donor, raw), raw.base_model_omit)  // generate.ts mergeDeep: objects merge, arrays/primitives REPLACE; strip id/benchmarks/license/links/weights from donor
        delete raw.base_model                            // output has no base_model
      }
      const cost = normalizeCost(raw.cost)               // generate.ts normalizeCost: tier size>=200000 -> context_over_200k on OutputCost

      models.push(toModelRow(slug, modelId, canonicalId, raw))   // typed facets: capabilities, modalities, limits, dates, status, canonical_id

      pricing.push(toPricingRowModelsDev(slug, modelId, cost, raw.cost /* keep unknown keys */))
      touchedModelKeys.push([slug, modelId])
      touchedProviderSlugs.push(slug)
    }
    // models/<lab>/<model>.toml changes: re-resolve any serving model whose base_model points here (enrichment donor)
  }
  return { providers, models, pricing, touchedModelKeys, touchedProviderSlugs: dedupe(touchedProviderSlugs) }
}
```

> **Alternative (lower-risk, data-analysis §3 recommendation):** instead of re-implementing `mergeDeep`/`applyOmit`/`normalizeCost`, ingest the already-resolved **`api.json`** as the spine (resolution done, `base_model` merged away) and left-join `models.json` for benchmarks/license. Diffing raw TOML still tells us *which* providers/models changed; we then pull just those slices from the resolved artifact. Pick one and document; the pseudocode above is the raw-resolve path.

### 3.2 `normalizePortkey(files)` — cents/token → $/1M, custom_pricing (data-analysis §4)

```ts
function normalizePortkey(files: {path,body}[]): Normalized {
  const pricing = [], touchedModelKeys = []
  for (const f of files) {
    const m = f.path.match(/^(pricing|general)\/(.+)\.json$/)
    if (!m) continue
    const [, kind, portkeySlug] = m
    const modelsDevSlug = REVERSE_ALIAS[portkeySlug] ?? portkeySlug   // §5: bedrock->amazon-bedrock, x-ai->xai
    const obj = JSON.parse(f.body)

    for (const [rawKey, val] of Object.entries(obj)) {
      if (typeof val !== 'object' || val === null) continue           // skip top-level name/description scalars (data-analysis §4.1)
      if (rawKey === 'default') { /* keep as provider-wide fallback; portkey_default_used=1 when applied */ }

      const modelId = stripContextSuffix(rawKey)                      // strip -gt-128k / -lte-128k / -gt-200k (data-analysis §4.7)
      if (kind === 'pricing') {
        const pc = val.pricing_config?.pay_as_you_go ?? {}
        pricing.push({
          provider_slug: modelsDevSlug, model_id: modelId,
          currency: val.pricing_config?.currency ?? 'USD', unit: 'usd_per_1m',
          input_per_m:        centsPerTokenToPerM(pc.request_token?.price),         // (price/100)*1_000_000  (data-analysis §4.2)
          output_per_m:       centsPerTokenToPerM(pc.response_token?.price),
          cache_read_per_m:   centsPerTokenToPerM(pc.cache_read_input_token?.price),
          cache_write_per_m:  centsPerTokenToPerM(pc.cache_write_input_token?.price),
          reasoning_per_m:    centsPerTokenToPerM(val.pricing_config?.additional_units?.thinking_token?.price),
          input_audio_per_m:  centsPerTokenToPerM(pc.request_audio_token?.price),
          output_audio_per_m: centsPerTokenToPerM(pc.response_audio_token?.price),
          // long-tail blobs (gap-fill — data-analysis §4.4): store verbatim, do not flatten
          batch_json:          json(val.pricing_config?.batch_config),
          finetune_json:       json(val.pricing_config?.finetune_config),
          tool_surcharges_json:json(pickToolSurcharges(val.pricing_config?.additional_units)),
          image_pricing_json:  json(pc.image),
          video_pricing_json:  json(pickVideoFields(val.pricing_config)),  // gap-fill (data-analysis §4.4 "Video pricing"): video_seconds / video_duration_seconds_* / input_video_* raw keys
          custom_pricing_json: json(val.custom_pricing),               // OUT-OF-SPEC, raw-only (data-analysis §4.4 / §9 risk 5)
          calculate_json:      json(val.pricing_config?.calculate),
          pricing_source: 'portkey',
          portkey_default_used: rawKey === 'default' ? 1 : 0,
        })
        touchedModelKeys.push([modelsDevSlug, modelId])
      }
      // kind === 'general': capability params — optional enrichment; no context window (data-analysis §4.6)
    }
  }
  return { providers: [], models: [], pricing, touchedModelKeys, touchedProviderSlugs: [] }
}

const centsPerTokenToPerM = (price?: number) =>
  price == null ? null : (price / 100) * 1_000_000     // cents/token -> $/1M  (data-analysis §4.2)
```

### 3.3 Upsert conflict policy — **models.dev wins**

```ts
async function upsert(n: Normalized) {
  // providers / models: only models.dev writes these tables -> plain PK upsert
  for (const p of n.providers) await db.insert(providers).values(p).onConflictDoUpdate({ target: providers.slug, set: p })
  for (const m of n.models)    await db.insert(models).values(m).onConflictDoUpdate({ target: [models.provider_slug, models.model_id], set: m })

  // model_pricing: TWO sources merge on (provider_slug, model_id). models.dev wins on overlapping typed columns.
  for (const row of n.pricing) {
    if (row.pricing_source === 'modelsdev') {
      // models.dev: authoritative — overwrite the typed cost columns, mark provenance both/modelsdev
      await db.insert(model_pricing).values(row).onConflictDoUpdate({
        target: [model_pricing.provider_slug, model_pricing.model_id],
        set: { ...row, pricing_source: sql`CASE WHEN ${model_pricing.pricing_source}='portkey' THEN 'both' ELSE 'modelsdev' END` },
      })
    } else {
      // portkey: gap-fill ONLY. COALESCE keeps any existing models.dev value; portkey fills NULLs + owns gap-only blobs.
      await db.insert(model_pricing).values(row).onConflictDoUpdate({
        target: [model_pricing.provider_slug, model_pricing.model_id],
        set: {
          input_per_m:        sql`COALESCE(${model_pricing.input_per_m},        ${row.input_per_m})`,
          output_per_m:       sql`COALESCE(${model_pricing.output_per_m},       ${row.output_per_m})`,
          cache_read_per_m:   sql`COALESCE(${model_pricing.cache_read_per_m},   ${row.cache_read_per_m})`,
          cache_write_per_m:  sql`COALESCE(${model_pricing.cache_write_per_m},  ${row.cache_write_per_m})`,
          reasoning_per_m:    sql`COALESCE(${model_pricing.reasoning_per_m},    ${row.reasoning_per_m})`,
          input_audio_per_m:  sql`COALESCE(${model_pricing.input_audio_per_m},  ${row.input_audio_per_m})`,
          output_audio_per_m: sql`COALESCE(${model_pricing.output_audio_per_m}, ${row.output_audio_per_m})`,
          // gap-only dimensions models.dev never has -> portkey always owns
          batch_json: row.batch_json, finetune_json: row.finetune_json,
          tool_surcharges_json: row.tool_surcharges_json, image_pricing_json: row.image_pricing_json,
          video_pricing_json: row.video_pricing_json, custom_pricing_json: row.custom_pricing_json,
          calculate_json: row.calculate_json, cache_write_1h_per_m: row.cache_write_1h_per_m,
          portkey_default_used: row.portkey_default_used,
          pricing_source: sql`CASE WHEN ${model_pricing.pricing_source}='modelsdev' THEN 'both' ELSE 'portkey' END`,
        },
      })
    }
  }
}
```

**Ordering matters within a tick:** when both sources are processed, process models.dev pricing before Portkey for the same key so `COALESCE` sees the authoritative value already present. Since the two sources run as **separate Workflow instances**, the COALESCE policy is what enforces "models.dev wins" regardless of which instance lands first — Portkey never overwrites a non-NULL typed column.

---

## 4. Idempotency, Failure & Resume

- **Cursor advances last, only on success.** `advance-cursor` (step 10) runs only after parse→upsert→prune→reindex→logos all complete. If any earlier step throws after its retries are exhausted, the Workflow instance fails **without** advancing `sync_state.last_sha`. The next cron tick reads the *same* `base` and re-diffs `base..HEAD` — re-processing the same (or a superset of) changed files.
- **Re-runs are no-ops.** Every write is a PK-keyed upsert (`onConflictDoUpdate`); pruning a path whose row is already gone is a no-op; FTS reindex is delete-then-insert for the touched keys. Re-ingesting the same SHA produces identical rows.
- **Per-step checkpoints.** Each `step.do(...)` is a durable checkpoint: on retry/resume the Workflow re-enters at the failed step with prior step results memoized — we do **not** re-fetch HEAD or re-diff if those steps already succeeded; we resume at the first incomplete step. Use `retries: { limit, delay, backoff: 'exponential' }` (set above on the network-bound steps 2/3/4/9) for transient `5xx`/timeouts.
- **Rate-limit / 403:** treated as a step failure (no cursor advance) — next tick retries after the GitHub reset window (§1.5).
- **`touchSync` vs `advanceCursor`:** the unchanged early-out records `last_status='unchanged'` and `synced_at` **without** moving `last_sha` semantics (it's already == head). A failed run leaves `last_status` reflecting the failure for observability, cursor untouched.
- **Partial-batch safety:** because upsert/prune/reindex operate on the *whole* diff set inside their own steps, a crash mid-`upsert` re-runs the entire `upsert` step on resume (idempotent), never leaving the cursor ahead of committed data.

`sync_state` row shape (owned by this pipeline): `source TEXT PK, last_sha TEXT, synced_at INT, last_status TEXT, changed_count INT`.

---

## 5. models.dev-slug ↔ Portkey-slug Alias Seed (data-analysis §4.7 / §9 risk 4)

A missing alias **silently drops Portkey pricing**, so this map is seeded explicitly and is the single reconciliation point. `SLUG_ALIAS` maps a models.dev slug → its Portkey counterpart(s); `REVERSE_ALIAS` is the inverse used when ingesting Portkey files. Some models.dev providers map to **two** Portkey files (e.g. `google` ↔ `google` + `vertex-ai`).

```ts
// models.dev slug -> portkey slug(s). Identity when omitted.
const SLUG_ALIAS: Record<string, string[]> = {
  'amazon-bedrock': ['bedrock'],
  'xai':            ['x-ai'],
  'google':         ['google', 'vertex-ai'],   // models.dev 'google' draws from BOTH portkey files
  'azure':          ['azure-openai'],
  'together':       ['together-ai'],
  'fireworks':      ['fireworks-ai'],
  // identity (listed for documentation / explicitness):
  'openai':         ['openai'],
  'anthropic':      ['anthropic'],
  'groq':           ['groq'],
  'mistral':        ['mistral'],
  'cohere':         ['cohere'],
  'openrouter':     ['openrouter'],
}

// portkey slug -> models.dev slug (inverse; built from SLUG_ALIAS at module load)
const REVERSE_ALIAS: Record<string, string> = buildReverse(SLUG_ALIAS)
// => { bedrock:'amazon-bedrock', 'x-ai':'xai', 'vertex-ai':'google', google:'google',
//      'azure-openai':'azure', 'together-ai':'together', 'fireworks-ai':'fireworks', ... }
```

**Maintenance note (data-analysis §9 risk 4):** when ingestion logs a Portkey file whose `REVERSE_ALIAS` resolves to a `provider_slug` with **zero** matching models in the `models` table, emit a warning (`pricing_source='portkey'` rows with no joinable model) so a missing/changed alias surfaces instead of silently dropping pricing.

---

## 6. Test Strategy (reuse Real/Fake seam + vitest-pool-workers)

Mirrors the repo's existing seam (`getService(env)` Real/Fake on `USE_FAKE_JOBS`, `worker/services/Service.ts`) and the two-suite split. See the `cloudflare-workflow-e2e` skill for `introspectWorkflowInstance` patterns.

### 6.1 Seam extension

Add to the existing interfaces:
```ts
interface Ingestor {
  getCursor(source): Promise<string|null>
  resolveHead(repo): Promise<string>
  compare(repo, base, head): Promise<{ changed: string[]; removed: string[]; truncated: boolean }>
  fullFileList(repo, head): Promise<{ changed: string[]; removed: string[]; truncated: boolean }>
  fetchRaw(repo, head, paths): Promise<{ path: string; body: string }[]>
  normalizeModelsDev(files): Normalized
  normalizePortkey(files): Normalized
  upsert(n): Promise<void>
  prune(source, removed): Promise<void>
  reindexFts(keys): Promise<void>
  syncLogos(slugs): Promise<void>
  advanceCursor(source, head, status, n): Promise<void>
  touchSync(source, head, status, n): Promise<void>
}
interface Service { catalog(): Catalog; ingest(): Ingestor }   // extend, don't replace
```
`getService(env)` returns `RealIngestor(env)` (touches `DB`/`KV`/GitHub via `fetch`) or `FakeIngestor` (canned, deterministic) per `USE_FAKE_JOBS`. The `scheduled` handler already no-ops when fake (§2.2) so e2e never hits GitHub.

### 6.2 Workflow-logic tests — `@cloudflare/vitest-pool-workers` (`test:cf`)

Real D1/KV in `workerd`. As in the existing `e2e-cf/detail.cache.test.ts` pattern, **migrations are NOT auto-applied in the pool** — `CREATE TABLE` the catalog + `sync_state` in `beforeEach`, then `DELETE` to reset.

- **`introspectWorkflowInstance`** (skill) to drive the Workflow: `disableSleeps`, `mockStepResult('resolve-head', 'sha-HEAD')`, `mockStepResult('diff', { changed:['providers/anthropic/models/claude-sonnet-4-5.toml'], removed:[] })`, `mockStepResult('fetch-files', [{ path, body: FIXTURE_TOML }])`, then let `normalize`/`upsert`/`reindex` run for real against D1.
- **Fixtures** = real TOML/JSON inputs checked into `worker/ingest/__fixtures__/`:
  - `modelsdev/providers/anthropic/provider.toml`, `.../models/claude-sonnet-4-5.toml` (with `base_model`), a donor `models/anthropic/claude-sonnet-4-5.toml`, and a `cost` carrying an unknown key (`citation`) to assert tolerant parsing (data-analysis §2.4).
  - `portkey/pricing/anthropic.json` (with `default` + a dated model + cents-per-token leaves), `portkey/pricing/google.json` (with `-gt-128k` suffix key), a `custom_pricing` sample from `pricing/openai.json`.
- **Assertions = D1 rows:** after the run, `SELECT * FROM models WHERE provider_slug='anthropic'` asserts resolved capabilities/limits; `SELECT input_per_m FROM model_pricing ...` asserts cents/token→$/1M (`0.003` cents → `$30/1M`); assert models.dev wins (set a models.dev `input` then a conflicting Portkey `request_token`, assert the models.dev value survives, `pricing_source='both'`); assert `custom_pricing_json` stored verbatim; assert the `-gt-128k` key collapsed to the base id; assert a removed-file path pruned its row; assert `sync_state.last_sha === head` only on success and **unchanged** on an injected `mockStepError('upsert', ...)`.
- **Idempotency test:** run the same fixtures twice; assert row counts and column values identical (no duplicate rows, no double-counted pricing).

### 6.3 e2e (`test:e2e`) — real `wrangler dev` over HTTP, `USE_FAKE_JOBS=true`

The `scheduled` handler no-ops under the fake seam, so e2e covers the **read** API (`/api/v1/models` etc.) over a `FakeIngestor`-seeded D1, not live ingestion. Live-ingestion contract (real GitHub) belongs in a token-gated `test:e2e:live`-style suite mirroring the existing HuggingFace live pattern — throw in `beforeAll` if `GITHUB_TOKEN` is absent (never `skip`).

### 6.4 Pure-function unit tests (fast, no Worker)

`centsPerTokenToPerM`, `mergeDeep`/`applyOmit`/`normalizeCost` (table-driven against data-analysis §2.5 examples), `deriveShape`, `stripContextSuffix`, `pickVideoFields` (§3.2), `canonicalId` derivation (`base_model` LAB/MODEL path id vs `normalizeModelId` fallback — 01 §1 / 03 §2.4), `SLUG_ALIAS`/`REVERSE_ALIAS` round-trip, and the compare-API `files[]`→`{changed,removed,truncated}` mapper (including the 300-file truncation flag).
