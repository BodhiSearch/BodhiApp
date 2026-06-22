# Catalog API Contract — Explore · API Models / API Providers

**Audience:** the backend engineer adding the catalog surface to `api-getbodhi-app` (`apps/api-getbodhi-app`), and the frontend engineer wiring the two BodhiApp Explore pages.
**Scope:** the HTTP contract the two Explore pages consume — five `GET` endpoints, their zod-openapi request/response schemas, facet-count + pagination shapes, the configure bridge payload, the `@bodhiapp/reference-api-types` additions, caching, and the precise design-consumer mapping.
**Grounding:** route/middleware conventions are taken verbatim from the live repo (`worker/app.ts`, `worker/routes/models.{list,detail}.ts`, `worker/schemas/*.ts`); the data model + fields are from `../data-analysis.md` §5/§7/§8; the design consumers are from `../design-prompt.md`.

> **Premise reminder (data-analysis §6.1):** none of this exists yet. The repo is a live HF GGUF proxy with one generic `model_cache` table — no `providers`/`models`/`model_pricing` tables, no FTS5, no provider routes. This document is the *target wire contract* the ingestion build (`01`/`02` plans) serves. Field names below are the **served-catalog** names (resolved `api.json` shape — `base_model` is already merged away, `cost` is `$/1M tokens`, `limit` is `{context,output}` only).

---

## 0. Conventions inherited from the existing router (do not re-invent)

These are copied from the live `models.list`/`models.detail` routes so the new routes drop into the same `OpenAPIHono<AppEnv>` app:

- **Route registration:** one `register<Name>(app)` fn per file under `worker/routes/`, each builds a `createRoute({method,path,request,responses})` and calls `app.openapi(route, handler)`. Register them in `worker/app.ts` alongside `registerModelsList`/`registerModelsDetail`.
- **Service seam:** handlers only ever call `getService(c.env).catalog()` (extend the `Catalog`/`Service` interface in `worker/services/Service.ts` with the new methods + a `FakeService` twin — see data-analysis §6, do **not** touch `env.DB`/`env.KV` in a route).
- **Validation envelope:** zod 400s are mapped to `{error:'validation', message}` by the app-level `defaultHook`. Keep enum-vs-loose-string discipline: a value that is *structurally* valid but *semantically* unsupported (e.g. an unknown provider slug on a path) returns **422** from the handler via the `errorSchema`, not a 400.
- **Errors:** reuse `errorSchema` (`{error, message?}`) for 404/422 and the app `onError` for 500 (`{error:'internal'}`). Catalog endpoints are DB-backed, **not** upstream proxies, so `upstreamErrorSchema`/502 from the HF path does **not** apply here.
- **Auth:** `authMiddleware` is already mounted on `/api/*` and is public read-through (no `Authorization` ⇒ `next()`; a present-but-invalid bearer ⇒ 401). All five catalog endpoints are public reads — no per-route auth.
- **Path prefix:** all under `/api/v1/`.
- **zod source:** `import { z } from '@hono/zod-openapi'` (the openapi-extended z, with `.openapi()`), exactly as `worker/schemas/*.ts` does today.
- **Types single-source-of-truth:** every response schema is constrained to a `@bodhiapp/reference-api-types` interface via a compile-time `satisfies`/assignment guard (the `_modelTypeGuard` pattern in `worker/schemas/model.ts:67`). Add the interfaces in §4, then guard each schema below.

> **Path-collision note:** the existing single-model route is `GET /api/v1/models/{source}/{namespace}/{repo}` (3 path segments). The new catalog model-detail is `GET /api/v1/models/{slug}/{model_id}` (2 segments). Hono matches by segment count so these do **not** collide, but `model_id` frequently contains slashes/dots (Bedrock `anthropic.claude…`, dated ids) — **the model id MUST be URL-encoded by the client and `decodeURIComponent`-ed in the handler** (same as the existing detail route does for `namespace`/`repo`). Register the 3-segment route before the 2-segment one to avoid greedy-match ambiguity, or keep the trailing segment a single `{model_id}` capture and require encoding.

---

## 1. Shared schemas (put in `worker/schemas/catalog.ts`)

These primitives are referenced by multiple endpoints. Each `.openapi('Name')` name becomes a reusable OpenAPI component.

```ts
import { z } from '@hono/zod-openapi'

// ── Enums (mirror models.dev served values) ──────────────────────────────────────────────────────
export const modalitySchema = z.enum(['text', 'audio', 'image', 'video', 'pdf']).openapi('Modality')
// models.dev `status` is OPTIONAL with only these real values; absent ⇒ the UI synthesizes "Stable".
export const modelStatusSchema = z.enum(['alpha', 'beta', 'deprecated']).openapi('ModelStatus')

// Capability keyword bag, derived from the boolean fields + modalities (vision = modalities.input
// includes 'image'). Display + facet use. Mirrors the design's capability chips.
export const capabilitySchema = z.enum(['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision']).openapi('Capability')

// ── Unified pricing shape ($/1M tokens) ──────────────────────────────────────────────────────────
// Typed columns = the common dimensions the pricing facet + spec grid use (data-analysis §5.3).
// The long-tail (image/video maps, batch, finetune, custom_pricing, calculate AST) is carried as
// opaque JSON so the contract is stable while ingestion enriches it — NOT typed per-field in v1.
export const pricingSchema = z
  .object({
    currency: z.string().default('USD'),
    // All per-1M-token. null = field absent at source (omit the row in the rail).
    input_per_m: z.number().nullable(),
    output_per_m: z.number().nullable(),
    cache_read_per_m: z.number().nullable(),
    cache_write_per_m: z.number().nullable(),
    cache_write_1h_per_m: z.number().nullable(),
    reasoning_per_m: z.number().nullable(),
    input_audio_per_m: z.number().nullable(),
    output_audio_per_m: z.number().nullable(),
    // Opaque enrichment blobs (Portkey gap dimensions; data-analysis §4.4 + §5.3). Present only on
    // model-detail, omitted on lite rows. `unknown` so the wire shape never breaks on new keys.
    context_over_200k: z.unknown().nullable().optional(),
    batch: z.unknown().nullable().optional(),
    finetune: z.unknown().nullable().optional(),
    tool_surcharges: z.unknown().nullable().optional(),
    image_pricing: z.unknown().nullable().optional(),
    video_pricing: z.unknown().nullable().optional(),
    custom_pricing: z.unknown().nullable().optional(),
    calculate: z.unknown().nullable().optional(),
    // Provenance (data-analysis §5.3): which source filled this row.
    pricing_source: z.enum(['modelsdev', 'portkey', 'both']).nullable(),
    portkey_default_used: z.boolean().default(false),
  })
  .openapi('Pricing')

// A "lite" pricing shape for list rows / served-by rows — only the band-relevant fields.
export const pricingLiteSchema = z
  .object({
    input_per_m: z.number().nullable(),
    output_per_m: z.number().nullable(),
    cache_read_per_m: z.number().nullable(),
    cache_write_per_m: z.number().nullable(),
  })
  .openapi('PricingLite')

// ── Pagination (matches the design's numbered pager) ─────────────────────────────────────────────
// API-Models mode pages 8 models/page (design §2d `usePagination(rows, 8)`); API-Providers mode
// pages 5/page (data-analysis §7.1). page is 1-based; the unit ('models' | 'providers') is a UI
// label only — the server returns counts in the resource's natural unit.
export const pageMetaSchema = z
  .object({
    page: z.number().int().min(1),
    page_size: z.number().int().min(1),
    total: z.number().int().min(0),
  })
  .openapi('PageMeta')

// ── Facet counts (so the sidebar shows LIVE counts) ──────────────────────────────────────────────
// Each facet is a record of value → count over the CURRENT result set (post-filter, pre-pagination).
// Counts are computed with the OTHER filters applied but the facet's own dimension released, so a
// chip shows how many rows it WOULD add (standard faceted-search semantics).
export const facetBucketSchema = z.record(z.string(), z.number().int().min(0))
export const modelFacetsSchema = z
  .object({
    capability: facetBucketSchema, // keys: reasoning|tool_call|structured_output|attachment|vision
    modality: facetBucketSchema, //   keys: text|audio|image|video|pdf
    status: facetBucketSchema, //     keys: alpha|beta|deprecated|stable (stable = status absent)
    provider: facetBucketSchema, //   keys: provider slug → model count
    open_weights: facetBucketSchema, // keys: open|closed
  })
  .openapi('ModelFacets')
```

---

## 2. Endpoints

### 2.1 `GET /api/v1/providers` — provider list + facets

**Design consumer:** Page B (API Providers, mode `'api'`) **list** + sidebar. Row = one provider (rank / `ProviderLogo` / `name` / models-count / format). Sidebar groups Status·Capability·Pricing·API-Format. — design-prompt §3 ("Keep the row list shape (`ApiRow`: rank, `ProviderLogo`, name, models count, format)"; sidebar realign bullet), data-analysis §7.1.

**Request — query (`worker/schemas/catalog.ts`):**

```ts
export const providersQuerySchema = z.object({
  q: z.string().min(1).max(200).optional().openapi({ description: 'Free-text over provider name/slug.' }),
  capability: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(capabilitySchema).optional())
    .openapi({ description: 'Repeatable; provider matches if ANY of its models has the capability.' }),
  api_format: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(z.enum(['openai', 'openai_responses', 'anthropic', 'anthropic_oauth', 'gemini', 'other'])).optional())
    .openapi({ description: 'Repeatable; the BodhiApp api_format the provider maps to (§3). "other" = unmapped openai-compatible.' }),
  pricing_max: z.coerce.number().nonnegative().optional().openapi({ description: 'Max input $/Mtok across the provider (range filter, design $/M slider).' }),
  sort: z.enum(['rank', 'name', 'model_count']).default('rank').openapi({ description: 'rank = model_count desc (the #N rank the design renders).' }),
  page: z.coerce.number().int().min(1).default(1),
  page_size: z.coerce.number().int().min(1).max(100).default(5), // design API-mode pager = 5/page
})
```

**Response — `200 providerListResponseSchema`:**

```ts
export const providerSummarySchema = z
  .object({
    slug: z.string(), // models.dev dir id — canonical (anthropic, amazon-bedrock)
    name: z.string(),
    logo_url: z.string().nullable(), // served logo (see §5 caching); null ⇒ UI falls back to monogram
    model_count: z.number().int(),
    rank: z.number().int(), // 1-based, by model_count desc; the design's #N
    api_base_url: z.string().nullable(), // models.dev `api`; null for native providers (bridge fills, §3)
    provider_shape: z.enum(['openai', 'openai-compatible', 'anthropic', 'openrouter', 'kiro', 'native']),
    api_format_hint: z.enum(['openai', 'openai_responses', 'anthropic', 'anthropic_oauth', 'gemini', 'other']), // for the API-Format facet + bridge
    capabilities_summary: z.array(capabilitySchema), // union across the provider's models (facet)
    pricing_summary: z.object({ min_in_per_m: z.number().nullable(), min_out_per_m: z.number().nullable() }), // for the pricing range facet
  })
  .openapi('ProviderSummary')

export const providerListResponseSchema = z
  .object({
    items: z.array(providerSummarySchema),
    facets: z
      .object({
        capability: facetBucketSchema,
        api_format: facetBucketSchema, // keys: openai|openai_responses|anthropic|anthropic_oauth|gemini|other
      })
      .openapi('ProviderFacets'),
    page: pageMetaSchema.shape.page,
    page_size: pageMetaSchema.shape.page_size,
    total: pageMetaSchema.shape.total,
  })
  .openapi('ProviderList')
```

### 2.2 `GET /api/v1/providers/{slug}` — provider detail + configure bridge

**Design consumer:** Page B provider **detail rail header + "Provider meta block"** (Env var / Base URL / Docs / npm) and the **"Configure in Bodhi"** derivation. — design-prompt §3 ("Provider meta block (new): `Env var` = `env[]` … `Base URL` = `api` … `Docs` = `doc` … `npm` = `npm`"), §5 (Configure prefill contract), data-analysis §8.

**Request — path:** `slug` (string, the models.dev canonical slug). Unknown slug ⇒ **404** `errorSchema` (`{error:'not_found'}`).

**Response — `200 providerDetailResponseSchema`:** this is the **configure-bridge payload** (§3 below has the full mapping table). It carries everything the catalog *can* supply; the user always supplies `api_key`.

```ts
export const providerDetailResponseSchema = z
  .object({
    slug: z.string(),
    name: z.string(),
    logo_url: z.string().nullable(),
    model_count: z.number().int(),
    // ── models.dev provider record (data-analysis §2.2) ──
    env: z.array(z.string()), // credential env var names, e.g. ["ANTHROPIC_API_KEY"]
    npm: z.string().nullable(), // SDK npm package
    doc_url: z.string().nullable(), // docs URL
    api_base_url: z.string().nullable(), // models.dev `api`; null for native (anthropic/openai/google)
    provider_shape: z.enum(['openai', 'openai-compatible', 'anthropic', 'openrouter', 'kiro', 'native']),
    // ── configure bridge (data-analysis §8) ──
    bridge: z
      .object({
        // The api_format the user's config should use (BodhiApp ApiFormat enum, §3.1).
        api_format: z.enum(['openai', 'openai_responses', 'anthropic', 'anthropic_oauth', 'gemini']),
        // The base_url the form should prefill: models.dev `api` when present, else a BodhiApp preset.
        base_url: z.string().nullable(),
        // Where base_url came from — so the form can flag "base URL required" when neither source has one.
        base_url_source: z.enum(['modelsdev_api', 'bodhi_preset', 'user_required']),
        // Bedrock's `api` is a literal AWS_REGION placeholder (data-analysis §2.2/§9.9) — flag it.
        base_url_requires_substitution: z.boolean().default(false),
      })
      .openapi('ConfigureBridge'),
  })
  .openapi('ProviderDetail')
```

### 2.3 `GET /api/v1/providers/{slug}/models` — models served by a provider

**Design consumer:** Page B provider-detail rail **"Models (N)"** table — each `prov-mrow` row `{name, caps[], ctx (=limit.context), $in/$out (=cost.input/output)}`, `fmtPrice` Free rule. — design-prompt §3 ("Models (N) section … each `prov-mrow` shows model `name` + mini specs `ctx` … + `$in / $out` … + capability chips"), data-analysis §7.2.

**Request:** path `slug`; query `page`/`page_size` (defaults `page=1`, `page_size=50`), optional `sort` (`name`|`context`|`price`). Unknown slug ⇒ 404.

**Response — `200 providerModelsResponseSchema`:**

```ts
export const providerModelRowSchema = z
  .object({
    model_id: z.string(), // exact API string (the join-key id)
    name: z.string(),
    caps: z.array(capabilitySchema), // derived from the boolean fields + vision
    context_limit: z.number().int().nullable(), // limit.context
    output_limit: z.number().int().nullable(), // limit.output
    pricing: pricingLiteSchema, // this provider's price for this model
    status: modelStatusSchema.nullable(), // absent ⇒ stable
    modalities_in: z.array(modalitySchema),
    modalities_out: z.array(modalitySchema),
  })
  .openapi('ProviderModelRow')

export const providerModelsResponseSchema = z
  .object({
    items: z.array(providerModelRowSchema),
    page: pageMetaSchema.shape.page,
    page_size: pageMetaSchema.shape.page_size,
    total: pageMetaSchema.shape.total,
  })
  .openapi('ProviderModelList')
```

### 2.4 `GET /api/v1/models` — global model search/list + facet counts

**Design consumer:** Page A (API Models, mode `'api-catalog'`) **list, columns, sidebar facets, search, numbered pager**. — design-prompt §2b (columns: rank/Model/Context/Input $/Output $/Capabilities/Providers), §2a (the 7 filter groups), §2d (`ShellSearch` ⌘K, `usePagination(rows, 8)`, numbered `<Pagination unit="models">`), data-analysis §7.4.

> **Logical-model grouping (the spine of this endpoint).** The `models` table stores **serving rows** — one per `(provider_slug, model_id)` (~5297, data-analysis §2.1), NOT ~220 abstract models. Page A lists **logical models**, so the list/detail group serving rows by `models.canonical_id` (01 §1; derived from the `base_model` `LAB/MODEL` path id, falling back to a normalized `model_id` when a serving row has no `base_model`). This grouping is the documented backing for everything below:
> - `GET /api/v1/models` does `GROUP BY canonical_id`, emitting one `ModelLite` per group. `provider_count = COUNT(DISTINCT provider_slug)` within the group; `sort=providers` orders by that count desc.
> - The **primary/representative provider** for a group (whose `slug`, `pricing`, and per-provider fields the `ModelLite` row carries) is the serving row with the **lowest non-null `input_per_m`** (cheapest), tie-broken by provider `model_count` desc then `slug` asc — deterministic so the row's deep-link + Configure target is stable.
> - `GET /api/v1/models/{slug}/{model_id}` resolves the group from the `(slug, model_id)` serving row's `canonical_id`; `served_by[]` is the other serving rows in that group, and `bridge` targets the primary provider.
>
> Facet counts (`provider`, etc.) are over the **grouped** result set. `01` owns the `canonical_id` column + its index; this endpoint owns the GROUP BY semantics.

**Request — query (full §7.4 table, faithfully):**

```ts
export const modelsQuerySchema = z.object({
  q: z.string().min(1).max(200).optional().openapi({ description: 'Free-text search → models_fts (name, model_id, provider, family, modalities, capability bag).' }),
  capability: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(capabilitySchema).optional())
    .openapi({ description: 'Repeatable; AND. tool_call|reasoning|structured_output|attachment|vision.' }),
  modality: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(modalitySchema).optional())
    .openapi({ description: 'Repeatable; OR. Matches modalities_in ∪ modalities_out.' }),
  pricing_max: z.coerce.number().nonnegative().optional().openapi({ description: 'Max input $/Mtok (design slider $0–$75/M, step 0.5). Range filter on input_per_m.' }),
  pricing_band: z.enum(['free', 'low', 'mid', 'high']).optional().openapi({ description: 'Coarse band alternative to pricing_max. free = input==0 && output==0.' }),
  context_min: z.coerce.number().int().nonnegative().optional().openapi({ description: 'Min context window (tokens). Design slider in K.' }),
  status: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(z.enum(['alpha', 'beta', 'deprecated', 'stable'])).optional())
    .openapi({ description: 'Repeatable. "stable" selects rows whose status is ABSENT.' }),
  provider: z
    .preprocess((v) => (v == null ? undefined : Array.isArray(v) ? v : [v]), z.array(z.string()).optional())
    .openapi({ description: 'Repeatable; OR. Provider slug filter (also the page-B → page-A deep-link param).' }),
  open_weights: z.enum(['open', 'closed']).optional().openapi({ description: 'open = open_weights true; closed = false.' }),
  sort: z.enum(['updated', 'context', 'price', 'name', 'providers']).default('updated').openapi({ description: 'updated=last_updated desc, context=limit.context desc, price=input_per_m asc, providers=COUNT(DISTINCT provider_slug) per canonical_id group desc (see §2.4 grouping note).' }),
  page: z.coerce.number().int().min(1).default(1),
  page_size: z.coerce.number().int().min(1).max(100).default(8), // design API-Models pager = 8/page
})
```

**Response — `200 modelsListResponseSchema`** (`items` are **detail-lite** rows — enough for the list + columns; full blobs only on §2.5):

```ts
export const modelLiteSchema = z
  .object({
    slug: z.string(), // primary/representative provider slug — cheapest input_per_m in the canonical_id group (§2.4); the row's deep-link + Configure target
    model_id: z.string(), // the primary provider's exact model_id for that group (paired with slug for the detail route)
    name: z.string(),
    family: z.string().nullable(), // OPTIONAL/sparse — muted sub-line
    context_limit: z.number().int().nullable(), // limit.context — "Context" column
    output_limit: z.number().int().nullable(),
    pricing: pricingLiteSchema, // PRIMARY/representative provider price — "Input $"/"Output $" columns
    caps: z.array(capabilitySchema), // "Capabilities" chips (only the true ones)
    status: modelStatusSchema.nullable(),
    open_weights: z.boolean(),
    modalities_in: z.array(modalitySchema),
    modalities_out: z.array(modalitySchema),
    provider_count: z.number().int(), // "Providers" column = COUNT(DISTINCT provider_slug) in the canonical_id group (§2.4)
    release_date: z.string().nullable(),
    last_updated: z.string().nullable(),
  })
  .openapi('ModelLite')

export const modelsListResponseSchema = z
  .object({
    items: z.array(modelLiteSchema),
    facets: modelFacetsSchema, // live sidebar counts (design §2a wants real counts, not display-only)
    page: pageMetaSchema.shape.page,
    page_size: pageMetaSchema.shape.page_size,
    total: pageMetaSchema.shape.total,
  })
  .openapi('ModelsList')
```

### 2.5 `GET /api/v1/models/{slug}/{model_id}` — full model detail

**Design consumer:** Page A per-model **detail rail** — `DetailHeader` (monogram/family/status/open_weights), the **spec grid** (Pricing · Limits · Modalities · Capabilities · Meta), and the **"Served by (N)"** provider list. — design-prompt §2c, data-analysis §7.3. Cache-aside (KV→D1) per §5.

**Request:** path `slug` + `model_id` (**URL-encoded; `decodeURIComponent` in handler** — ids carry dots/colons/slashes). Missing pair ⇒ 404 `errorSchema`.

**Response — `200 modelDetailResponseSchema`:**

```ts
export const servedBySchema = z
  .object({
    slug: z.string(),
    name: z.string(),
    logo_url: z.string().nullable(),
    base_url: z.string().nullable(), // muted sub-line in the "Served by" row
    pricing: pricingLiteSchema, // this provider's $in/$out for THIS model (right-aligned in the row)
  })
  .openapi('ServedBy')

export const modelDetailResponseSchema = z
  .object({
    slug: z.string(), // primary/representative provider of the canonical_id group (§2.4)
    model_id: z.string(),
    name: z.string(),
    family: z.string().nullable(),
    status: modelStatusSchema.nullable(),
    // capabilities (data-analysis §5.2) — booleans, the rail renders chips + derives `vision`
    reasoning: z.boolean(),
    tool_call: z.boolean(),
    structured_output: z.boolean().nullable(),
    attachment: z.boolean(),
    open_weights: z.boolean(),
    temperature: z.union([z.boolean(), z.number()]).nullable(),
    reasoning_options: z.unknown().nullable(), // {toggle?,effort?,budget_tokens?} — muted note if present
    // limits + modalities
    context_limit: z.number().int().nullable(), // limit.context
    output_limit: z.number().int().nullable(), // limit.output (NO input key — models.dev limit is {context,output})
    modalities_in: z.array(modalitySchema),
    modalities_out: z.array(modalitySchema),
    // dates
    release_date: z.string().nullable(),
    last_updated: z.string().nullable(),
    knowledge_cutoff: z.string().nullable(), // models.dev `knowledge`
    // pricing — full unified shape (PRIMARY/representative provider) + blobs
    pricing: pricingSchema,
    // enrichment (models.json — data-analysis §5.2)
    license: z.string().nullable(),
    links: z.unknown().nullable(),
    weights: z.unknown().nullable(),
    benchmarks: z.unknown().nullable(),
    // the providers serving this model — the other serving rows in the same canonical_id group, each with its own price (design "Served by (N)", §2.4)
    served_by: z.array(servedBySchema),
    // configure bridge for the PRIMARY provider (so the footer CTA can prefill without a 2nd call)
    bridge: providerDetailResponseSchema.shape.bridge,
  })
  .openapi('ModelDetail')
```

---

## 3. Configure-bridge payload (data-analysis §8)

`GET /api/v1/providers/{slug}` (and the embedded `bridge` on `GET /api/v1/models/{slug}/{model_id}`) is what powers design-prompt §5's "Configure in Bodhi" prefill. The catalog derives `(api_format, base_url)`; the **user always supplies `api_key`**; the catalog can pre-fill the model id.

### 3.1 BodhiApp `api_format` enum (the only legal target values)

`openai`, `openai_responses`, `anthropic`, `anthropic_oauth`, `gemini`, `llm_liberty_oauth`. `supports_chat_completions` is true only for `openai`/`anthropic`/`anthropic_oauth`. The bridge emits one of `openai`/`openai_responses`/`anthropic`/`anthropic_oauth`/`gemini` (data-analysis §8.1).

### 3.2 Mapping table — models.dev provider/shape → BodhiApp `(api_format, base_url)`

| models.dev provider / shape | `api_format` | `base_url` | `base_url_source` |
|---|---|---|---|
| `anthropic` (native, no `api`) | `anthropic` | BodhiApp preset `https://api.anthropic.com/v1` | `bodhi_preset` |
| `openai` (native, no `api`) | `openai` | BodhiApp preset `https://api.openai.com/v1` | `bodhi_preset` |
| `google` (native, no `api`) | `gemini` | BodhiApp preset `https://generativelanguage.googleapis.com/v1beta` | `bodhi_preset` |
| `openrouter` (shape requires `api`) | `openai` | **models.dev `api`** | `modelsdev_api` |
| any `openai-compatible` (groq, x-ai/xai, mistral, together, deepseek…) | `openai` | **models.dev `api`** | `modelsdev_api` |
| native w/o `api` and no BodhiApp preset | `openai` (best-effort) | `null` | `user_required` |
| `amazon-bedrock` (`api` = literal `AWS_REGION`) | `openai` (best-effort) | the literal string, with `base_url_requires_substitution:true` | `modelsdev_api` |

models.dev's `api` (when present) is authoritative; native providers have no `api`, so the bridge falls back to the BodhiApp preset (BodhiApp has **no backend preset table** — presets are frontend constants, data-analysis §8.2, so the catalog emits the URL string here so the form needn't re-derive it).

### 3.3 Catalog-supplies vs user-supplies

| Field | Catalog supplies? | Source |
|---|---|---|
| `api_format` | yes | `bridge.api_format` |
| `base_url` | yes when `api`/preset known; `null` ⇒ form shows "base URL required" | `bridge.base_url` + `base_url_source` |
| model ids to expose | yes | `GET /api/v1/providers/{slug}/models` |
| **`api_key`** | **NO — user always supplies** | user |
| `prefix` / `extra_headers` / `extra_body` | no (UI defaults; `extra_headers` only for `anthropic_oauth`) | UI |

> **Hard constraint (data-analysis §8.3):** BodhiApp's `create_default` calls `fetch_models` **before** persist, so the prefilled `base_url` must actually answer with the user's key. `api_format` is immutable on edit. The bridge only provides good defaults — it does not (and cannot) validate the key.

---

## 4. `@bodhiapp/reference-api-types` additions

Hand-written interfaces appended to `packages/api-types/src/index.ts` (the package is types-only, hand-maintained, single-source-of-truth; the worker imports these and the zod schemas above are guarded against them via `satisfies`/assignment — see `worker/schemas/model.ts:67`). Adding them is one `api/v*` publish cycle (data-analysis §7 / typesPackage note). Mirror the existing JSDoc-per-field style.

```ts
// ── Catalog: shared enums ─────────────────────────────────────────────────────────────────────
export type Modality = 'text' | 'audio' | 'image' | 'video' | 'pdf'
/** Only real models.dev values; absent ⇒ "Stable" (synthesized by the UI). */
export type ModelStatus = 'alpha' | 'beta' | 'deprecated'
export type Capability = 'reasoning' | 'tool_call' | 'structured_output' | 'attachment' | 'vision'
/** BodhiApp api_format the bridge can target. */
export type BridgeApiFormat = 'openai' | 'openai_responses' | 'anthropic' | 'anthropic_oauth' | 'gemini'
export type ProviderShape = 'openai' | 'openai-compatible' | 'anthropic' | 'openrouter' | 'kiro' | 'native'
export type ApiFormatHint = BridgeApiFormat | 'other'

// ── Catalog: pricing ($/1M tokens) ────────────────────────────────────────────────────────────
export interface PricingLite {
  input_per_m: number | null
  output_per_m: number | null
  cache_read_per_m: number | null
  cache_write_per_m: number | null
}
export interface Pricing extends PricingLite {
  currency: string
  cache_write_1h_per_m: number | null
  reasoning_per_m: number | null
  input_audio_per_m: number | null
  output_audio_per_m: number | null
  /** Opaque Portkey-gap blobs; present only on model detail. */
  context_over_200k?: unknown | null
  batch?: unknown | null
  finetune?: unknown | null
  tool_surcharges?: unknown | null
  image_pricing?: unknown | null
  video_pricing?: unknown | null
  custom_pricing?: unknown | null
  calculate?: unknown | null
  pricing_source: 'modelsdev' | 'portkey' | 'both' | null
  portkey_default_used: boolean
}

// ── Catalog: pagination + facets ──────────────────────────────────────────────────────────────
export interface PageMeta {
  page: number
  page_size: number
  total: number
}
/** value → count over the current result set. */
export type FacetBucket = Record<string, number>
export interface ModelFacets {
  capability: FacetBucket
  modality: FacetBucket
  status: FacetBucket
  provider: FacetBucket
  open_weights: FacetBucket
}

// ── Catalog: configure bridge ─────────────────────────────────────────────────────────────────
export interface ConfigureBridge {
  api_format: BridgeApiFormat
  base_url: string | null
  base_url_source: 'modelsdev_api' | 'bodhi_preset' | 'user_required'
  base_url_requires_substitution: boolean
}

// ── Catalog: providers ────────────────────────────────────────────────────────────────────────
export interface ProviderSummary {
  slug: string
  name: string
  logo_url: string | null
  model_count: number
  rank: number
  api_base_url: string | null
  provider_shape: ProviderShape
  api_format_hint: ApiFormatHint
  capabilities_summary: Capability[]
  pricing_summary: { min_in_per_m: number | null; min_out_per_m: number | null }
}
export interface ProviderListResponse extends PageMeta {
  items: ProviderSummary[]
  facets: { capability: FacetBucket; api_format: FacetBucket }
}
export interface ProviderDetailResponse {
  slug: string
  name: string
  logo_url: string | null
  model_count: number
  env: string[]
  npm: string | null
  doc_url: string | null
  api_base_url: string | null
  provider_shape: ProviderShape
  bridge: ConfigureBridge
}
export interface ListProvidersQuery {
  q?: string
  capability?: Capability | Capability[]
  api_format?: ApiFormatHint | ApiFormatHint[]
  pricing_max?: number
  sort?: 'rank' | 'name' | 'model_count'
  page?: number
  page_size?: number
}

// ── Catalog: models by provider ───────────────────────────────────────────────────────────────
export interface ProviderModelRow {
  model_id: string
  name: string
  caps: Capability[]
  context_limit: number | null
  output_limit: number | null
  pricing: PricingLite
  status: ModelStatus | null
  modalities_in: Modality[]
  modalities_out: Modality[]
}
export interface ProviderModelsResponse extends PageMeta {
  items: ProviderModelRow[]
}

// ── Catalog: global model list ────────────────────────────────────────────────────────────────
export interface ModelLite {
  slug: string
  model_id: string
  name: string
  family: string | null
  context_limit: number | null
  output_limit: number | null
  pricing: PricingLite
  caps: Capability[]
  status: ModelStatus | null
  open_weights: boolean
  modalities_in: Modality[]
  modalities_out: Modality[]
  provider_count: number
  release_date: string | null
  last_updated: string | null
}
export interface ListCatalogModelsQuery {
  q?: string
  capability?: Capability | Capability[]
  modality?: Modality | Modality[]
  pricing_max?: number
  pricing_band?: 'free' | 'low' | 'mid' | 'high'
  context_min?: number
  status?: ('alpha' | 'beta' | 'deprecated' | 'stable') | ('alpha' | 'beta' | 'deprecated' | 'stable')[]
  provider?: string | string[]
  open_weights?: 'open' | 'closed'
  sort?: 'updated' | 'context' | 'price' | 'name' | 'providers'
  page?: number
  page_size?: number
}
export interface ModelsListResponse extends PageMeta {
  items: ModelLite[]
  facets: ModelFacets
}

// ── Catalog: model detail ─────────────────────────────────────────────────────────────────────
export interface ServedBy {
  slug: string
  name: string
  logo_url: string | null
  base_url: string | null
  pricing: PricingLite
}
export interface ModelDetailResponse {
  slug: string
  model_id: string
  name: string
  family: string | null
  status: ModelStatus | null
  reasoning: boolean
  tool_call: boolean
  structured_output: boolean | null
  attachment: boolean
  open_weights: boolean
  temperature: boolean | number | null
  reasoning_options: unknown | null
  context_limit: number | null
  output_limit: number | null
  modalities_in: Modality[]
  modalities_out: Modality[]
  release_date: string | null
  last_updated: string | null
  knowledge_cutoff: string | null
  pricing: Pricing
  license: string | null
  links: unknown | null
  weights: unknown | null
  benchmarks: unknown | null
  served_by: ServedBy[]
  bridge: ConfigureBridge
}
export interface GetCatalogModelQuery {
  /** reserved */
  include?: string
}
```

**Wiring (per existing convention):** in `worker/schemas/catalog.ts`, add the guards, e.g.
```ts
const _providerListGuard: ProviderListResponse = {} as z.infer<typeof providerListResponseSchema>
const _modelsListGuard: ModelsListResponse = {} as z.infer<typeof modelsListResponseSchema>
const _modelDetailGuard: ModelDetailResponse = {} as z.infer<typeof modelDetailResponseSchema>
// …one per response schema. Any drift between published types and zod is a typecheck error.
```

---

## 5. Caching (reuse `worker/cache/store.ts` — the two-tier `CacheStore`, KV hot → D1 warm)

The catalog inverts the HF model: **D1 is the source of truth** (queried directly with FTS5 + column filters), and KV/`CacheStore` is an optional read accelerator for the expensive/stable reads (data-analysis §5.5). Use the existing `CacheStore` (`get`/`put`, injectable clock) and `getSettings(env).cacheTtlMs()` (default 24h) so TTLs stay a config edit, not a rebuild.

| Endpoint | Cache via `CacheStore`? | TTL | Cache key |
|---|---|---|---|
| `GET /api/v1/providers/{slug}` | **yes** — KV→D1 cache-aside | `cacheTtlMs()` (24h) | `catalog:provider:v1:{slug}` |
| `GET /api/v1/providers/{slug}/models` | **yes** (paged) | 24h | `catalog:provider-models:v1:{slug}:{page}:{page_size}:{sort}` |
| `GET /api/v1/models/{slug}/{model_id}` | **yes** — the heaviest read (blobs + served_by + bridge) | 24h | `catalog:model:v1:{slug}:{encodeURIComponent(model_id)}` |
| `GET /api/v1/providers` (list) | optional — only the **unfiltered default page** (`page=1`, default sort, no facet filters) | 1h | `catalog:providers:v1:default` |
| `GET /api/v1/models` (search/list) | **no** — query+facet-cardinality is too high; serve from D1/FTS5 directly | — | — |
| provider logos | **yes** (KV-only, logo bytes) — set during ingestion (data-analysis §5.5) | ingestion-managed | `logo:{slug}` |

**Invalidation:** the ingestion Workflow (data-analysis §6.2 step 8/9) is the only writer; on a sync that touches `(provider_slug, model_id)`, it should `put` the refreshed model/provider entries (or delete the keys so the next read repopulates) — the same cache-aside `put` the cold path uses. The `catalog:providers:v1:default` and per-provider pages should be deleted when `model_count`/membership changes. Bump the `:v1:` key segment on any breaking response-shape change.

---

## 6. Endpoint → design-consumer map (authoritative)

| Endpoint | Page | Consumer (design-prompt §) |
|---|---|---|
| `GET /api/v1/providers` | **B** (mode `'api'`) | List rows (`ApiRow`: rank/logo/name/models-count/format) + sidebar facets (Status·Capability·Pricing·API-Format) — §3 list + sidebar realign; data-analysis §7.1 |
| `GET /api/v1/providers/{slug}` | **B** | Detail-rail header + "Provider meta block" (Env var=`env[]`, Base URL=`api`, Docs=`doc`, npm=`npm`) **and** the Configure-in-Bodhi bridge — §3 meta block + §5 prefill; data-analysis §8 |
| `GET /api/v1/providers/{slug}/models` | **B** | Detail-rail "Models (N)" table (`prov-mrow`: name + ctx + `$in/$out` + caps, Free rule) — §3 Models(N); data-analysis §7.2 |
| `GET /api/v1/models` | **A** (mode `'api-catalog'`) | List + columns (#/Model/Context/Input $/Output $/Capabilities/Providers), the 7 sidebar filter groups with **live facet counts**, `ShellSearch` ⌘K, numbered `usePagination(rows, 8)` — §2a/§2b/§2d; data-analysis §7.4 |
| `GET /api/v1/models/{slug}/{model_id}` | **A** | Per-model detail rail: `DetailHeader` + spec grid (Pricing·Limits·Modalities·Capabilities·Meta) + "Served by (N)" provider list + Configure CTA — §2c; data-analysis §7.3 |

**Cross-link params** (design §1) resolve through the same surface: page-B→page-A `?provider=<slug>` → `GET /api/v1/models?provider=<slug>`; page-A→page-B `?select=<slug>` → `GET /api/v1/providers/{slug}`; Configure `?provider=<slug>&model=<modelId>` is satisfied by the `bridge` on either provider-detail or model-detail.
