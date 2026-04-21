# 20 · api.getbodhi.app public APIs
*New endpoints hosted at `https://api.getbodhi.app/v1/*`. Public, non-user-specific.*

These are all **NEW**. This doc is the primary deliverable of this spec pass.

## 1. Scope
api.getbodhi.app is the **curated catalog + enrichment layer**. It answers questions that are the same for every user:
- "What AI providers exist and what's their default config?" (for Create-API-Model form + unconnected rows)
- "What's the specialization taxonomy and which benchmark does each map to?" (for the Specialization filter)
- "Who's #N on HumanEval right now?" (for ranked mode)
- "What's trending on HuggingFace this week?" (for the `↑ Trending` browse cut)
- "Enrich this HF repo with normalized license / capability tags / cost (if API) / size." (for catalog row annotations)

It does NOT answer user-specific questions (my aliases, my files, my API keys). Those are bodhi-server.

## 2. Conventions
- Base path: `/v1`.
- Auth: **TBD** (see README § "Out of scope"). Treat all endpoints as unauthenticated-read for this pass. Mutations do not exist (this is a read-only catalog surface for the frontend).
- All responses use envelope `{ data, total, page, page_size }` matching bodhi-server's style. Cursor pagination is avoided — total counts fit in the DB.
- `ETag` + `Cache-Control: public, max-age=3600` on every response. Frontends should respect `If-None-Match`.
- Error envelope mirrors bodhi-server (`BodhiErrorResponse`) for shared TS typings.

## 3. Endpoints at a glance
| Path | Method | Purpose |
|---|---|---|
| `/v1/providers` | GET | List all AI providers (the directory) |
| `/v1/providers/{code}` | GET | Get one provider + its offered models + cost |
| `/v1/api-formats` | GET | Extended formats list for Create-API-Model form |
| `/v1/specializations` | GET | Specialization taxonomy + benchmark mapping |
| `/v1/benchmarks` | GET | All supported benchmarks (HumanEval, GPQA, MMMU, …) with descriptions |
| `/v1/leaderboards/{benchmark}` | GET | Ranked list of models for a benchmark |
| `/v1/repos/trending` | GET | Trending HF repos (curated subset) |
| `/v1/repos/new` | GET | New-launch HF repos (curated subset) |
| `/v1/repos/{owner}/{name}` | GET | Enriched metadata for one HF repo (license-normalized, capability-tagged, etc.) |
| `/v1/repos/search` | GET | Full-text search over the mirrored HF catalog (curated) |
| `/v1/licenses` | GET | Normalized license taxonomy (Apache-2 / MIT / Llama / Gemma / CC-BY / Proprietary) |

Detailed specs below.

---

## 4. Providers (Bodhi Directory)
### `GET /v1/providers` — list all providers
Drives the `provider-unconnected` rows in Models `All Models` mode, the source filter sidebar, and the Create-API-Model picker enrichment.

```yaml
get:
  path: /v1/providers
  operationId: listProviders
  tags: [directory]
  parameters:
    - name: page
      in: query
      schema: { type: integer, default: 1, minimum: 1 }
    - name: page_size
      in: query
      schema: { type: integer, default: 50, minimum: 1, maximum: 100 }
    - name: specialization
      in: query
      description: narrow to providers that offer models with this specialization
      schema: { type: string }
    - name: capability
      in: query
      description: filter providers by offered model capability (e.g. "tool-use", "vision")
      schema: { type: array, items: { type: string } }
  responses:
    '200':
      schema:
        allOf:
          - $ref: PaginatedEnvelope
          - properties:
              data: { type: array, items: { $ref: ProviderSummary } }

ProviderSummary:
  type: object
  required: [code, name, api_format, default_base_url, auth_mode, model_count, category]
  properties:
    code:
      type: string
      description: "stable identifier, matches bodhi-server api_format value where applicable"
      example: "openai"
    name:              { type: string, example: "OpenAI" }
    logo_url:          { type: string, format: uri, nullable: true }
    api_format:        { type: string, example: "openai-completions" }
    default_base_url:  { type: string, format: uri, example: "https://api.openai.com/v1" }
    auth_mode:         { type: string, enum: [api-key, oauth, none] }
    homepage:          { type: string, format: uri, nullable: true }
    docs_url:          { type: string, format: uri, nullable: true }
    pricing_url:       { type: string, format: uri, nullable: true }
    short_description: { type: string, nullable: true }
    category:          { type: string, enum: [first-party, open-compat, aggregator, self-hosted] }
    model_count:       { type: integer, description: "how many models this provider offers (from /v1/providers/{code}/models)" }
    specializations:   { type: array, items: { type: string }, description: "coverage tags — specialization codes" }
    cost_tier:         { type: string, enum: [free, low, medium, high, mixed], nullable: true }
    uptime_percent:    { type: number, nullable: true, example: 99.9 }
    verified:          { type: boolean, description: "true if Bodhi team has validated this entry" }
    updated_at:        { type: string, format: date-time }
```

### `GET /v1/providers/{code}` — provider detail + offered models
Drives the `UnconnectedProviderPanel` detail view, the `↗ provider` backlink from api-model rows, and cost/capability chips on `provider-unconnected` rows.

```yaml
get:
  path: /v1/providers/{code}
  operationId: getProvider
  tags: [directory]
  parameters:
    - name: code
      in: path
      required: true
      schema: { type: string }
  responses:
    '200':
      schema: { $ref: ProviderDetail }
    '404':
      schema: { $ref: BodhiErrorResponse }

ProviderDetail:
  allOf:
    - $ref: ProviderSummary
    - type: object
      required: [models]
      properties:
        description: { type: string, nullable: true, description: "longer blurb" }
        rate_limits:
          type: object
          nullable: true
          properties:
            rpm_default: { type: integer, nullable: true }
            tpm_default: { type: integer, nullable: true }
        models:
          type: array
          items: { $ref: DirectoryModel }
        tos_url:     { type: string, format: uri, nullable: true }
        signup_url:  { type: string, format: uri, nullable: true }

DirectoryModel:
  type: object
  required: [id, model_id, name, capabilities, context_length]
  properties:
    id:
      type: string
      description: "stable catalog ID, used for ranked-mode cross-reference"
      example: "openai/gpt-5"
    model_id:
      type: string
      description: "provider-native model identifier, unprefixed"
      example: "gpt-5"
    name:            { type: string, description: "display label" }
    family:          { type: string, nullable: true, example: "gpt-5" }
    capabilities:
      type: object
      properties:
        tool_use:         { type: boolean }
        vision:           { type: boolean }
        reasoning:        { type: boolean }
        structured_output: { type: boolean }
        embedding:        { type: boolean }
        speech:           { type: boolean }
        image_gen:        { type: boolean }
    context_length:   { type: integer, example: 128000 }
    output_tokens:    { type: integer, nullable: true }
    cost:
      type: object
      nullable: true
      properties:
        input_per_mtok:       { type: number, example: 1.25 }
        output_per_mtok:      { type: number, example: 10.00 }
        cached_input_per_mtok: { type: number, nullable: true }
        currency:             { type: string, default: "USD" }
    specializations: { type: array, items: { type: string } }
    benchmarks:
      type: array
      items:
        type: object
        required: [key, score]
        properties:
          key:   { type: string, example: "HumanEval" }
          score: { type: number }
    recommended: { type: boolean, description: "Bodhi recommendation flag" }
    deprecated:  { type: boolean, default: false }
```

---

## 5. API formats (enrichment for Create-API-Model)
### `GET /v1/api-formats` — extended formats list
Mirrors the `API_FORMATS` fixture in `primitives.jsx`. The frontend uses this to populate the ApiFormatPicker; bodhi-server's `/bodhi/v1/models/api/formats` remains the canonical enum for server-side validation (see `10-bodhi-server-apis.md §4`).

```yaml
get:
  path: /v1/api-formats
  operationId: listApiFormats
  tags: [directory]
  responses:
    '200':
      schema:
        type: object
        required: [data]
        properties:
          data:
            type: array
            items:
              type: object
              required: [code, label, default_base_url, auth_mode, bodhi_server_enum]
              properties:
                code:              { type: string, example: "openai-completions" }
                label:              { type: string, example: "OpenAI — Completions" }
                default_base_url:  { type: string, format: uri }
                auth_mode:         { type: string, enum: [api-key, oauth, none] }
                provider_code:     { type: string, description: "joins with /v1/providers/{code}", example: "openai" }
                bodhi_server_enum:
                  type: string
                  description: "matches the bodhi-server ApiFormat enum value used on create/update"
                  enum: [openai, openai_responses, anthropic, anthropic_oauth, gemini]
                docs_url:          { type: string, format: uri, nullable: true }
                supports_fetch_models: { type: boolean, description: "provider exposes /v1/models endpoint" }
                notes:             { type: string, nullable: true }
```

---

## 6. Specializations + benchmarks
### `GET /v1/specializations` — taxonomy + benchmark mapping
Drives the Specialization sidebar filter and the "benchmark-side-effect sort" (see `models.md §4, §6`).

```yaml
get:
  path: /v1/specializations
  operationId: listSpecializations
  tags: [taxonomy]
  responses:
    '200':
      schema:
        type: object
        required: [data]
        properties:
          data:
            type: array
            items:
              type: object
              required: [code, label, benchmark_key]
              properties:
                code:            { type: string, example: "coding" }
                label:            { type: string, example: "Coding" }
                benchmark_key:    { type: string, example: "HumanEval" }
                order:            { type: integer, description: "stable sort order in UI" }
                description:      { type: string, nullable: true }
                capability_hint:  { type: string, nullable: true, description: "capability that this specialization maps to (e.g. 'tool-use' for agent)" }
```

Example response (matching the wireframe's `SPECIALIZATIONS` constant):
```json
{
  "data": [
    {"code": "chat",      "label": "Chat · general",     "benchmark_key": "ArenaElo",   "order": 1},
    {"code": "coding",    "label": "Coding",             "benchmark_key": "HumanEval",  "order": 2},
    {"code": "agent",     "label": "Agentic · tool-use", "benchmark_key": "BFCL",       "order": 3, "capability_hint": "tool-use"},
    {"code": "reason",    "label": "Reasoning",          "benchmark_key": "GPQA",       "order": 4},
    {"code": "longctx",   "label": "Long context",       "benchmark_key": "RULER",      "order": 5},
    {"code": "multiling", "label": "Multilingual",       "benchmark_key": "mMMLU",      "order": 6},
    {"code": "vision",    "label": "Vision + text",      "benchmark_key": "MMMU",       "order": 7, "capability_hint": "vision"},
    {"code": "embed",     "label": "Text embedding",     "benchmark_key": "MTEB",       "order": 8, "capability_hint": "embedding"},
    {"code": "memb",      "label": "Multimodal embed",   "benchmark_key": "MMEB",       "order": 9},
    {"code": "small",     "label": "Small & fast",       "benchmark_key": "OpenLLMLB",  "order": 10}
  ]
}
```

### `GET /v1/benchmarks` — benchmark registry
Metadata for each benchmark: display label, higher-is-better direction, source URL, last refresh time.

```yaml
get:
  path: /v1/benchmarks
  operationId: listBenchmarks
  tags: [taxonomy]
  responses:
    '200':
      schema:
        type: object
        required: [data]
        properties:
          data:
            type: array
            items:
              type: object
              required: [key, label, direction, last_refreshed_at]
              properties:
                key:                { type: string, example: "HumanEval" }
                label:              { type: string, example: "HumanEval (code)" }
                direction:          { type: string, enum: [higher-is-better, lower-is-better] }
                scale_max:          { type: number, nullable: true, example: 100 }
                description:        { type: string, nullable: true }
                source_url:         { type: string, format: uri, nullable: true }
                last_refreshed_at:  { type: string, format: date-time }
```

---

## 7. Leaderboards — ranked mode feed
### `GET /v1/leaderboards/{benchmark}` — ranked list for a benchmark
**The core endpoint for ranked display mode.** Returns the full leaderboard for the benchmark; frontend filters/joins against local entities.

```yaml
get:
  path: /v1/leaderboards/{benchmark}
  operationId: getLeaderboard
  tags: [leaderboards]
  parameters:
    - name: benchmark
      in: path
      required: true
      description: benchmark_key from /v1/benchmarks
      schema: { type: string, example: "HumanEval" }
    - name: page
      in: query
      schema: { type: integer, default: 1 }
    - name: page_size
      in: query
      schema: { type: integer, default: 100, minimum: 1, maximum: 500 }
    - name: kind
      in: query
      description: restrict to one model kind (optional)
      schema: { type: string, enum: [any, api, local-gguf] }
    - name: specialization
      in: query
      description: "optional — usually the benchmark pins a specialization, but callable directly"
      schema: { type: string }
  responses:
    '200':
      schema:
        type: object
        required: [benchmark, data, total, page, page_size, last_refreshed_at]
        properties:
          benchmark: { $ref: BenchmarkSummary }
          data:
            type: array
            items: { $ref: LeaderboardEntry }
          total:              { type: integer }
          page:               { type: integer }
          page_size:          { type: integer }
          last_refreshed_at:  { type: string, format: date-time }

BenchmarkSummary:
  type: object
  required: [key, label, direction]
  properties:
    key:       { type: string }
    label:     { type: string }
    direction: { type: string, enum: [higher-is-better, lower-is-better] }

LeaderboardEntry:
  type: object
  required: [rank, model_id, score, kind]
  properties:
    rank:       { type: integer, description: "absolute rank, 1-indexed, never renumbered by filters" }
    model_id:
      type: string
      description: "canonical identifier — joins with DirectoryModel.id for API, or repo+filename for GGUF"
      example: "anthropic/claude-sonnet-4.5"
    display_name: { type: string, example: "Claude Sonnet 4.5" }
    score:      { type: number, example: 82.1 }
    kind:
      type: string
      enum: [api, local-gguf]
      description: "api = provider-hosted; local-gguf = downloadable .gguf file"
    # Populated for kind=api:
    provider_code:  { type: string, nullable: true, example: "anthropic" }
    provider_model: { type: string, nullable: true, example: "claude-sonnet-4.5" }
    # Populated for kind=local-gguf:
    hf_repo:     { type: string, nullable: true, example: "Qwen/Qwen3-Coder-32B-GGUF" }
    hf_filename: { type: string, nullable: true, example: "qwen3-coder-32b.Q4_K_M.gguf" }
    # Shared enrichment:
    license:         { type: string, nullable: true }
    size_bytes:      { type: integer, nullable: true }
    context_length:  { type: integer, nullable: true }
    capabilities:    { type: array, items: { type: string } }
    specializations: { type: array, items: { type: string } }
    source_url:      { type: string, format: uri, nullable: true, description: "where this score came from" }
```

### Join semantics (reminder from `00-architecture.md §4`)
- Rank numbers are **absolute**. The frontend never renumbers them under filters.
- `LeaderboardEntry.kind = api` with matching `provider_code` + `provider_model` → the frontend checks its local `api-model` list to find user configs (stacks them into the ranked row's primaries).
- `LeaderboardEntry.kind = local-gguf` with matching `hf_repo` + `hf_filename` → the frontend checks local files/aliases; renders accordingly (alias-stack or orphan or pullable).

---

## 8. HF catalog mirror — trending / new / detail / search
### `GET /v1/repos/trending` — trending HF repos (curated subset)
```yaml
get:
  path: /v1/repos/trending
  operationId: listTrendingRepos
  tags: [catalog]
  parameters:
    - name: page
      in: query
      schema: { type: integer, default: 1 }
    - name: page_size
      in: query
      schema: { type: integer, default: 30, maximum: 100 }
    - name: window
      in: query
      description: trending window
      schema: { type: string, enum: [1d, 7d, 30d], default: 7d }
  responses:
    '200':
      schema:
        allOf:
          - $ref: PaginatedEnvelope
          - properties:
              data: { type: array, items: { $ref: RepoSummary } }
```

### `GET /v1/repos/new` — new launches
Same shape as `/trending`, but sorted by `created_at` desc with curation filters (only repos that pass our "has GGUF files + reasonable quality" check).

### `GET /v1/repos/{owner}/{name}` — enriched repo detail
Single repo with normalized license, capability tags, quant enumeration with sizes, mirror of HF description. Used by `HfRepoPanel`.

```yaml
get:
  path: /v1/repos/{owner}/{name}
  operationId: getRepo
  tags: [catalog]
  parameters:
    - name: owner
      in: path
      required: true
      schema: { type: string }
    - name: name
      in: path
      required: true
      schema: { type: string }
  responses:
    '200':
      schema: { $ref: RepoDetail }
    '404':
      schema: { $ref: BodhiErrorResponse }

RepoSummary:
  type: object
  required: [id, owner, name, downloads, likes, tags, updated_at]
  properties:
    id:            { type: string, example: "Qwen/Qwen3.5-9B-GGUF" }
    owner:         { type: string }
    name:          { type: string }
    display_name:  { type: string, nullable: true, description: "human-friendly; falls back to name" }
    author:        { type: string }
    description:   { type: string, nullable: true, description: "short blurb" }
    pipeline_tag:  { type: string, nullable: true, example: "text-generation" }
    architecture:  { type: string, nullable: true, example: "qwen3" }
    license:       { type: string, nullable: true, description: "normalized: Apache-2 | MIT | Llama | Gemma | CC-BY | Proprietary | Other" }
    license_raw:   { type: string, nullable: true, description: "raw HF license tag, pre-normalization" }
    num_parameters: { type: integer, nullable: true, example: 9000000000 }
    context_length: { type: integer, nullable: true }
    capabilities: { type: array, items: { type: string } }
    specializations: { type: array, items: { type: string } }
    downloads:     { type: integer }
    likes:         { type: integer }
    trending_score: { type: number, nullable: true }
    created_at:    { type: string, format: date-time }
    updated_at:    { type: string, format: date-time }
    gated:         { type: boolean, default: false }
    tags:          { type: array, items: { type: string } }

RepoDetail:
  allOf:
    - $ref: RepoSummary
    - type: object
      properties:
        readme_md:   { type: string, nullable: true, description: "README snippet (first ~2kb) — full readme fetched lazy from HF" }
        base_model:  { type: string, nullable: true, description: "e.g. 'Qwen/Qwen3.5-9B' when this is a quantized repo" }
        quants:
          type: array
          description: "enumerated from .gguf files in the repo; ranked by size ascending"
          items:
            type: object
            required: [filename, quant, size_bytes]
            properties:
              filename:      { type: string, example: "qwen3.5-9b-q4_k_m.gguf" }
              quant:         { type: string, example: "Q4_K_M" }
              size_bytes:    { type: integer }
              is_sharded:    { type: boolean, default: false }
              shard_count:   { type: integer, nullable: true }
              is_default:    { type: boolean, description: "flagged by Bodhi as the recommended entry" }
              fit_hint:      { type: string, nullable: true, enum: [small, balanced, quality, full-precision] }
        scores:
          type: array
          description: "benchmark scores for this repo (by the underlying model)"
          items:
            type: object
            required: [benchmark_key, score]
            properties:
              benchmark_key: { type: string }
              score:         { type: number }
        related_repos:
          type: array
          items: { $ref: RepoSummary }
          description: "sibling quantizations of the same base model"
```

### `GET /v1/repos/search` — full-text search (curated mirror)
Fast text search over the api.getbodhi.app mirror. Complements direct HF search for users who want enriched/normalized results.

```yaml
get:
  path: /v1/repos/search
  operationId: searchRepos
  tags: [catalog]
  parameters:
    - name: q
      in: query
      required: true
      schema: { type: string, example: "qwen coder" }
    - name: page
      in: query
      schema: { type: integer, default: 1 }
    - name: page_size
      in: query
      schema: { type: integer, default: 30, maximum: 100 }
    - name: license
      in: query
      schema: { type: array, items: { type: string } }
    - name: capability
      in: query
      schema: { type: array, items: { type: string } }
    - name: specialization
      in: query
      schema: { type: string }
    - name: size_bucket
      in: query
      description: pre-computed size bucket
      schema: { type: string, enum: [lt-5gb, 5-15gb, gt-15gb] }
    - name: sort
      in: query
      schema: { type: string, enum: [relevance, downloads, likes, trending, recent], default: relevance }
  responses:
    '200':
      schema:
        allOf:
          - $ref: PaginatedEnvelope
          - properties:
              data: { type: array, items: { $ref: RepoSummary } }
```

### Freshness
- `/v1/repos/trending` + `/v1/repos/new`: refreshed hourly.
- `/v1/repos/{owner}/{name}`: refreshed nightly or on explicit user-triggered fetch (see `30-huggingface-integration.md §5`).
- `/v1/repos/search`: indexed nightly.

---

## 9. Licenses taxonomy
### `GET /v1/licenses` — license normalization reference
Supports the License filter sidebar. Returns the canonical taxonomy used in enrichment:

```yaml
get:
  path: /v1/licenses
  operationId: listLicenses
  tags: [taxonomy]
  responses:
    '200':
      schema:
        type: object
        required: [data]
        properties:
          data:
            type: array
            items:
              type: object
              required: [code, label, osi_approved]
              properties:
                code:          { type: string, example: "apache-2" }
                label:          { type: string, example: "Apache-2.0" }
                osi_approved:   { type: boolean }
                permissive:     { type: boolean }
                commercial_use: { type: boolean }
                url:            { type: string, format: uri, nullable: true }
                aliases:        { type: array, items: { type: string }, description: "raw tags we map to this code" }
```

---

## 10. Error model
All `4xx` / `5xx` use the shared `BodhiErrorResponse` shape (see `10-bodhi-server-apis.md §1`). Common codes:
- `404 not_found_error` — `provider_not_found`, `repo_not_found`, `benchmark_not_found`, `specialization_not_found`.
- `400 validation_error` — `invalid_pagination`, `invalid_sort`, `invalid_window`.
- `429 rate_limit_exceeded` — rate limit. Header `Retry-After` present.
- `503 service_unavailable` — upstream (HF) outage; frontend should fall back to direct HF.

## 11. Seeding & maintenance
How api.getbodhi.app data gets populated — not a frontend API concern but required to scope the backend build.

- **Providers** (`/v1/providers`, `/v1/api-formats`): **fully curated** — a YAML/JSON file in the api.getbodhi.app repo, reviewed by humans, shipped on deploy. ~20 entries.
- **Specializations / benchmarks / licenses**: curated + static. Change rarely.
- **Leaderboards** (`/v1/leaderboards/{key}`): mix of (a) HF Open LLM Leaderboard scraped nightly, (b) published benchmark tables (Chatbot Arena, BFCL, MMMU) scraped/API'd, (c) manual curation for API-model scores not covered by HF leaderboards.
- **Repos** (`/v1/repos/*`): mirror of HF top-N GGUF repos by downloads + trending. See `30-huggingface-integration.md` for the full pipeline.
- **Refresh cadence** documented in `§8` per endpoint. The backend exposes no refresh trigger to the frontend; staleness is hidden behind `last_refreshed_at` fields.

## 12. Summary
12 new endpoints on api.getbodhi.app. In priority order of frontend dependency:

1. `/v1/providers` + `/v1/providers/{code}` — required for `All Models` mode.
2. `/v1/specializations` — required for the Specialization filter.
3. `/v1/benchmarks` + `/v1/leaderboards/{benchmark}` — required for ranked mode.
4. `/v1/api-formats` — can initially ship as a static frontend fixture, migrated to API when available.
5. `/v1/repos/trending` + `/v1/repos/new` — required for the `↑ Trending` / `★ New` browse menu.
6. `/v1/repos/{owner}/{name}` + `/v1/repos/search` — optional (frontend can call HF direct); api.getbodhi.app adds enrichment + caching.
7. `/v1/licenses` — can start as a static frontend fixture.
