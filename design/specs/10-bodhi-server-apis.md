# 10 · bodhi-server APIs
*Every bodhi-server endpoint the Models / Create-alias / Create-API-model screens call.*

Source of truth is `crates/routes_app/src/models/*`. This file annotates each endpoint as:
- **KEEP** — no change required; existing shape satisfies the wireframe.
- **CHANGE** — shape needs a gap filled (new field, new query param, etc.) for the wireframe. Called out explicitly.
- **NEW** — does not exist yet. Must be added.

Summary: **everything needed for My Models is already implemented**. The only recommended additions are convenience/batch endpoints that could also be computed client-side (noted as `CHANGE (optional)`).

## 1. Conventions
- Base path: `/bodhi/v1`.
- Auth: all endpoints require an authenticated session or bearer token (see `routes.rs` route groups for role requirements — User / PowerUser). Existing.
- Response envelope for lists:
  ```yaml
  PaginatedEnvelope:
    type: object
    required: [data, total, page, page_size]
    properties:
      data:
        type: array
        items: {}   # overridden per endpoint
      total: { type: integer, minimum: 0 }
      page: { type: integer, minimum: 1 }
      page_size: { type: integer, minimum: 1, maximum: 100 }
  ```
- Error envelope is `BodhiErrorResponse` (existing):
  ```yaml
  BodhiErrorResponse:
    type: object
    required: [error]
    properties:
      error:
        type: object
        required: [message, type, code]
        properties:
          message: { type: string }
          type:    { type: string, enum: [validation_error, not_found_error, authentication_error, authorization_error, conflict_error, internal_server_error] }
          code:    { type: string }
  ```
- Pagination/sort query params (existing `PaginationSortParams`):
  ```yaml
  PaginationSortParams:
    type: object
    properties:
      page:       { type: integer, default: 1, minimum: 1 }
      page_size:  { type: integer, default: 30, minimum: 1, maximum: 100 }
      sort:       { type: string, description: "field to sort by (endpoint-specific)" }
      sort_order: { type: string, enum: [asc, desc], default: asc }
  ```

---

## 2. Unified models list — the Models page primary feed
### `GET /bodhi/v1/models` — list all aliases (user + model + API) · **KEEP** (with one `CHANGE (optional)` noted)
Used by: Models page (`My Models` mode), ranked-mode local join.

```yaml
get:
  path: /bodhi/v1/models
  operationId: listAllModels
  tags: [models]
  parameters:
    - $ref: PaginationSortParams
    - name: kind
      in: query
      required: false
      schema: { type: array, items: { type: string, enum: [alias, file, api-model] } }
      description: |
        CHANGE (optional): filter by kind (maps to Alias source). Today the frontend
        receives all kinds + filters client-side. Server-side filtering is nice-to-have
        to avoid over-fetching for kind-restricted views. Not a blocker — client-side
        works.
    - name: source
      in: query
      required: false
      schema: { type: array, items: { type: string } }
      description: |
        CHANGE (optional): filter by originating provider code for api-model rows
        (e.g. "openai"). Same rationale as `kind`.
  responses:
    '200':
      description: Paginated list of all aliases
      schema:
        allOf:
          - $ref: PaginatedEnvelope
          - properties:
              data:
                type: array
                items: { $ref: AliasResponse }
```

### `AliasResponse` shape (existing — services::AliasResponse)
Discriminated union by `source` field:
```yaml
AliasResponse:
  oneOf:
    - $ref: UserAliasResponse   # source=user
    - $ref: ModelAliasResponse  # source=model
    - $ref: ApiAliasResponse    # source=api

UserAliasResponse:
  type: object
  required: [id, alias, repo, filename, snapshot, source, request_params, context_params, created_at, updated_at, model_params]
  properties:
    id:              { type: string, format: uuid }
    alias:           { type: string, example: "my-qwen-coder" }
    repo:            { type: string, example: "Qwen/Qwen3-Coder-32B-GGUF" }
    filename:        { type: string, example: "qwen3-coder-32b.Q4_K_M.gguf" }
    snapshot:        { type: string, example: "abc12345" }
    source:          { type: string, enum: [user] }
    model_params:    { type: object, additionalProperties: true }
    request_params:  { $ref: OAIRequestParams }
    context_params:  { type: array, items: { type: string } }
    created_at:      { type: string, format: date-time }
    updated_at:      { type: string, format: date-time }
    metadata:        { $ref: ModelMetadata, nullable: true }

ModelAliasResponse:
  type: object
  required: [source, alias, repo, filename, snapshot]
  properties:
    source:     { type: string, enum: [model] }
    alias:      { type: string, description: "auto-generated; owner/repo:quant style" }
    repo:       { type: string }
    filename:   { type: string }
    snapshot:   { type: string }
    metadata:   { $ref: ModelMetadata, nullable: true }

ApiAliasResponse:
  type: object
  required: [source, id, api_format, base_url, has_api_key, models, forward_all_with_prefix, created_at, updated_at]
  properties:
    source:                   { type: string, enum: [api] }
    id:                       { type: string, example: "openai-gpt4" }
    api_format:               { type: string, enum: [openai, openai_responses, anthropic, anthropic_oauth, gemini] }
    base_url:                 { type: string, format: uri }
    has_api_key:              { type: boolean }
    models:                   { type: array, items: { $ref: ApiModel } }
    prefix:                   { type: string, nullable: true, example: "openai/" }
    forward_all_with_prefix:  { type: boolean }
    extra_headers:            { type: object, additionalProperties: { type: string }, nullable: true }
    extra_body:               { type: object, additionalProperties: true, nullable: true }
    created_at:               { type: string, format: date-time }
    updated_at:               { type: string, format: date-time }

ModelMetadata:
  type: object
  required: [capabilities, context, architecture]
  properties:
    capabilities:
      type: object
      properties:
        vision:   { type: boolean, nullable: true }
        audio:    { type: boolean, nullable: true }
        thinking: { type: boolean, nullable: true }
        tools:
          type: object
          properties:
            function_calling: { type: boolean, nullable: true }
            structured_output: { type: boolean, nullable: true }
    context:
      type: object
      properties:
        max_tokens: { type: integer }
    architecture:
      type: object
      properties:
        family: { type: string, example: "qwen2" }
    chat_template: { type: string, nullable: true }
```

### `GET /bodhi/v1/models/{id}` — get alias by UUID · **KEEP**
Used by: AliasPanel detail, alias edit pre-population.
Returns `UserAliasResponse`. Existing behaviour.

---

## 3. Local files (disk scan) — `file` rows
### `GET /bodhi/v1/models/files` — list local GGUF files · **KEEP**
Used by: Files filter in Models page, QuantPicker in Create-alias (to reuse already-downloaded quants).

```yaml
get:
  path: /bodhi/v1/models/files
  parameters:
    - $ref: PaginationSortParams
  responses:
    '200':
      schema:
        allOf:
          - $ref: PaginatedEnvelope
          - properties:
              data: { type: array, items: { $ref: LocalModelResponse } }

LocalModelResponse:
  type: object
  required: [repo, filename, snapshot, model_params]
  properties:
    repo:     { type: string }
    filename: { type: string }
    snapshot: { type: string }
    size:     { type: integer, nullable: true, description: "bytes" }
    model_params:  { type: object, additionalProperties: true }
    metadata: { $ref: ModelMetadata, nullable: true }
```

### Downloads (Downloads panel)
- `GET /bodhi/v1/models/files/pull` — list download requests · **KEEP**
- `POST /bodhi/v1/models/files/pull` — start a download · **KEEP**
- `GET /bodhi/v1/models/files/pull/{id}` — get download status · **KEEP**

```yaml
NewDownloadRequest:
  type: object
  required: [repo, filename]
  properties:
    repo:     { type: string, example: "Qwen/Qwen3.5-9B-GGUF" }
    filename: { type: string, example: "qwen3.5-9b-q4_k_m.gguf" }

DownloadRequest:
  type: object
  required: [id, repo, filename, status, created_at, updated_at]
  properties:
    id:                { type: string, format: uuid }
    repo:              { type: string }
    filename:          { type: string }
    status:            { type: string, enum: [pending, downloading, completed, error] }
    error:             { type: string, nullable: true }
    total_bytes:       { type: integer, nullable: true }
    downloaded_bytes:  { type: integer, nullable: true }
    started_at:        { type: string, format: date-time, nullable: true }
    created_at:        { type: string, format: date-time }
    updated_at:        { type: string, format: date-time }
```

### `CHANGE (optional)` — progress stream
The DownloadProgressStrip in Create-alias polls `GET …/pull/{id}` every 500 ms. Adding an SSE stream (`GET /bodhi/v1/models/files/pull/{id}/stream`) would reduce polling overhead. Not blocking for the wireframe migration.

---

## 4. API models — `api-model` + `provider-connected` rows
The Models page derives **both** `api-model` rows and `provider-connected` rows from the single `/bodhi/v1/models/api/*` surface:
- `api-model` kind = one row per `ApiAliasResponse`.
- `provider-connected` kind = one row per **distinct `api_format + base_url`** across all `ApiAliasResponse`s. Computed client-side.

### `GET /bodhi/v1/models/api/{id}` — get API model · **KEEP**
### `POST /bodhi/v1/models/api` — create API model · **KEEP**
### `PUT /bodhi/v1/models/api/{id}` — update API model · **KEEP**
### `DELETE /bodhi/v1/models/api/{id}` — delete API model · **KEEP**
### `POST /bodhi/v1/models/api/test` — test connectivity · **KEEP**
### `POST /bodhi/v1/models/api/fetch-models` — fetch available models from provider · **KEEP**
### `GET /bodhi/v1/models/api/formats` — list supported formats · **KEEP**

```yaml
ApiModelRequest:
  type: object
  required: [api_format, base_url, api_key]
  properties:
    api_format:
      type: string
      enum: [openai, openai_responses, anthropic, anthropic_oauth, gemini]
    base_url:   { type: string, format: uri }
    api_key:
      # ApiKeyUpdate: { type: "keep" } | { type: "set", value: string | null }
      oneOf:
        - { type: object, properties: { type: { enum: [keep] } } }
        - { type: object, properties: { type: { enum: [set] }, value: { type: string, nullable: true } } }
    models:     { type: array, items: { type: string } }
    prefix:     { type: string, nullable: true }
    forward_all_with_prefix: { type: boolean, default: false }
    extra_headers:           { type: object, nullable: true }
    extra_body:              { type: object, nullable: true }

TestPromptRequest:
  type: object
  required: [base_url, model, prompt]
  properties:
    creds:
      oneOf:
        - { type: object, properties: { type: { enum: [api_key] }, value: { type: string, nullable: true } } }
        - { type: object, properties: { type: { enum: [id] }, value: { type: string } } }
    base_url:   { type: string, format: uri }
    model:      { type: string }
    prompt:     { type: string, maxLength: 30 }
    api_format: { type: string, default: openai }

FetchModelsRequest:
  type: object
  required: [base_url]
  properties:
    creds:      { $ref: TestPromptRequest/properties/creds }
    base_url:   { type: string, format: uri }
    api_format: { type: string, default: openai }

FetchModelsResponse:
  type: object
  required: [models]
  properties:
    models: { type: array, items: { type: string } }
```

### `CHANGE` — `GET /bodhi/v1/models/api/formats` should enumerate more formats
Today returns 5: `openai, openai_responses, anthropic, anthropic_oauth, gemini`. Wireframe `API_FORMATS` constant (see `shared-primitives.md §4`) lists **10**: add `openrouter`, `hf-inference`, `nvidia-nim`, `groq`, `together`.

Suggested response shape extension:
```yaml
ApiFormatsResponse:
  type: object
  required: [data]
  properties:
    data:
      type: array
      items:
        type: object
        required: [code, label, default_base_url]
        properties:
          code:              { type: string, example: "openai-completions" }
          label:             { type: string, example: "OpenAI — Completions" }
          default_base_url:  { type: string, format: uri }
          # Optional enrichment (could come from api.getbodhi.app instead):
          auth_mode:         { type: string, enum: [api-key, oauth, none] }
```

**Open question (for user):** Should the extended formats list live on bodhi-server or on api.getbodhi.app? Rationale pro-bodhi-server: it's required to *create* an api-model, and should always be available even when offline. Rationale pro-getbodhi: adding a new provider doesn't require a bodhi-server release. **Recommendation:** keep the canonical `code` / validation enum on bodhi-server (so saves don't fail for unknown codes), but enrich labels/default URLs from api.getbodhi.app.

### `POST /bodhi/v1/models/api/{id}/sync-models` — populate cache · **KEEP** (testing helper)

---

## 5. Local alias CRUD — Create local alias screen
### `POST /bodhi/v1/models/alias` — create · **KEEP**
### `PUT /bodhi/v1/models/alias/{id}` — update · **KEEP**
### `DELETE /bodhi/v1/models/alias/{id}` — delete · **KEEP**
### `POST /bodhi/v1/models/alias/{id}/copy` — copy · **KEEP**

```yaml
UserAliasRequest:
  type: object
  required: [alias, repo, filename]
  properties:
    alias:    { type: string, pattern: "^[a-z0-9][a-z0-9:-]*$", example: "my-qwen-coder" }
    repo:     { type: string, example: "Qwen/Qwen3-Coder-32B-GGUF" }
    filename: { type: string, example: "qwen3-coder-32b.Q4_K_M.gguf" }
    snapshot: { type: string, nullable: true }
    request_params: { $ref: OAIRequestParams, nullable: true }
    context_params: { type: array, items: { type: string }, nullable: true }

OAIRequestParams:
  type: object
  properties:
    temperature:        { type: number, minimum: 0, maximum: 2 }
    top_p:              { type: number, minimum: 0, maximum: 1 }
    max_tokens:         { type: integer, minimum: 1, nullable: true }
    seed:               { type: integer, nullable: true }
    frequency_penalty:  { type: number }
    presence_penalty:   { type: number }
    stop:               { type: array, items: { type: string }, nullable: true }
    user:               { type: string, nullable: true }
    response_format:    { type: string, nullable: true }
    # … (canonical shape mirrors OpenAI chat/completions request params)

CopyAliasRequest:
  type: object
  required: [alias]
  properties:
    alias: { type: string }
```

### `CHANGE` — llama-server args palette
The `ArgsPalette` primitive (see `alias.md §5.5`) lists known llama-server flags with help text. Today this is a static fixture (`ARGS_HELP` constant). Two options:
1. **Static**: ship `ARGS_HELP` as part of the frontend bundle. Simple, but drifts from the actual `llama-server` binary the bodhi-server uses.
2. **Dynamic**: add `GET /bodhi/v1/models/alias/llama-args` returning the parsed `--help` output of the bundled llama-server.

**Recommendation:** option 2, new endpoint (NEW):
```yaml
get:
  path: /bodhi/v1/models/alias/llama-args
  operationId: getLlamaServerArgs
  tags: [models-alias]
  description: |
    NEW. Returns parsed `llama-server --help` so the Create-alias ArgsPalette
    tracks the actual binary. Cacheable (immutable per bodhi-server release).
  responses:
    '200':
      schema:
        type: object
        required: [version, args]
        properties:
          version: { type: string, example: "b3821" }
          args:
            type: array
            items:
              type: object
              required: [flag]
              properties:
                flag:        { type: string, example: "--ctx-size" }
                alias:       { type: string, nullable: true, example: "-c" }
                value_type:  { type: string, enum: [int, float, string, bool, enum], nullable: true }
                default:     { type: string, nullable: true }
                description: { type: string }
                group:       { type: string, nullable: true, enum: [sampling, server, gpu, batching, logging] }
```

### `CHANGE` — FitCheckCard prediction
The `FitCheckCard` in Create-alias needs "will this fit? expected tok/s". Today the wireframe shows a static computed hint. Two paths:
1. Compute client-side from `model_params.size_bytes` + known hardware.
2. Add a server endpoint that runs an internal estimator.

**Recommendation:** optional `POST /bodhi/v1/models/alias/fit-check` (NEW). Stub for this pass; not a blocker.

```yaml
post:
  path: /bodhi/v1/models/alias/fit-check
  operationId: aliasFitCheck
  tags: [models-alias]
  description: NEW (optional). Quick fit-prediction for the Create-alias FitCheckCard.
  requestBody:
    schema:
      type: object
      required: [repo, filename]
      properties:
        repo:     { type: string }
        filename: { type: string }
        snapshot: { type: string, nullable: true }
  responses:
    '200':
      schema:
        type: object
        required: [fit, expected_tok_s, gpu_layers_fit]
        properties:
          fit:             { type: string, enum: [green, yellow, red] }
          expected_tok_s:  { type: number, description: "est. tokens/sec on detected rig" }
          gpu_layers_fit:  { type: integer }
          notes:           { type: string, nullable: true }
```

---

## 6. Metadata / queue — already implemented
### `POST /bodhi/v1/models/refresh` — refresh metadata · **KEEP**
### `GET /bodhi/v1/queue` — queue status · **KEEP**

These drive the implicit "metadata fresh" indicator in the Models list. Existing shapes are fine.

---

## 7. Summary of required bodhi-server changes
- **Must-have (not blocking migration, but improves UX):**
  - Extended `GET /bodhi/v1/models/api/formats` returning 10 formats (or delegate to api.getbodhi.app for the label/default-url enrichment, keeping the enum canonical on bodhi-server).
- **Nice-to-have (can defer):**
  - Kind/source filter params on `GET /bodhi/v1/models`.
  - Download progress SSE stream.
  - `GET /bodhi/v1/models/alias/llama-args` for dynamic ArgsPalette.
  - `POST /bodhi/v1/models/alias/fit-check` for FitCheckCard.

Everything else is already in place. The migration is unblocked if **none** of these land in the first pass; they're quality-of-life.

## 8. Out of scope on bodhi-server
These live elsewhere (see `20-getbodhi-public-apis.md`):
- Provider directory (`provider-unconnected` rows).
- Curated leaderboards / benchmark scores.
- Specialization → benchmark mapping.
- Trending / new launches.
- HF repo search + metadata enrichment (handled directly by frontend via HF; see `30-huggingface-integration.md`).
