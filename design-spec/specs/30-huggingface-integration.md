# 30 · HuggingFace integration
*Analysis of HF Hub APIs used (a) directly by the frontend for browse and (b) by api.getbodhi.app to seed/refresh its mirror.*

Authoritative reference: https://huggingface.co/.well-known/openapi.json (machine-readable) and https://huggingface.co/docs/hub/api.

## 1. Two integration points
1. **Direct frontend calls** (browser → `huggingface.co/api/*`): on-demand repo lookups, deep search, long-tail repos not in our mirror. CORS-safe (HF allows `*` origin on `/api/*`).
2. **Server-side seeding** (api.getbodhi.app → `huggingface.co/api/*` + raw file fetches for GGUF header parsing): periodic ingestion of the top-N GGUF repos, trending cuts, license/capability normalization. Tokenized with a dedicated `HF_TOKEN` for higher rate limits.

Both integration points use the same HF endpoints; the difference is cadence + auth + storage.

## 2. HF endpoints we rely on
### 2.1 `GET https://huggingface.co/api/models` — list/search models
The workhorse. Used by `/v1/repos/search`, `/v1/repos/trending`, and the frontend's live search.

**Query params (subset we use):**
| Param | Purpose |
|---|---|
| `search` | full-text search across model IDs |
| `filter` | tag filter (comma-separated or repeated); e.g. `filter=gguf,text-generation` |
| `library` | HF library filter; `library=gguf` restricts to GGUF repos |
| `pipeline_tag` | pipeline filter; e.g. `pipeline_tag=text-generation` |
| `author` | namespace filter; e.g. `author=Qwen` |
| `sort` | one of `downloads`, `likes`, `trendingScore`, `createdAt`, `lastModified` |
| `direction` | `-1` for desc, absent/`1` for asc |
| `limit` | page size (max 1000, recommend 100) |
| `cursor` | opaque cursor from `Link: <...?cursor=...>; rel="next"` header |
| `full` | `true` for full metadata incl. siblings, cardData, config |

**Response shape (observed from `?search=gguf&limit=3&full=true`):**
```yaml
HfModelListItem:
  type: object
  properties:
    _id:            { type: string }
    id:             { type: string, example: "Qwen/Qwen2.5-7B-Instruct-GGUF" }
    author:         { type: string }
    modelId:        { type: string }
    private:        { type: boolean }
    gated:          { type: [boolean, string], description: "false | 'auto' | 'manual'" }
    disabled:       { type: boolean, nullable: true }
    downloads:      { type: integer }
    likes:          { type: integer }
    trendingScore:  { type: number, nullable: true, description: "present when sort=trendingScore or full=true" }
    sha:            { type: string, description: "current commit SHA of main" }
    lastModified:   { type: string, format: date-time }
    createdAt:      { type: string, format: date-time }
    pipeline_tag:   { type: string, nullable: true }
    library_name:   { type: string, nullable: true, example: "transformers" }
    tags:           { type: array, items: { type: string } }
    siblings:
      type: array
      description: "present only with ?full=true"
      items:
        type: object
        properties:
          rfilename: { type: string, example: "qwen2.5-7b-instruct-q4_k_m.gguf" }
    cardData:       { type: object, nullable: true, description: "YAML frontmatter of README" }
    config:         { type: object, nullable: true }
    model-index:    { type: object, nullable: true }
    spaces:         { type: array, items: { type: string }, nullable: true }
```

**Pagination note.** HF uses cursor pagination — no `total` count returned. `Link` header contains `<next>` and sometimes `<previous>`. api.getbodhi.app MUST normalize this to page-based pagination when mirroring (compute total = mirrored_row_count).

### 2.2 `GET https://huggingface.co/api/models/{owner}/{name}` — single model detail
Used by `/v1/repos/{owner}/{name}` on first-fetch + lazy cache-miss path from frontend.

**Response (observed from `?owner=Qwen&name=Qwen2.5-7B-Instruct-GGUF`):**
```yaml
HfModelInfo:
  allOf:
    - $ref: HfModelListItem
    - type: object
      properties:
        widgetData:    { type: array, nullable: true }
        cardData:
          type: object
          description: "parsed README YAML frontmatter"
          properties:
            license:       { type: string, nullable: true, example: "apache-2.0" }
            license_link:  { type: string, nullable: true }
            language:      { type: array, items: { type: string }, nullable: true }
            pipeline_tag:  { type: string, nullable: true }
            base_model:    { type: string, nullable: true, example: "Qwen/Qwen2.5-7B-Instruct" }
            tags:          { type: array, items: { type: string }, nullable: true }
        gguf:
          type: object
          nullable: true
          description: "present when HF has parsed the GGUF header; Bodhi Directory consumes this gratefully"
          properties:
            total:          { type: integer, description: "parameter count" }
            architecture:   { type: string, example: "qwen2" }
            context_length: { type: integer }
            chat_template:  { type: string, nullable: true }
            bos_token:      { type: string, nullable: true }
            eos_token:      { type: string, nullable: true }
        usedStorage:   { type: integer, description: "total size in bytes across all siblings" }
        siblings:      { type: array, description: "always present on single-item endpoint" }
```

### 2.3 `GET https://huggingface.co/api/models/{owner}/{name}/tree/{revision}` — file tree
Used to enumerate files + individual sizes (per-quant byte counts). `/v1/repos/{owner}/{name}.quants[].size_bytes` is populated from here.

```yaml
HfTreeEntry:
  type: object
  properties:
    type:    { type: string, enum: [file, directory] }
    oid:     { type: string, description: "blob SHA" }
    size:    { type: integer }
    path:    { type: string }
    lfs:
      type: object
      nullable: true
      properties:
        oid: { type: string }
        size: { type: integer, description: "actual file size for LFS files (GGUF is always LFS)" }
        pointerSize: { type: integer }
```

**Note:** for `.gguf` files `size` in the non-LFS field is the pointer size (~150 bytes). Use `lfs.size` for the real byte count.

### 2.4 `GET https://huggingface.co/{owner}/{name}/resolve/{revision}/{filename}` — download URL
Used by bodhi-server for actual pulls. Already integrated via `HubService.download`. Not consumed by this spec pass for frontend.

### 2.5 `GET https://huggingface.co/api/models-tags-by-type` — tag taxonomy
Returns the full tag catalog (pipeline tags, library tags, language tags, license tags). Used by api.getbodhi.app seeding to map HF tags → Bodhi normalized categories:
- `license:apache-2.0` → `apache-2`
- `license:mit` → `mit`
- `license:llama3` / `license:llama2` → `llama`
- `license:gemma` → `gemma`
- `license:cc-by-4.0` etc. → `cc-by`
- everything else → `proprietary`

### 2.6 `GET https://huggingface.co/api/models/{id}/revision/{rev}` — resolve snapshot SHA
Needed if the frontend wants to show a pin'd snapshot. Already covered by bodhi-server's alias snapshot logic.

### 2.7 `GET https://huggingface.co/.well-known/openapi.json`
The canonical HF OpenAPI spec. api.getbodhi.app's seeding service should compile TS/Rust client bindings from this (keeps in sync with HF changes).

## 3. GGUF metadata — why we mirror
HF's `gguf` field on model detail contains parsed header info for GGUF files (architecture, context length, chat template, BOS/EOS). But it's:
- Not always present — HF only parses GGUFs when the repo has `library: gguf` tagged.
- Not per-file — one `gguf` object per repo even when there are multiple quants.
- Missing some fields we care about: explicit vision/audio capability flags, tool-use hints.

api.getbodhi.app's seeding therefore does its own GGUF header parse for canonical repos. The pipeline:
1. Fetch `/api/models/{id}` → get siblings list.
2. For each `.gguf` sibling, fetch the first 4 MB via `/resolve/.../gguf` with `Range: bytes=0-4194303`.
3. Parse the GGUF header client-side (see `llama.cpp` spec; first 4 MB covers the metadata KV table).
4. Extract: `architecture`, `context_length`, `chat_template`, `num_params`, `tool_calling_support` (inferred from chat template jinja), `vision` (inferred from KV keys `clip.*`, `vision.*`).
5. Store normalized in the mirror DB.

Skipped for non-GGUF repos — not in Bodhi's scope.

## 4. Trending / new-launches
### Trending
HF exposes `sort=trendingScore&direction=-1` on `/api/models`. The score is recomputed daily. api.getbodhi.app should:
1. Nightly: fetch `GET /api/models?library=gguf&sort=trendingScore&direction=-1&limit=100`.
2. Intersect with the mirrored repo set (only surface repos we've already enriched).
3. Expose as `/v1/repos/trending`.

Window parameter (`1d`/`7d`/`30d`) is a Bodhi addition — HF returns only the current snapshot. api.getbodhi.app stores daily snapshots of the trending list to compute per-window movers.

### New launches
Sort: `createdAt&direction=-1`. Filter: GGUF library + `downloads > 50` (noise filter). Nightly refresh. No window param needed; "new" means "created in last 30 days".

## 5. Seeding pipeline (api.getbodhi.app side)
A summary of the backend pipeline — not frontend API, but required to understand which HF calls are batched vs on-demand.

### 5.1 Continuous ingest (nightly cron)
1. **Top repos by downloads.** Fetch `GET /api/models?library=gguf&sort=downloads&direction=-1&limit=100&full=true`. For each, call `GET /api/models/{id}/tree/main` and first-4MB GGUF parse on new siblings.
2. **Trending cut.** `GET /api/models?library=gguf&sort=trendingScore&direction=-1&limit=100`. Same enrichment if repo is new.
3. **New repos.** `GET /api/models?library=gguf&sort=createdAt&direction=-1&limit=100`. Same.
4. **Recheck staleness.** For mirrored repos whose `lastModified` differs from stored, re-fetch + re-enrich.
5. **Tag map refresh.** `GET /api/models-tags-by-type` weekly.

### 5.2 On-demand ingest (cache miss)
When frontend requests `GET /v1/repos/{owner}/{name}` for a repo not in the mirror:
1. api.getbodhi.app detects miss → synchronous fetch from HF with short timeout (2 s).
2. If HF returns 200, mirror the response (enrich asynchronously — surface a `partial: true` flag until enrichment completes).
3. If HF returns 404 or 429, forward the error upstream (frontend falls back to direct HF).

### 5.3 Leaderboard ingest
Separate pipelines per benchmark:
| Benchmark | Source | Update cadence |
|---|---|---|
| HumanEval / OpenLLMLB / ARC / MMLU / MMLU-Pro / GPQA | HF Open LLM Leaderboard dataset (`open-llm-leaderboard/contents`) | weekly |
| Chatbot Arena Elo | lmarena.ai public leaderboard CSV | weekly |
| BFCL (tool-use) | gorilla-llm/berkeley-function-calling-leaderboard | weekly |
| MMMU (vision) | https://mmmu-benchmark.github.io/#leaderboard (HTML scrape) | monthly |
| RULER (long-ctx) | published tables on HF spaces | monthly |
| MTEB | https://huggingface.co/spaces/mteb/leaderboard dataset | weekly |
| API-model benchmarks (GPT / Claude / Gemini scores) | manual curation file | per-release |

Each ingest maps the leaderboard's model identifiers → canonical Bodhi `LeaderboardEntry` shape (see `20-getbodhi-public-apis.md §7`). Unknown model IDs get flagged for manual review (don't silently skip).

## 6. Rate limits
HF's public rate limit (unauthenticated) is ~500 req/hour per IP — fine for individual frontend browsing. For seeding, api.getbodhi.app must authenticate with `Authorization: Bearer hf_...` (higher shared quota).

Frontend behaviour on 429 from HF:
1. Fall back to `/v1/repos/{owner}/{name}` on api.getbodhi.app (our mirror).
2. If that's also a miss, render a `stale · HF unreachable` chip on the row.

## 7. Direct HF calls from the frontend
For completeness, the frontend endpoints hit directly are:
- `GET /api/models?search=...&library=gguf&full=true` — live search when user types in the filter bar (api.getbodhi.app mirror used first; HF fallback only if user opts in to "include long-tail").
- `GET /api/models/{id}` — on demand for not-yet-mirrored repos.
- `GET /api/models/{id}/tree/main` — quant enumeration if `/v1/repos/{id}.quants` is empty.

All use `credentials: "omit"` and no token — browser fetches only.

## 8. Open questions
1. **License normalization taxonomy edge cases.** HF has ~40 licenses in common use; our normalized set is 6 (Apache-2 / MIT / Llama / Gemma / CC-BY / Proprietary). Need a reviewed mapping file in the api.getbodhi.app repo.
2. **Quant detection regex.** `:Q4_K_M` style is standard but not universal — e.g. `-UD-IQ4_NL_XL` (unsloth prefix). The parser in seeding must tolerate unknown quant labels and fall back to `quant = null, filename = rfilename`.
3. **Multi-part GGUFs** (`-00001-of-00004.gguf`). Must be collapsed into a single logical quant with `is_sharded=true, shard_count=4`. The Create-alias QuantPicker needs this distinction to set filename correctly.
4. **Gated repos** (HF requires user approval). Frontend must surface the `gated` flag clearly; pulling requires the user's own HF token. Out of scope for this spec pass but noted for the pull flow.

## 9. Summary
- **Frontend** calls HF directly for on-demand / long-tail; api.getbodhi.app is the default source for browse.
- **api.getbodhi.app** mirrors HF nightly, enriches with GGUF header parse + license/capability normalization, and exposes the curated view via `/v1/repos/*`.
- **bodhi-server** uses HF only for actual downloads (existing `HubService.download`).
- No change to bodhi-server's HF integration.
