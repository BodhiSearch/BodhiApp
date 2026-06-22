# Explore API Models — Catalog Ingestion Data-Format Analysis

**Audience:** the backend engineer building the catalog ingestion pipeline + catalog API inside `api-getbodhi-app`.
**Scope:** how to shallow-clone `anomalyco/models.dev` and `Portkey-AI/models`, re-ingest only changed files since the last sync, normalize the union into a unified D1 catalog (with FTS5 search), and serve it to two BodhiApp Explore pages (API Models, models-first; API Providers, provider-first).
**Status of premise:** The current `api-getbodhi-app` repo does **not** implement any of this yet — see §6. This document specifies the new pipeline, grounded only in the research findings cited inline.

---

## 1. Executive Summary

We are unifying **two independent, free, no-auth source datasets** into one catalog:

1. **models.dev** (`anomalyco/models.dev`) — the **PRIMARY** contract. A TOML-authored, schema-validated catalog of provider + model **metadata**: display names, context/output limits, modalities, capability booleans (attachment/reasoning/tool_call/open_weights), release/knowledge dates, status, and a *shallow* cost block (input/output + a few cache/audio fields). It also defines the **provider** concept we need (id, name, env vars, npm pkg, doc URL, and — critically for BodhiApp — an optional `api` base URL). Source of record: the `providers/` and `models/` TOML trees, the Zod schema in `packages/core/src/schema.ts`, and the generated `api.json`/`models.json`/`catalog.json`. (Cite: `packages/core/src/schema.ts:323-373`; `packages/web/script/build.ts:77-88`.)

2. **Portkey** (`Portkey-AI/models`) — the **FALLBACK / ENRICHMENT** source. A community-maintained pricing+param database (~2,852 priced models / 44 providers in `pricing/`, ~4,106 entries / 52 providers in `general/`). Its value is **pricing depth** — the "hidden dimensions" models.dev omits: cache write vs read (+1h TTL), thinking/reasoning tokens, audio in/out, per-request tool surcharges, video/image-gen pricing, batch pricing, fine-tune pricing, and region- / execution-mode- / context-tier-keyed pricing. It carries **no** model metadata (no context window, no display names, no dates, no modalities). (Cite: Portkey `README.md:62-109`; `openapi.yaml:289-434`.)

**Unified-catalog goal:** one D1 relational schema (`providers`, `models`, `model_pricing`) plus an FTS5 index, where models.dev supplies the **breadth** (metadata + provider taxonomy) and Portkey supplies the **pricing depth**, joined on `(provider-slug, model-id)`. On overlapping fields, **models.dev wins** (it is schema-validated; Portkey is crowd-sourced and only JSON-validity-checked). (Cite: Portkey `CONTRIBUTING.md:150-185`; `scripts/sync-to-gateway.js:1-83`.)

**Re-ingest-changed-files strategy:** store a per-source **git SHA cursor**. On each cron tick, shallow-clone (or fetch) the source repo, `git diff --name-only <last_sha>..HEAD`, parse only the changed TOML/JSON files, upsert affected rows, delete rows whose files were removed, and rebuild the FTS5 rows for touched models. Advance the cursor to the new HEAD. (models.dev's own sync uses git-diff/mtime against per-provider files and emits hourly per-provider PRs — we mirror that diffing approach on the *consumer* side. Cite: `sync.md:1-112`.)

**Two consuming Explore pages** (both reuse one BodhiApp React app parameterized by `MODELS_MODE`; cite: `design/models/bodhi-models-app.jsx:9-13`):
- **API Models** (models-first): global model list/search with facets — capability, modality, pricing band, context size, status.
- **API Providers** (provider-first): provider list (rank/logo/name/models-count) → provider detail (model table with price/caps/ctx) → a **configure** CTA that maps a models.dev provider into a BodhiApp API-model config.

---

## 2. models.dev TOML Source Schema (PRIMARY contract)

models.dev authors data as TOML under **two parallel trees**, validated by a strict Zod schema (unknown keys fail; unique names enforced). (Cite: `packages/core/src/schema.ts:143-373`; strict-error rules at `:130-156`.)

### 2.1 The `models/` vs `providers/` split

| Tree | Path shape | Meaning | Schema | Has cost? | Count |
|---|---|---|---|---|---|
| `providers/<id>/provider.toml` | provider record | A **serving** provider (anthropic, openai, openrouter, groq, …) | `Provider` 323-373 | n/a | — |
| `providers/<id>/models/<model>.toml` | per-provider model | A model **as served by that provider** (has cost, status, provider-overrides, reasoning_options, interleaved) | `ModelBase` 213-271 | **yes** | ~5297 |
| `models/<lab>/<model>.toml` | lab-agnostic model | The **abstract** model (license, links, weights, benchmarks) — NO cost/status/provider/reasoning_options/interleaved | `ModelMetadata` 188-211 | **no** | ~220 |

Key consequence for ingestion: **the serving model (`providers/<id>/models/*.toml`) is the row you want** for the catalog — it carries cost + capabilities + provider lineage. The lab-agnostic `models/<lab>/*.toml` is a metadata *donor* via `base_model` inheritance (license/links/weights/benchmarks). (Cite: `packages/core/src/schema.ts:143-211,245-270`; `220 vs 5297`.)

### 2.2 `provider.toml` fields (`Provider`, schema.ts:323-373)

| Field | Type | Required | Meaning | Example |
|---|---|---|---|---|
| `id` | string | **derived from dir name** (not in file) | provider slug | `anthropic`, `amazon-bedrock` |
| `name` | string | **required** | display name | `"Anthropic"` |
| `env` | string[] (min 1) | **required** | env var names for credentials (google has 3) | `["ANTHROPIC_API_KEY"]` |
| `npm` | string | **required** | npm SDK package | `@ai-sdk/anthropic` |
| `doc` | string (URL) | **required** | docs URL | `https://docs.anthropic.com` |
| `api` | string (base URL) | **OPTIONAL** | API base URL — **the field BodhiApp's configure bridge needs** | `https://api.anthropic.com` |
| `models` | record | (resolved at generate time) | map of model-id → resolved Model | — |

**`api` field refinement rule (important):** `api` is **REQUIRED** for `openai-compatible` / `openrouter` shapes, **OPTIONAL** for `anthropic`/`openai`/`kiro`, and **FORBIDDEN** otherwise. Native providers anthropic/openai/google/groq/cohere have **no** `api` field (the SDK hard-codes the URL). **119 of 145 providers have `api`.** Bedrock's `api` carries an **unresolved `AWS_REGION` placeholder** — store literal, do not resolve. (Cite: `packages/core/src/schema.ts:323-373`; `providers/anthropic/provider.toml`; `providers/amazon-bedrock/models/openai.gpt-5.5.toml:17`.) → This directly drives §8's bridge table; for providers without `api`, BodhiApp must supply the base_url from its own preset table.

### 2.3 Serving model fields (`ModelBase`, schema.ts:213-271)

| Field | Type | Required | Meaning | Example |
|---|---|---|---|---|
| `id` | string | **derived from path** (path-style, dots literal) | model id | `claude-sonnet-4-5`, `gemini-2.5-pro` |
| `name` | string | **required** | display name | `"Claude Sonnet 4.5"` |
| `attachment` | bool | **required** | accepts file attachments | `true` |
| `reasoning` | bool | **required** | reasoning model | `true` |
| `tool_call` | bool | **required** | supports tool/function calling | `true` |
| `open_weights` | bool | **required** | open-weight model | `false` |
| `release_date` | DateString `YYYY-MM(-DD)` | **required** | release date | `2025-09-29` |
| `last_updated` | DateString | **required** | last update | `2025-09-29` |
| `modalities` | `{input:[], output:[]}` enum arrays | **required** | I/O modalities — enum `text/audio/image/video/pdf`; **REPLACED on inherit** (not merged) | `{input:[text,image,pdf], output:[text]}` |
| `limit` | `{context:int, output:int}` | **required** | token limits (`output` optional on lab-metadata variant) | `{context:200000, output:64000}` |
| `family` | enum | optional | model family | — |
| `reasoning_options` | `{toggle?, effort?, budget_tokens?}` | optional | reasoning controls | `{budget_tokens:{min:1024}}` |
| `interleaved` | `true` OR `{<field>}` | optional | interleaved reasoning; object form owns the field key | — |
| `structured_output` | bool | optional | structured output support | — |
| `temperature` | bool/number | optional | temperature support | — |
| `knowledge` | DateString | optional | knowledge cutoff | `2025-01` |
| `status` | enum `alpha/beta/deprecated` | optional | lifecycle status | `deprecated` |
| `experimental` | record (`modes`) | optional | experimental modes — **NOT inherited** (only ~28 files) | — |
| `provider` | table (per-model override) | optional | per-model `npm/api/shape/body/headers` override (lives here, not top-level) | bedrock `api` literal `AWS_REGION` |
| `cost` | `Cost` | optional | pricing (see §2.4) | — |
| `base_model` | path `LAB/MODEL-id` | optional | inheritance pointer (see §2.5) | `anthropic/claude-sonnet-4-5` |
| `base_model_omit` | string[] (dot-paths) | optional | keys to delete after merge | — |

### 2.4 `Cost`, tiers, limit, modalities, reasoning, interleaved (schema.ts:24-139; generate.ts:230-276)

**`Cost` (strict, 72-92)** — values are **USD per MILLION tokens**:

| Field | Required | Meaning |
|---|---|---|
| `input` | **required** | input token cost |
| `output` | **required** | output token cost |
| `reasoning` | optional | reasoning token cost |
| `cache_read` | optional | cached-read cost |
| `cache_write` | optional | cache-write cost |
| `input_audio` | optional | audio input cost |
| `output_audio` | optional | audio output cost |

**`AuthoredCost`** adds `tiers` (and **forbids** `context_over_200k` in source). At generate time, `normalizeCost` (230-276) turns a tier with `size >= 200000` into a synthesized `context_over_200k` block on the **output cost** (`OutputCost`). `CostTier` = `{tier:{type, context, size}}`; duplicate tier sizes **throw**. (Cite: `packages/core/src/schema.ts:24-139,219-230`; `generate.ts:230-276`.)

> **OPEN QUESTION (do not assume schema.ts is the complete cost contract):** live TOMLs (minimax, perplexity) carry `cached_input`, `cached_write`, `citation`, `request` which are **NOT** in the strict `Cost` (72-92). Confirm against `bun validate` before treating the Zod schema as exhaustive. The ingestion parser must **tolerate unknown cost keys** rather than reject the file. (Cite: `packages/core/src/schema.ts:72-92`; `providers/minimax/models/MiniMax-M2.1.toml:12-16`.)

### 2.5 `base_model` inheritance (the join + id-from-path rules)

- `base_model = "LAB/MODEL-id"` is a **path-style pointer**; `base_model_omit` lists **dot-paths** to delete.
- `mergeDeep` (generate.ts 146-165): objects merged; **arrays and primitives replaced**; strips `id/benchmarks/license/links/weights` from the donor; `applyOmit` deletes the omit-paths **after** merge.
- Output has **no** `base_model` field. Missing `base_model` target = **error** (152).
- `id` derives from the **path**, nested path-style, dots literal. **1206 files use `base_model`.**

**Ingestion implication:** to compute a final serving-model row you must **resolve `base_model`** exactly like `generate.ts` does (or ingest the already-resolved `api.json` — see §3). Resolving from raw TOML means you re-implement `mergeDeep` + `applyOmit` + `normalizeCost`. The lower-risk path is to ingest the generated JSON. (Cite: `generate.ts:14-21,91-228`; `providers/google/models/gemini-2.5-pro.toml:1`.)

---

## 3. models.dev Generated API Shape (which file is genuinely models.dev)

The build emits three JSON files (`build.ts:77-88`):

| File | Content | Schema | Use for ingestion? |
|---|---|---|---|
| `api.json` | `Record<providerId, Provider>` with **resolved** models (cost/limit/modalities/capabilities all populated, `base_model` already merged away) | `Provider` (323-337) + resolved `Model` (303-308) + `OutputCost` | **YES — primary ingestion target.** Resolution already done. |
| `models.json` | `Record<modelId, ModelMetadata>` keyed by path id (e.g. `anthropic/claude-sonnet-4-5`); adds `license/links/weights/benchmarks`; **NO cost, NO provider**; `limit.output` optional | `ModelMetadata` (188-209) | Optional metadata enrichment (benchmarks, license). |
| `catalog.json` | `{ models: Models, providers: Providers }` — both of the above combined | server.ts:114-122 | Single-fetch convenience = api.json ∪ models.json. |

`api.json` example: provider `anthropic`, model `claude-sonnet-4-5` → `cost {input:3, output:15, cache_read:0.3, cache_write:3.75}`, `limit {context:200000, output:64000}`, `modalities {input:[text,image,pdf], output:[text]}`, `reasoning_options {budget_tokens:{min:1024}}`. (Cite: `packages/core/src/schema.ts:213-337`; `providers/anthropic/models/claude-sonnet-4-5.toml`.)

`models.json` example: `anthropic/claude-sonnet-4-5` includes `benchmarks` (e.g. name `SWE-Bench Pro`, score `43.6`, metric `resolve rate`, dataset `public`). (Cite: `packages/core/src/schema.ts:188-211`; `models/anthropic/claude-sonnet-4-5.toml`.)

**Is anything in the clone an OpenRouter dump, not models.dev?** models.dev *includes* `openrouter` as one provider (its `provider.toml` requires an `api` URL because its shape is `openrouter`). That provider's `models/*.toml` are still models.dev-authored TOML under the same strict schema — **not** a verbatim OpenRouter API dump. There is no separate "OpenRouter dump file" inside the models.dev repo; the `openrouter` rows are first-class models.dev catalog entries. The generated `api.json`/`models.json`/`catalog.json` are the genuine models.dev artifacts. **Logos are NOT in any JSON** — they are served as `GET /logos/<PROVIDERID>.svg` with `default.svg` fallback. (Cite: `packages/function/src/worker.ts:66-133`; `packages/web/src/server.ts:114-122`.)

**Recommendation:** ingest **`api.json`** (resolved serving models, includes cost) as the spine, optionally left-join `models.json` for benchmarks/license/links/weights. Fetch `logos/<id>.svg` separately and cache in KV (see §5). This avoids re-implementing `generate.ts`.

> NOTE: the public Worker rewrites `_`-prefixed keys and synthesizes a `model-schema.json`. If ingesting raw TOML instead of JSON, account for the `_`-prefix rename. (Cite: `packages/function/src/worker.ts:66-112`.)

---

## 4. Portkey Fallback Schema + Gap List + Join Key

### 4.1 Layout

Two parallel dirs, **one JSON file per provider**, each a flat map `modelId → config`:

- `pricing/<provider>.json` — 44 files, ~2,852 entries. Also served live at `https://configs.portkey.ai/pricing/<provider>.json`.
- `general/<provider>.json` — 52 files, ~4,106 entries. **Carries top-level scalar keys `name` and `description` mixed in with model objects** — when iterating, filter to `typeof value === "object"`.

The file set is **not symmetric**: `general/` has extras with no pricing (fireworks, github, lambda, replicate, sambanova, voyage, nscale, modal, inference-net, upstage, lemonfox-ai, open-ai, nomic-ai); `pricing/` has extras with no general (monsterapi, predibase, sagemaker, workers-ai, nomic). (Cite: `pricing/*.json`, `general/*.json`; `general/cohere.json:2-3`.)

**Every provider file has a `default` entry** (zero prices + the canonical `calculate` formula) used as the provider-wide fallback. (Cite: `pricing/anthropic.json:2-71`; `CONTRIBUTING.md:80-103`.)

### 4.2 CRITICAL unit gotcha

Portkey prices are in **CENTS PER TOKEN**, not dollars, not per-million.
`costDollars = (tokens * price) / 100`. Example: `0.003` = `$0.03/1K` = **`$30/1M`**.
To cross-ref a models.dev `$/1M` value: `price_in_cents = (dollars_per_million / 1_000_000) * 100`. (Cite: `README.md:62-74`; `CONTRIBUTING.md:34-43`.)

### 4.3 Per-model pricing schema (`pricing_config`)

```
model.pricing_config = {
  pay_as_you_go: { ...leaf {price} },
  batch_config?:   { request_token, response_token, cache_read_input_token },
  finetune_config?:{ pay_per_token?, pay_per_hour? },
  calculate?:      { request, response }  // recursive AST, usually inherited from `default`
  currency: "USD"
}
```
`pay_as_you_go` leaves: `request_token`, `response_token`, `cache_write_input_token`, `cache_read_input_token`, `request_audio_token`, `response_audio_token`, `cache_read_audio_input_token`, `additional_units{}`, `image{}`. Each leaf = `{ price: <number> }`. (Cite: `README.md:78-109`; `openapi.yaml:289-349`; `pricing/anthropic.json:84-113`.)

### 4.4 The explicit GAP list — what Portkey adds over models.dev

| Dimension | Portkey shape | models.dev coverage | Cite |
|---|---|---|---|
| **Cache write vs read (+1h TTL)** | `cache_write_input_token`, `cache_read_input_token`, `additional_units.cache_write_1h` | has `cache_read`/`cache_write` but no TTL variant | `pricing/anthropic.json:92-104,610-639` |
| **Thinking/reasoning tokens** | `additional_units.thinking_token` | `cost.reasoning` exists but rarely populated | `README.md:25,119`; `openapi.yaml:186-188` |
| **Audio in/out tokens** | `request_audio_token`, `response_audio_token`, `cache_read_audio_input_token` (+ as `additional_units`) | `input_audio`/`output_audio` only | `openapi.yaml:318-323`; `README.md:128-129` |
| **Per-request tool surcharges** | `additional_units.{web_search,file_search,search,enterprise_web_search,maps,routing_units}`; Perplexity tiered `web_search_{low,medium,high}_context` | **absent** | `README.md:114-142` |
| **Video pricing** | `video_seconds`, `video_duration_seconds_*`, `input_video_{essential,standard,plus}` | **absent** | `README.md:122-134` |
| **Image-gen pricing** | nested `image{quality}{size}{price}`, `megapixels`, `generated_images`, `image_1k`, `image_token`, `default_steps`, `default_sample_count` | **absent** | `README.md:159-167`; `openapi.yaml:369-386` |
| **Batch pricing** | `batch_config` (~50% text, ~20% off embeddings) | **absent** | `README.md:170-202`; `openapi.yaml:351-367` |
| **Fine-tune pricing** | `finetune_config.{pay_per_token,pay_per_hour}` | **absent** | `openapi.yaml:427-434` |
| **Region + execution-mode + context-tier pricing** | **`custom_pricing`** (OUT-OF-SPEC, on 110 OpenAI + 39 Vertex models): `context_tier_map{0:'lte-200k',200000:'gt-200k'}` + `regions{<region>}.execution_modes{standard\|batch\|flex\|priority}.[context_tiers.<tier>.]pricing_config` | **absent** | `pricing/openai.json gpt-4o-2024-08-06`; `pricing/vertex-ai.json` |
| **Context-threshold via id suffixes (Google)** | separate keys `…-gt-128k` / `…-lte-128k` (and `-gt-200k`/`-lte-200k`) | single model row | `pricing/google.json` keys |

**`calculate` AST:** `{operation: sum\|multiply, operands:[...]}` referencing `input_tokens`, `output_tokens`, `cache_*_tokens`, `rates.*`, etc. Usually only on `default`, inherited. (Cite: `openapi.yaml:388-425`.)

### 4.5 General (capability) config schema

`general/<provider>.json` model = `{ params:[{key,defaultValue?,minValue?,maxValue?,type?,options?,skipValues?}], type:{primary, supported:[]}, messages:{options:[roles]}, removeParams?:[], disablePlayground?:bool, isDefault?:bool }`.
`type.primary` ∈ chat/text/embedding/image/image_generation/audio/video/moderation/rerank/responses.
`type.supported` ∈ tools/image/vision/cache_control/audio/video/text/code/doc/pdf/mime_type/messages/responses/image_generation.
Param keys include `max_tokens`, `max_completion_tokens`, `temperature`, `top_p`, `top_k`, `n`, `stop`, `seed`, `frequency_penalty`, `presence_penalty`, `logit_bias`, `response_format`, `tool_choice`, `parallel_tool_calls`, `reasoning`, `reasoning_effort`, `thinking`, `verbosity`, `safe_prompt`, `candidateCount`, `stream`. (Cite: `openapi.yaml:436-518`; `CONTRIBUTING.md:111-149`.)
**The only capacity field Portkey has is max OUTPUT tokens** — indirectly, `params[key=max_tokens].maxValue` (usually `null`).

### 4.6 What models.dev HAS that Portkey LACKS

A grep over both Portkey dirs for `context_length|context_window|knowledge|cutoff|max_input_tokens|display_name|modalities|limit` returns **nothing**. Portkey has **no** context window/max input, **no** display name (id only), **no** knowledge cutoff, **no** release date, **no** modality lists, **no** open-weight/license flags, **no** provider-of-origin lineage. (Cite: grep over `pricing/`/`general/`.)

→ **Net:** models.dev = metadata breadth + provider taxonomy; Portkey = pricing + param depth. They are complementary, not redundant.

### 4.7 Join key

`(provider-slug, model-id)`:
- **provider-slug**: lowercase, hyphenated, exact filename stem — `openai`, `anthropic`, `google`, `x-ai`, `azure-openai`, `bedrock`, `together-ai`, `fireworks-ai`, `vertex-ai`.
- **model-id**: provider's **exact API string** — OpenAI dated (`gpt-4o-2024-08-06`), Anthropic dated + aliases (`claude-sonnet-4-5-20250929` AND `claude-sonnet-4-5` AND `claude-sonnet-4-0`), Bedrock dotted/versioned (`anthropic.claude…`, `ai21.jamba-1-5-mini-v1:0`, `amazon.nova-2-lite-v1:0`).

**Join caveats:** (a) strip Google `-gt-128k`/`-lte-128k` context suffixes before joining; (b) multiple Portkey keys (dated + `-latest` + `-0`) can map to one models.dev model; (c) fall back to the provider's `default` entry when an id is absent; (d) **provider-slug mismatch** — models.dev uses `amazon-bedrock`, Portkey uses `bedrock`; models.dev `xai` vs Portkey `x-ai`; models.dev `google` vs Portkey `google`/`vertex-ai`. A **slug-alias map** is required (see §5/§9). (Cite: `openapi.yaml:79-95`; `pricing/anthropic.json`; `README.md:42-58`.)

### 4.8 Portkey API shape (if pulling live instead of cloning)

1. Bulk static: `GET https://configs.portkey.ai/pricing/<provider>.json` → entire provider pricing file.
2. Single model: `GET https://api.portkey.ai/model-configs/pricing/<provider>/<model>` and `…/general/<provider>/<model>`. 404 `{error:'Model not found'}` when missing. No auth.

> **WARNING:** `openapi.yaml` does **not** document the `custom_pricing`/`regions`/`execution_modes`/`context_tiers` extension or `cache_read_audio_input_token`. **Parse the raw cloned files, not the spec**, to capture region/throughput/context-tier pricing. (Cite: `README.md:30-60`; `openapi.yaml:37-96,194-237`.)

---

## 5. Proposed Unified Catalog Data Model (D1 + FTS5 + KV)

Design goal: models.dev spine (resolved `api.json`), Portkey enrichment joined on `(slug, model-id)`, lossless storage of pricing depth via a JSON column for the long-tail dimensions, with typed columns for the facets the two Explore pages filter/sort on.

### 5.1 `providers`

| Column | Type | Source | Notes |
|---|---|---|---|
| `slug` | TEXT PK | models.dev dir id | canonical slug (`anthropic`, `amazon-bedrock`) |
| `name` | TEXT NOT NULL | models.dev `name` | display name |
| `doc_url` | TEXT | models.dev `doc` | |
| `npm` | TEXT | models.dev `npm` | |
| `env_vars` | TEXT (json array) | models.dev `env` | credential env var names |
| `api_base_url` | TEXT NULL | models.dev `api` | **null for native providers** — bridge fills from BodhiApp preset (§8) |
| `provider_shape` | TEXT | derived from `api` refinement | `openai`/`openai-compatible`/`anthropic`/`openrouter`/`kiro`/native |
| `logo_kv_key` | TEXT NULL | fetched `logos/<slug>.svg` | KV pointer |
| `portkey_slug` | TEXT NULL | slug-alias map | Portkey-side slug if it differs (`bedrock`, `x-ai`, `vertex-ai`) |
| `model_count` | INT | computed | for provider-list ranking |
| `source_sha` | TEXT | ingest cursor | provenance |
| `updated_at` | INT | ingest | |

### 5.2 `models`

| Column | Type | Source | Notes |
|---|---|---|---|
| `id` | TEXT | composite (`provider_slug` + `model_id`) | PK = `(provider_slug, model_id)` |
| `provider_slug` | TEXT NOT NULL FK | models.dev | |
| `model_id` | TEXT NOT NULL | models.dev path id | exact API string |
| `name` | TEXT NOT NULL | models.dev `name` | display |
| `family` | TEXT NULL | models.dev | |
| `status` | TEXT NULL | models.dev `status` | alpha/beta/deprecated; **facet** |
| `attachment` | INT(bool) | models.dev | capability **facet** |
| `reasoning` | INT(bool) | models.dev | capability **facet** |
| `tool_call` | INT(bool) | models.dev | capability **facet** |
| `structured_output` | INT(bool) NULL | models.dev | capability **facet** |
| `open_weights` | INT(bool) | models.dev | |
| `temperature` | TEXT NULL | models.dev | bool/number |
| `modalities_in` | TEXT (json) | models.dev | enum array — **facet** |
| `modalities_out` | TEXT (json) | models.dev | enum array — **facet** |
| `context_limit` | INT | models.dev `limit.context` | **facet (context band)** + sort |
| `output_limit` | INT NULL | models.dev `limit.output` | |
| `release_date` | TEXT NULL | models.dev | `YYYY-MM(-DD)` |
| `last_updated` | TEXT NULL | models.dev | sort (Updated column) |
| `knowledge_cutoff` | TEXT NULL | models.dev | |
| `reasoning_options` | TEXT (json) NULL | models.dev | toggle/effort/budget_tokens |
| `license` | TEXT NULL | models.json | enrichment |
| `links` | TEXT (json) NULL | models.json | enrichment |
| `weights` | TEXT (json) NULL | models.json | enrichment |
| `benchmarks` | TEXT (json) NULL | models.json | enrichment |
| `has_pricing` | INT(bool) | computed | whether any priced row exists |
| `source_sha` | TEXT | cursor | |
| `updated_at` | INT | ingest | |

### 5.3 `model_pricing`

Holds the union of models.dev cost + Portkey pricing depth. Typed columns for the **common** dimensions (so the Explore "pricing band" facet can use SQL), plus a `pricing_json` blob for the long-tail (custom_pricing, image/video maps, calculate AST, batch/finetune).

| Column | Type | Source | Notes |
|---|---|---|---|
| `provider_slug` | TEXT FK | | PK part |
| `model_id` | TEXT FK | | PK part |
| `currency` | TEXT | both | `USD` |
| `unit` | TEXT | normalized | store everything as **USD per 1M tokens** (convert Portkey cents/token → $/1M) |
| `input_per_m` | REAL NULL | models.dev `cost.input` ∥ Portkey `request_token` | models.dev wins on conflict |
| `output_per_m` | REAL NULL | models.dev `cost.output` ∥ Portkey `response_token` | |
| `cache_read_per_m` | REAL NULL | models.dev `cache_read` ∥ Portkey `cache_read_input_token` | |
| `cache_write_per_m` | REAL NULL | models.dev `cache_write` ∥ Portkey `cache_write_input_token` | |
| `cache_write_1h_per_m` | REAL NULL | Portkey `additional_units.cache_write_1h` | gap-fill |
| `reasoning_per_m` | REAL NULL | models.dev `cost.reasoning` ∥ Portkey `thinking_token` | gap-fill |
| `input_audio_per_m` | REAL NULL | both | |
| `output_audio_per_m` | REAL NULL | both | |
| `context_over_200k_json` | TEXT NULL | models.dev `OutputCost.context_over_200k` / Portkey `custom_pricing.context_tier_map` | tiered output |
| `batch_json` | TEXT NULL | Portkey `batch_config` | gap-fill |
| `finetune_json` | TEXT NULL | Portkey `finetune_config` | gap-fill |
| `tool_surcharges_json` | TEXT NULL | Portkey `additional_units.{web_search,…}` | gap-fill |
| `image_pricing_json` | TEXT NULL | Portkey `image{}` | gap-fill |
| `video_pricing_json` | TEXT NULL | Portkey video fields | gap-fill |
| `custom_pricing_json` | TEXT NULL | Portkey `custom_pricing` (region/exec-mode/tier) | OUT-OF-SPEC; parse raw |
| `calculate_json` | TEXT NULL | Portkey `calculate` AST | |
| `pricing_source` | TEXT | `modelsdev`/`portkey`/`both` | provenance per row |
| `portkey_default_used` | INT(bool) | | true if filled from Portkey `default` fallback |

**Pricing-band facet derivation:** compute a coarse band from `input_per_m`+`output_per_m` (e.g. free / low / mid / high) either as a generated column or in the query; the design's API-mode pricing filter is a `$0–$20 /M` range slider, so expose the raw `input_per_m`/`output_per_m` for range filtering and `Free` when both are 0. (Cite: `design/models/models-filters.jsx:51-58`; `design/models/models-detail.jsx:195-247` `fmtPrice`.)

### 5.4 FTS5 index

`models_fts` (FTS5, external-content over `models`) indexing: `name`, `model_id`, `provider_slug`, provider `name`, `family`, flattened `modalities_in`/`modalities_out`, and a capability keyword bag (e.g. `tool_call reasoning vision attachment`). This backs the global model search box in API-Models mode and provider search. Rebuild FTS rows only for models touched in a sync (see §6). FTS5 already part of the planned Cloudflare stack (the project intends D1+FTS5 per the memory note; not yet built in repo — §6).

### 5.5 KV (logos + hot reads)

- `logo:<slug>` → provider SVG (fetched from models.dev `GET /logos/<slug>.svg`, fallback `default.svg`). Logos are not in any JSON. (Cite: `worker.ts:118-133`.)
- Reuse the existing two-tier cache-aside pattern (KV hot → D1 warm) for model-detail responses. (Cite existing impl: `worker/cache/store.ts:18-50`.)

---

## 6. Ingestion Pipeline Design

### 6.1 What already exists in `api-getbodhi-app` vs what is NEW

**Reality check (cite: backend-repo-state findings):** the current repo is a **live, enriching HuggingFace GGUF proxy** — NOT a models.dev/Portkey catalog. A grep for `models.dev|portkey|workflow|cron|scheduled|fts5|git clone|shallow|ingest|/orgs|taxonomy` returns **zero** matches. (Cite: `docs/claude-plans/20260621-api-models-local-explore.md:1-10`.)

| Capability | Status today | This design |
|---|---|---|
| Cloudflare Worker (Hono + zod-openapi) serving `/api/` + `/ui/` | **EXISTS** | reuse |
| D1 binding `DB` (dev/prod) | **EXISTS** (single `model_cache` table: key/data/fetched_at/expires_at) | **NEW** typed `providers`/`models`/`model_pricing` + FTS5 |
| KV binding `KV` | **EXISTS** (hot cache) | reuse for logos + detail cache |
| Cache-aside (KV→D1→origin) | **EXISTS** (single-model path) | reuse pattern |
| Service seam `getService(env)` Real/Fake via `USE_FAKE_JOBS` | **EXISTS** | reuse for ingestion fakes in tests |
| Workflows / Queues / Cron triggers / R2 | **NONE in wrangler.jsonc** | **NEW** — must be added |
| FTS5 search index | **NONE** | **NEW** |
| provider-list / provider-detail / taxonomy / orgs endpoints | **NONE** (provider is a derived HF-author display string) | **NEW** |
| git shallow-clone / diff-since-sync ingestion | **NONE** | **NEW** |

The v1 plan **explicitly deferred** all search-index/ingest infrastructure until HF rate limits forced it. (Cite: `docs/claude-plans/20260621-api-models-local-explore.md:18-34`.) This design is that deferred build.

> **Cloudflare git-clone caveat:** Workers/Workflows cannot run `git` natively. Two viable shapes: (a) **GitHub API tarball / per-file fetch + a stored HEAD SHA** (use the GitHub "compare two commits" API to get changed file paths since the cursor, then fetch only those raw files); or (b) a separate CI job (GitHub Actions) does the clone+diff and pushes normalized rows via an authenticated ingest endpoint. **(a) is preferred** — fully self-contained in the Worker/Workflow, no external runner. "Shallow clone" in this design = GitHub compare-API + raw-file fetch, not literal `git`. (See §9 risks.)

### 6.2 Cron + Workflow shape

```
Cron trigger (hourly, mirrors models.dev's hourly sync cadence — sync.md:1-112)
  └─> enqueue IngestWorkflow (one instance per source: models.dev, portkey)

IngestWorkflow (per source):
  step 1  read cursor:   SELECT last_sha FROM sync_state WHERE source = ?
  step 2  resolve HEAD:  GET repo HEAD sha (GitHub API)
  step 3  diff:          GitHub compare API last_sha..HEAD -> changed/removed file paths
                         (first run: full file list)
  step 4  fetch:         raw-fetch only changed files (TOML for models.dev, JSON for portkey)
  step 5  parse+normalize:
            models.dev: prefer api.json (resolved). If diffing raw TOML, resolve base_model
                        (mergeDeep + applyOmit + normalizeCost) per generate.ts.
            portkey:    parse raw pricing/<p>.json + general/<p>.json (NOT openapi.yaml);
                        cents/token -> $/1M; capture custom_pricing.
  step 6  upsert:        providers / models / model_pricing (models.dev wins on overlap;
                         portkey fills gaps + default fallback)
  step 7  prune:         delete rows whose source file was removed in the diff
  step 8  reindex:       rebuild models_fts rows for touched (provider_slug, model_id)
  step 9  logos:         for new/changed providers, fetch logos/<slug>.svg -> KV
  step 10 advance:       UPDATE sync_state SET last_sha = HEAD, synced_at = now
```

`sync_state` table: `source TEXT PK, last_sha TEXT, synced_at INT, last_status TEXT, changed_count INT`.

**Idempotency:** all writes are upserts keyed by PK; a re-run on the same SHA is a no-op. **Failure handling:** do not advance the cursor unless steps 5–9 succeed for the batch; a partial failure re-diffs the same range next tick. Use the Workflow step retry/checkpoint semantics so a mid-run failure resumes rather than re-fetching everything.

**Why diff-since-SHA (not mtime):** the consumer has no filesystem mtime; the SHA cursor is the durable, monotonic, source-authoritative position. models.dev itself emits per-provider PRs hourly, so per-tick diffs are small. (Cite: `sync.md:1-112`.)

### 6.3 Parsing notes (parser must tolerate, not reject)

- models.dev: unknown cost keys (`cached_input`/`citation`/`request`) appear in live TOML but not in strict `Cost` — **store in `pricing_json`, do not throw** (§2.4 open question).
- Portkey `general/*.json`: skip top-level `name`/`description` scalars.
- Portkey `custom_pricing`: present on 110 OpenAI + 39 Vertex models, **not in openapi.yaml** — parse raw.
- Portkey Google context-suffix keys (`-gt-128k`): collapse to base model id; keep tier data in `context_over_200k_json`/`custom_pricing_json`.

---

## 7. Catalog API Surface (the two Explore pages)

All under `/api/v1/`. Reuse the existing zod-openapi + service-seam conventions; publish wire types via `@bodhiapp/reference-api-types` (the existing types-only package). (Cite existing: `packages/api-types/src/index.ts`; `apps/api-getbodhi-app/worker/app.ts:26-30`.)

### 7.1 Provider list — `GET /api/v1/providers`

Maps to **API Providers mode** list (rank/logo/name/models-count + Status/Capability/Pricing/API-Format sidebar). (Cite: `design/models/models-rows.jsx:114-137`; `design/models/models-filters.jsx:51-74`.)

Response item:
```
{ slug, name, logo_url, model_count, api_base_url, provider_shape,
  api_format_hint,            // OpenAI/Anthropic/Gemini/... for the API-Format facet + bridge
  capabilities_summary[],     // union of capability flags across its models (facet)
  pricing_summary{min_in,min_out}, // for the pricing range facet
  rank }                      // by model_count desc (design uses #rank)
```
Query: `?capability=&api_format=&pricing_max=&q=&page=&page_size=`. (Design's API-mode pagination = 5/page, unit `providers`. Cite: `design/models/models-main.jsx:114-117`.)

### 7.2 Models by provider — `GET /api/v1/providers/{slug}/models`

Maps to **API provider detail rail** "Models (N)" table: each row `{name, caps[], ctx, in, out}` with `fmtPrice` (Free when in=0&&out=0). (Cite: `design/models/models-detail.jsx:195-247`; data shape `design/models/bodhi-models-data.js:183-248`.)

Response item:
```
{ model_id, name, caps[],            // derived from capability booleans
  context_limit, output_limit,
  pricing{ input_per_m, output_per_m, cache_read_per_m, cache_write_per_m, ... },
  status, modalities_in[], modalities_out[] }
```

### 7.3 Model detail — `GET /api/v1/models/{slug}/{model_id}`

Full unified row: models.dev metadata + merged pricing (typed + json blobs) + benchmarks/license/links (models.json) + Portkey gap dimensions. Cache-aside via KV→D1. Used for both the model-first detail and provider-detail drill-in.

### 7.4 Global model search/list — `GET /api/v1/models`

Maps to **API Models mode** (models-first) with facets. Backed by FTS5 (`q`) + SQL filters.

Query params → facet mapping:
| Param | Facet | Backing column |
|---|---|---|
| `q` | free-text search | `models_fts` |
| `capability` (multi: tool_call/reasoning/vision/structured/attachment) | Capability chips | `tool_call`,`reasoning`,`structured_output`,`attachment`,`modalities_in` |
| `modality` (multi: text/audio/image/video/pdf) | Modality | `modalities_in`/`modalities_out` |
| `pricing_max` / `pricing_band` | Pricing range slider ($0–$20/M) | `model_pricing.input_per_m`,`output_per_m` |
| `context_min` | Context size band | `context_limit` |
| `status` (alpha/beta/deprecated/stable) | Status | `status` |
| `provider` (multi) | Provider filter | `provider_slug` |
| `sort` (updated/context/price/name) | column sort | resp. columns |
| `page`,`page_size` | pagination | — |

Response: `{ items:[modelDetail-lite], facets:{capability:{...counts}, modality:{...}, status:{...}, provider:{...}}, page, page_size, total }`. Facet counts let the sidebar show live counts. (Design's API-mode facets are currently display-only in the mock — wire them for real here. Cite: open question in current-bodhi-design.)

---

## 8. The models.dev-provider → BodhiApp-config Bridge

BodhiApp's configure flow is a serde-tagged `ApiModelRequest` on `api_format`. The backend treats `(api_format, base_url)` as **free-form** and **live-validates** by calling `fetch_models` before persist — so the catalog only needs to supply good **defaults**; the user always supplies the **API key**, and the catalog can pre-fill model ids. (Cite: `crates/services/src/models/model_objs.rs:1299-1393`; `crates/services/src/models/api_model_service.rs:93-379`.)

### 8.1 `api_format` enum (BodhiApp)

6 lowercase variants: `openai`, `openai_responses`, `anthropic`, `anthropic_oauth`, `gemini`, `llm_liberty_oauth`. `supports_chat_completions` is true only for `openai`, `anthropic`, `anthropic_oauth`. (Cite: `crates/services/src/models/model_objs.rs:744-775`.)

### 8.2 Mapping table (models.dev provider/shape → BodhiApp `(api_format, base_url)`)

| models.dev provider/shape | BodhiApp `api_format` | `base_url` source | Notes |
|---|---|---|---|
| `anthropic` (native, no `api`) | `anthropic` | BodhiApp preset `https://api.anthropic.com/v1` (models.dev `api` is null) | alt: `anthropic_oauth`, `llm_liberty_oauth` |
| `openai` (native, no `api`) | `openai` (or `openai_responses`) | BodhiApp preset `https://api.openai.com/v1` | |
| `google` (native, no `api`) | `gemini` | BodhiApp preset `https://generativelanguage.googleapis.com/v1beta` | not chat-completions |
| `openrouter` (shape requires `api`) | `openai` | **use models.dev `api` URL** | OpenAI-compatible |
| any `openai-compatible` (groq, x-ai/xai, mistral, together, …) | `openai` | **use models.dev `api` URL** | not in BodhiApp preset; URL comes from catalog |
| native w/o `api` and no BodhiApp preset | `openai` (best-effort) | **user must supply** | surface as "base URL required" |

models.dev's `api` field (when present) is the authoritative base_url; when absent (native providers), fall back to BodhiApp's frontend preset table. BodhiApp has **no backend preset table** — presets are frontend-only constants. (Cite: `crates/bodhi/src/components/api-models/providers/constants.ts:15-84`; `crates/services/src/models/api_model_service.rs:330-352`.)

### 8.3 What the catalog can pre-fill vs what the user must supply

| Field | Catalog can supply? | Source |
|---|---|---|
| `api_format` | yes (mapping above) | derived from provider/shape |
| `base_url` | yes for `api`-bearing + preset providers; **no** for unknown openai-compatible | models.dev `api` ∥ preset |
| `models` (ids to expose) | yes — list from `providers/{slug}/models` | catalog |
| `prefix` / `forward_all_with_prefix` | UI default | — |
| **`api_key`** | **NO — user always supplies** (action keep/set, max 4096) | user |
| `extra_headers`/`extra_body` | only for `anthropic_oauth` | UI |

**Hard constraint:** `create_default` calls `fetch_models` **before** persist, so a bad key/url yields an `AiApi` error and the config is rejected. The bridge's defaults must produce a URL that actually answers `fetch_models` with the user's key. `api_format` is **immutable on edit** (`ApiFormatImmutableOnEdit`) — delete+recreate to switch. (Cite: `crates/routes_app/src/models/api/routes_api_models.rs:15-438`; `crates/services/src/models/api_model_service.rs:126-172`.)

---

## 9. Open Questions / Risks

1. **No native `git` in Workers.** "Shallow clone + diff since SHA" must be implemented via GitHub compare API + raw-file fetch (preferred, self-contained) or an external CI runner that pushes rows. Validate GitHub API rate limits for hourly diffs of two repos (authenticated token raises limits). (Cite: backend-repo-state — no clone/cron/workflow exists today.)

2. **models.dev cost contract may be incomplete.** Strict `Cost` (schema.ts:72-92) omits `cached_input`/`cached_write`/`citation`/`request` that appear in live TOML (minimax, perplexity). Confirm via `bun validate`; until then the parser must **tolerate unknown cost keys** (store in `pricing_json`). (Cite: `packages/core/src/schema.ts:72-92`; `providers/minimax/models/MiniMax-M2.1.toml:12-16`.)

3. **Ingest resolved JSON vs raw TOML.** Ingesting `api.json` avoids re-implementing `generate.ts` (mergeDeep/applyOmit/normalizeCost) but couples us to the published artifact and its `_`-prefix Worker rewrite. Ingesting raw TOML is more work but lets us diff at the per-file granularity the SHA-cursor wants. **Recommendation:** diff raw files to know *what changed*, but for models.dev re-derive the resolved row (or re-pull the relevant slice of `api.json`). Decide and document. (Cite: `build.ts:77-88`; `worker.ts:66-112`; `generate.ts:91-228`.)

4. **Provider-slug divergence between sources.** models.dev `amazon-bedrock`/`xai`/`google` vs Portkey `bedrock`/`x-ai`/`google`+`vertex-ai`. Requires a maintained **slug-alias map**; a missing alias silently drops Portkey pricing. (Cite: §4.7; `general/bedrock.json`.)

5. **Portkey `custom_pricing` is out-of-spec.** Region/exec-mode/context-tier pricing exists in raw files but not `openapi.yaml`. Must parse raw; the single-model live API may return a flattened view (unverified — curl before relying). (Cite: Portkey openQuestions; `openapi.yaml`.)

6. **Context-tier pricing needs a context value Portkey lacks.** Google/Vertex price by 128k/200k threshold but Portkey has no context window. To resolve the correct tier we need models.dev `limit.context` — confirm every tiered model has a models.dev counterpart, else the tier is unresolvable. (Cite: Portkey openQuestions.)

7. **Crowd-sourced Portkey freshness.** Only JSON-validity-checked; treat as best-effort and prefer models.dev on overlap (already the policy). Record `pricing_source`/`portkey_default_used` for transparency. (Cite: `CONTRIBUTING.md:150-185`.)

8. **One models.dev provider deviates** (name/npm/env count 144 vs 145). The parser must not assume every provider satisfies the required contract — log + skip-or-degrade rather than fail the whole sync. (Cite: models.dev-toml-schema open question.)

9. **Bedrock `api` carries unresolved `AWS_REGION`.** Store literal; the bridge (§8) cannot use it as a base_url without substitution — flag bedrock as "user must supply region/url". (Cite: `providers/amazon-bedrock/models/openai.gpt-5.5.toml:17`.)

10. **API-mode facets currently display-only in the design mock.** Confirm the real page wires capability/pricing/api-format/modality filters to the new `/api/v1/models` facets (this doc assumes yes). (Cite: current-bodhi-design open question.)

11. **New page wiring.** Open: is API-Models a new `MODELS_MODE` value in the existing single app (+ new row renderer + FILTERS entry + DetailBody branch) or a separate page? The architecture favors a new mode. (Cite: `design/models/bodhi-models-app.jsx:9-13`; current-bodhi-design open question.)
