# Plan: `gemini` API Format Support — As-Built

> Squash-merged single commit. This document records the final delivered state plus the upfront plan and the points where reality diverged. Companion: `ai-docs/claude-plans/202604/20260413-gemini-api-format/TECHDEBT.md`.

## Context

Google's Gemini API uses a different shape than OpenAI/Anthropic: auth via `x-goog-api-key` header, action-style URLs (`/v1beta/models/{model}:generateContent`), and a distinct request body (`contents[].parts[].text` + optional `systemInstruction`). We added `ApiFormat::Gemini`, mirrored the existing multi-provider scaffolding (OpenAI, OpenAIResponses, Anthropic, AnthropicOAuth) for Gemini, and exposed the upstream API verbatim at `/v1beta/*`. No `/gemini/` wrapper — `/v1beta/*` doesn't collide with OpenAI's `/v1/*` or Anthropic's `/anthropic/v1/*`.

### Templates mirrored

The Anthropic stack landed earlier and was mirrored structurally for Gemini:

- Anthropic OpenAPI sync pipeline → `sync-gemini-openapi.mjs`, `gemini-openapi-filter.yaml`, `openapi-gemini.json`.
- `ApiFormat::Anthropic` + `LlmEndpoint` + opaque proxy → `ApiFormat::Gemini`, `LlmEndpoint::Gemini*`, `routes_gemini.rs`.
- `AIProviderClient` strategy + `ApiModel` discriminated union → `GeminiProviderClient`, `ApiModel::Gemini(GeminiModel)`.
- `AnthropicOAuth` with `anthropic_auth_middleware` + pi-ai wiring → `gemini_auth_middleware`, frontend preset, pi-ai `google-generative-ai` API.

### Clarifications adopted from user

1. Routes mount directly at `/v1beta/*` (no `/gemini/*` wrapper).
2. **Original**: model IDs stored stripped (`gemini-2.5-flash`); `GeminiModel` had `id` field with custom serde stripping/restoring `models/`.
   **Final**: schema mirrors Google's `Model` exactly — both `name` and `base_model_id` are present (both required per spec); `name` is always derived as `models/{base_model_id}`; the strip happens at one explicit point (`GeminiProviderClient::models`), no custom serde.
3. Filtered OpenAPI spec keeps 5 endpoints: `models.list`, `models.get`, `models.generateContent`, `models.streamGenerateContent`, `models.embedContent`.
4. `/bodhi/v1/models` (and `/v1/models`) surface Gemini aliases — covered by `test_oai_models_handler_gemini_alias_included_with_prefix` and `_without_prefix`.
5. `/ui/chat` pi-ai streams Gemini natively via `@mariozechner/pi-ai`'s `google-generative-ai` API. `gemini_auth_middleware` strips `SENTINEL_API_KEY` from `x-goog-api-key`.
6. Loop-driven E2E specs auto-pick up Gemini via `API_FORMATS` extension. Specs that are NOT loop-driven (`api-models-extras.spec.mjs` is anthropic_oauth-only; `api-models-forward-all.spec.mjs` and `api-models-prefix.spec.mjs` are OpenAI-only) deliberately left as-is — see Phase 9 deviation notes.
7. `LLMock` extended with a `/v1beta` mount handler for Gemini paths.
8. Embeddings E2E spec added at `specs/api-models/api-gemini-embeddings.spec.mjs`.
9. New chat-UI spec added at `specs/chat/chat-gemini.spec.mjs`.

---

## Architecture: how Gemini slots in (final)

### Strategy pattern (after this work)

- `ApiFormat` enum in `crates/services/src/models/model_objs.rs` has 5 variants (added `Gemini`).
- `AiApiService` dispatches per-variant in three methods: `test_prompt`, `fetch_models`, `forward_request_with_method`.
- Per-variant `AIProviderClient` impl lives in `crates/services/src/ai_apis/ai_provider_client.rs`.
- `ApiModel` is a `#[serde(tag="provider")]` discriminated union with `Gemini(GeminiModel)` variant.
- `LlmEndpoint` has 5 new Gemini variants.
- Routes live in `crates/routes_app/src/gemini/` using opaque-proxy handlers (no utoipa annotations; docs come from `resources/openapi-gemini.json`).
- Chat UI sentinel: pi-ai sends `SENTINEL_API_KEY` as dummy credential; `gemini_auth_middleware` strips it on `/v1beta/*` so session-cookie auth falls through.
- Frontend `API_FORMAT_PRESETS` adds `gemini` entry. No `defaultHeaders`/`defaultBody` — extras editor stays hidden.
- Chat store maps `apiFormat='gemini'` → pi-ai `'google-generative-ai'` provider with `baseUrl=${origin}/v1beta`.

### Gemini-specific deltas vs the Anthropic template

| Area | Anthropic | Gemini |
|------|-----------|--------|
| Auth header | `x-api-key` | `x-goog-api-key` |
| URL mount | `/anthropic/v1/*` (also `/v1/messages`) | `/v1beta/*` |
| Action-style URLs | None | Yes (`/models/{id}:generateContent`) |
| Models-list response | `{data: [...], has_more, first_id, last_id}` | `{models: [{name: "models/X", baseModelId: "X", supportedGenerationMethods, ...}]}` |
| Model ID in request | JSON body `model` field | URL path segment |
| Request body | `{messages, max_tokens}` | `{contents, systemInstruction}` |
| Prefix strip | In shared `forward_to_upstream` on JSON `model` field | `gemini_action_handler` strips alias prefix from URL path before building `LlmEndpoint::Gemini*(stripped_model)` |
| OpenAI `/v1/chat/completions` compat | Accepted | **Rejected** (format error — no translation in scope) |
| pi-ai API | `anthropic-messages` | `google-generative-ai` |
| Auth middleware | `anthropic_auth_middleware` rewrites `x-api-key` → Bearer + strips SENTINEL | `gemini_auth_middleware` strips SENTINEL from `x-goog-api-key` + rewrites real key → Bearer on `/v1beta/*` |
| Streaming | SSE via Anthropic SDK (Accept header) | SSE via `?alt=sse` query param — **forwarded explicitly** by `gemini_action_handler` |
| `models/` prefix | N/A | Stripped at one point only (`GeminiProviderClient::models`); reapplied at one point (`gemini_wire_format` route helper) |

---

## Layered build-out — as built

The work landed upstream-to-downstream through the BodhiApp dependency chain. Each layer below describes the final state of the diff in that layer plus any deviation from the upfront plan and why.

### 1. OpenAPI sync pipeline (`ts-client/`, `Makefile`)

**As built**: `ts-client/scripts/sync-gemini-openapi.mjs` fetches Google's discovery doc, hashes the response body to skip regeneration when unchanged, and filters via `gemini-openapi-filter.yaml` down to 5 endpoints (`models.list`, `models.get`, `models.generateContent`, `models.streamGenerateContent`, `models.embedContent`). Outputs `ts-client/openapi-gemini.json` and copies to `crates/routes_app/resources/openapi-gemini.json`. New hey-api config `openapi-ts-gemini.config.ts` generates `src/types-gemini/`. New re-export `src/gemini.ts`. `Makefile` `openapi.gemini` target. `package.json` scripts and dist subpath export. **No deviation from plan.**

### 2. Services — domain types (`crates/services/src/models/`)

**As built**:
- `model_objs.rs`: `ApiFormat::Gemini` variant, `ApiModel::Gemini(GeminiModel)` variant in the discriminated union, `ApiModel::id()` returns `&base_model_id` for Gemini.
- `gemini_model.rs` (NEW): `GeminiModel` mirrors Google's `Model` schema (`openapi-gemini.json:5423-5494`) — both `name` and `base_model_id` are required fields, plus `maxTemperature`, `thinking`, and the other spec fields. Constructor `from_base_model_id()` enforces the invariant `name == format!("models/{}", base_model_id)`. **No custom serde.**
- `inference/inference_service.rs`: 5 new `LlmEndpoint::Gemini*` variants (`GeminiModels`, `GeminiModel(String)`, `GeminiGenerateContent(String)`, `GeminiStreamGenerateContent(String)`, `GeminiEmbedContent(String)`).
- `test_model_objs.rs`: `ApiFormat::Gemini` round-trip case + `ApiModel::Gemini` round-trip case.

**Deviation**: original plan had `GeminiModel { id, ... }` with custom serde stripping/restoring `models/`. That overloaded the upstream wire shape as the canonical serde representation and bled `models/X` through internal surfaces (`/bodhi/v1/models` → alias selector → frontend concatenated `prefix + name` and got `gem/models/gemini-flash-latest` instead of `gem/gemini-flash-latest`). Final design separates the layers: `base_model_id` is the internal source of truth; `name` is its derived projection kept in the struct so `@google/genai` SDK introspection of our `/v1beta/models` response works unchanged. The strip happens at one explicit boundary (next layer), not in serde.

### 3. Services — provider client + dispatch (`crates/services/src/ai_apis/`)

**As built**:
- `ai_provider_client.rs`: `GeminiProviderClient` with `x-goog-api-key` auth, contains the **single explicit translation point**: per upstream model entry, derive `base_model_id = name.strip_prefix("models/")`, then re-emit `name = format!("models/{}", base_model_id)` to enforce the struct invariant. Third-party Gemini-compatible hosts that don't use the prefix work too (strip is a no-op).
- `ai_api_service.rs`: `ApiFormat::Gemini` arms in `test_prompt`, `fetch_models`, `forward_request_with_method`. `extra_headers`/`extra_body` accepted but unused (mirrors OpenAI arm).
- `test_ai_api_gemini.rs`: `test_test_prompt_gemini_success` (mockito assertion of `x-goog-api-key` header + body shape), `test_fetch_models_gemini_filters_non_generate` (locks down the `models/` strip + `supportedGenerationMethods` filter), `test_forward_request_gemini_passes_through`.

**No deviation from plan.**

### 4. Services — repository + ApiModelService tests

**As built**: `test_api_alias_repository.rs` Gemini round-trip case (SQLite + PostgreSQL via `#[values]`), `test_api_model_service.rs` parameterized with Gemini for create/update/forward-all/selected-models flows, `test_utils/fixtures.rs` `gemini_model(id)` factory.

**No deviation from plan.**

### 5. Routes — `/v1beta/*` proxy + auth middleware (`crates/routes_app/src/gemini/`, `middleware/`)

**As built**:
- `gemini/mod.rs`: route constants `ENDPOINT_GEMINI_MODELS = "/v1beta/models"` and `ENDPOINT_GEMINI_MODEL = "/v1beta/models/{*model_path}"` (Axum 0.8 wildcard).
- `gemini/gemini_api_schemas.rs`: `GeminiApiError` using Google's error shape (`{error: {code, message, status}}`) with `invalid_request`, `missing_model`, `not_found`, `forbidden` constructors.
- `gemini/routes_gemini.rs`:
  - `gemini_models_list`: aggregates Gemini aliases' models, dedupes by aliased id, emits via `gemini_wire_format()` — overrides `name = "models/{prefix}{base_model_id}"` and `baseModelId = "{prefix}{base_model_id}"` so the `@google/genai` SDK consumes the response unchanged.
  - `gemini_models_get(Path<String>)`: local metadata lookup, returns same wire format.
  - `gemini_action_handler(Path<String>, Query<HashMap<String, String>>, Json<Value>)`: splits the path on the LAST `:` to separate model from action; validates action against `["generateContent", "streamGenerateContent", "embedContent"]`; resolves alias via `find_alias`; strips alias prefix; constructs `LlmEndpoint::Gemini*(stripped_model)`; **forwards all query params verbatim to upstream** so `@google/genai`'s `?alt=sse` round-trips (without this, Google returns a JSON array instead of SSE chunks and the client parser fails with "incomplete json segment").
  - Helpers: `resolve_gemini_alias`, `list_user_gemini_aliases`, `strip_alias_prefix`, `validate_model_id` (allows `/` for prefixed aliases).
- `gemini/test_gemini_routes.rs`: 36 tests including `test_action_handler_accepts_literal_slash_in_prefixed_alias` (locks down wildcard routing for prefixed aliases) and `test_action_handler_forwards_alt_sse_query_param` (locks down query param forwarding).
- `middleware/gemini_auth_middleware.rs` (NEW): on `/v1beta/*`, strips `SENTINEL_API_KEY` from `x-api-key`/Bearer/`x-goog-api-key`; if request has real `x-goog-api-key` and no `Authorization`, rewrites to `Authorization: Bearer <value>` (mirrors `anthropic_auth_middleware`'s `x-api-key` rewrite).
- `routes.rs`: `gemini_apis` router group with `from_fn(gemini_auth_middleware)` outermost layer, merged into `api_protected`. Same `api_auth_middleware(ResourceRole::User, …)` as `anthropic_apis`.
- `shared/constants.rs`: `API_TAG_GEMINI`. `shared/openapi.rs`: registers `openapi-gemini.json` as a swagger-ui tab.
- `oai/test_models.rs`: `test_oai_models_handler_gemini_alias_included_with_prefix` and `_without_prefix` lock down `/v1/models` and `/bodhi/v1/models` surfacing of Gemini aliases with bare ids (no `models/` leak).
- `oai/test_chat_completions.rs`: `test_chat_completions_rejects_gemini_alias` asserts `/v1/chat/completions` rejects Gemini aliases with format-mismatch error.

**Deviations from plan**:
1. **Wildcard route, not three literal-`:` routes**. Plan considered three POST routes with paths like `/v1beta/models/{m}:generateContent`. Reality: Axum 0.8's `{id}` matcher is single-segment, and prefixed aliases like `gem/gemini-flash-latest` produce multi-segment paths that 404 against `{id}`. Single wildcard `{*model_path}` cleanly handles GET (model lookup) and POST (action dispatch) with the action split done in the handler.
2. **Pre-existing `test_generate_content_strips_alias_prefix` was masking the wildcard bug** by URL-encoding the slash as `%2F` (decodes inside a single segment). New `test_action_handler_accepts_literal_slash_in_prefixed_alias` uses an unencoded `/` to reproduce production behavior.
3. **`?alt=sse` query forwarding** was not in the original plan. Discovered during browser verification — without it, Google returns a JSON array (`[{...},{...}]`) instead of SSE chunks (`data: {...}\r\n\r\n`), breaking the SDK's SSE parser.

### 6. Routes — formats endpoint

**As built**: `routes_api_models.rs` `api_models_formats` handler now returns all 5 formats including Gemini. New test `test_api_models_formats_includes_all_five`.

**Deviation**: original plan called for extending `#[case(ApiFormat::…)]` parameterization in `test_api_models_crud.rs`, `test_api_models_prefix.rs`, `test_api_models_sync.rs`, `test_api_models_validation.rs`, `test_api_models_isolation.rs`. On inspection none of those `routes_app` model test files actually had `ApiFormat`-parameterized cases — the plan's premise was wrong. Service-layer parameterization in `services::test_api_model_service` covers the cross-format CRUD behavior; this layer just owns the formats list endpoint.

### 7. OpenAPI + TS client regeneration

**As built**: `cargo run --package xtask openapi` and `make build.ts-client` produce updated `openapi.json` (`ApiFormat` lists `"gemini"`, `ApiModel` one-of includes Gemini variant, `GeminiModel` schema has both `name` and `baseModelId`) and `ts-client/src/types/types.gen.ts` (matching). **No deviation.**

### 8. Frontend — schema, form, chat (`crates/bodhi/src/`)

**As built**:
- `schemas/apiModel.ts`: `gemini` preset (`name: 'Google Gemini'`, `baseUrl: 'https://generativelanguage.googleapis.com/v1beta'`). No `defaultHeaders`/`defaultBody` — extras editor stays hidden automatically (preset-driven visibility). `getApiModelId(m)` returns `m.baseModelId` for Gemini variants.
- `components/api-models/providers/constants.ts`: Gemini provider entry (`id: 'gemini'`, icon, common models, doc URL).
- `stores/agentStore.ts`:
  - `apiFormatToPiApi('gemini') → 'google-generative-ai'`
  - `apiFormatToProvider('gemini') → 'google'`
  - `getBaseUrl('gemini') → ${origin}/v1beta` (pi-ai's google provider sets `httpOptions.baseUrl = model.baseUrl` + `apiVersion = ""`, so the `@google/genai` SDK appends `/models/{id}:generateContent` to land on our proxy).
  - `buildModel().maxTokens = 32000` (was `0`).
- Tests: `agentStore.test.ts` Gemini routing case; `ApiModelForm.extras.test.tsx` "extras hidden for gemini" case.

**Deviation**: `maxTokens: 32000` was not in the plan. Discovered during browser verification — pi-ai's google provider falls back to `Math.min(model.maxTokens, 32000)` when `options.maxTokens` is unset (slider OFF), producing `0`, which Gemini rejects with `"max_output_tokens must be positive"`. Matching the pi-ai Anthropic ceiling (`32000`) avoids the issue; the slider still overrides via `options.maxTokens`.

### 9. E2E (`crates/lib_bodhiserver_napi/tests-js/`)

**As built**:
- `fixtures/apiModelFixtures.mjs`: Gemini entry with all standard fields plus mock-specific `mockBaseUrlSuffix`, `mockModel`, `mockSecondaryModel`. Disambiguation prefix `multiTestPrefix: 'gmn/'`.
- `.env.test.example`: `INTEG_TEST_GEMINI_API_KEY` stub.
- `pages/components/ApiModelFormComponent.mjs`: Gemini display name + auto-filled base URL assertion.
- `specs/api-models/api-models-no-key.spec.mjs`: `LLMock` extended with a `/v1beta` mount handler — `GET /v1beta/models` returns Gemini-shape models list, `POST :generateContent` returns canned JSON, `POST :streamGenerateContent` returns SSE chunks. Hardcoded mock URLs and model names replaced with format-aware accessors so the existing `API_FORMATS` loop covers Gemini.
- `specs/api-models/api-live-upstream.spec.mjs`: already loop-driven over `API_FORMATS`, so Gemini auto-gets coverage including prefixed aliases (uses `multiTestPrefix: 'gmn/'`), primary endpoints, and chat UI invocation. This is the regression test for the wildcard routing fix.
- `specs/chat/chat-gemini.spec.mjs` (NEW): live spec — login, create Gemini alias via UI form, chat through `/ui/chat/`, assert reply contains expected token.
- `specs/api-models/api-gemini-embeddings.spec.mjs` (NEW): live spec — create Gemini embedding alias, mint app API token, POST `/v1beta/models/{id}:embedContent` via `fetchWithBearer`, assert non-empty embedding vector.

**Deviations**:
- `api-models-extras.spec.mjs` is anthropic_oauth-only (not loop-driven) — extras-hidden behavior for Gemini is covered by the frontend unit test `ApiModelForm.extras.test.tsx` instead. No E2E added.
- `api-models-forward-all.spec.mjs` and `api-models-prefix.spec.mjs` are OpenAI-only — left as-is.
- Mock variants of the new chat/embeddings specs were deferred. The live specs give stronger coverage; `LLMock` SSE mocking can be added as follow-up if CI cost becomes an issue.

---

## Final file inventory

| File | Status |
|------|--------|
| `ts-client/scripts/sync-gemini-openapi.mjs`, `gemini-openapi-filter.yaml`, `openapi-ts-gemini.config.ts`, `src/gemini.ts`, `src/types-gemini/*`, `src/openapi-typescript/openapi-schema-gemini.ts` | NEW |
| `crates/routes_app/resources/openapi-gemini.json` | NEW (filtered spec) |
| `crates/services/src/models/model_objs.rs` | `ApiFormat::Gemini` + `ApiModel::Gemini`; `id()` returns `&base_model_id` for Gemini |
| `crates/services/src/models/gemini_model.rs` | NEW — schema mirrors Google `Model` (both `name` + `base_model_id`); invariant via `from_base_model_id()` |
| `crates/services/src/inference/inference_service.rs` | 5 new `LlmEndpoint::Gemini*` variants |
| `crates/services/src/ai_apis/ai_provider_client.rs` | `GeminiProviderClient`; `models()` is the single `models/` strip boundary |
| `crates/services/src/ai_apis/ai_api_service.rs` | `ApiFormat::Gemini` dispatch arms |
| `crates/services/src/test_utils/fixtures.rs` | `gemini_model(id)` factory |
| `crates/routes_app/src/gemini/mod.rs` | `ENDPOINT_GEMINI_MODELS = "/v1beta/models"`, `ENDPOINT_GEMINI_MODEL = "/v1beta/models/{*model_path}"` (wildcard) |
| `crates/routes_app/src/gemini/routes_gemini.rs` | 5 handlers + `gemini_wire_format` egress helper; `gemini_action_handler` extracts `Query<HashMap>` and forwards all query params (e.g. `?alt=sse`) |
| `crates/routes_app/src/gemini/gemini_api_schemas.rs` | `GeminiApiError` (Google error shape) |
| `crates/routes_app/src/gemini/test_gemini_routes.rs` | 36 tests including new `test_action_handler_accepts_literal_slash_in_prefixed_alias` and `test_action_handler_forwards_alt_sse_query_param` |
| `crates/routes_app/src/middleware/gemini_auth_middleware.rs` | NEW — SENTINEL strip + `x-goog-api-key` → Bearer rewrite on `/v1beta/*` |
| `crates/routes_app/src/routes.rs` | `gemini_apis` router group + middleware wiring; merged into `api_protected` |
| `crates/routes_app/src/models/api/routes_api_models.rs` | `api_models_formats` returns all 5 formats |
| `crates/routes_app/src/oai/test_models.rs`, `test_chat_completions.rs` | Gemini surfacing on `/bodhi/v1/models` + `/v1/chat/completions` rejection of Gemini aliases |
| `crates/routes_app/src/shared/constants.rs`, `shared/openapi.rs` | `API_TAG_GEMINI`, swagger registration |
| `crates/bodhi/src/schemas/apiModel.ts` | `gemini` preset; `getApiModelId` reads `m.baseModelId` |
| `crates/bodhi/src/components/api-models/providers/constants.ts` | Gemini provider entry |
| `crates/bodhi/src/stores/agentStore.ts` | pi-ai `google-generative-ai` wiring; `getBaseUrl('gemini') → ${origin}/v1beta`; `buildModel().maxTokens = 32000` |
| `crates/bodhi/src/components/api-models/ApiModelForm.extras.test.tsx` | Test asserting extras hidden for Gemini |
| `crates/bodhi/src/stores/agentStore.test.ts` | Gemini routing test |
| `crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs` | Gemini fixture entry |
| `crates/lib_bodhiserver_napi/tests-js/.env.test.example` | `INTEG_TEST_GEMINI_API_KEY` |
| `crates/lib_bodhiserver_napi/tests-js/pages/components/ApiModelFormComponent.mjs` | Display name + baseUrl assertion |
| `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-no-key.spec.mjs` | aimock `/v1beta/*` handlers; format-aware mock URLs |
| `crates/lib_bodhiserver_napi/tests-js/specs/chat/chat-gemini.spec.mjs` | NEW — live chat UI + Gemini alias creation |
| `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-gemini-embeddings.spec.mjs` | NEW — live embeddings via app API token |
| `Makefile` | `openapi.gemini` target |
| `ai-docs/claude-plans/202604/20260413-gemini-api-format/TECHDEBT.md` | NEW — documents `models/` prefix root cause + remediation, `maxOutputTokens=0` workaround, wildcard route, test masking from URL-encoded paths, CSP/MCP unrelated noise |

---

## Verification (end-to-end, all green)

1. `cargo test --lib -p services -p routes_app` — 906 + 766 pass
2. `cd ts-client && npm test` — green
3. `cd crates/bodhi && npm test` — green (`ApiModelForm`, `agentStore` parameterized across all 5 formats)
4. `cargo run --package xtask openapi && make build.ts-client` — `ApiFormat` has `"gemini"`; `GeminiModel` type generated with both `name` and `baseModelId` fields
5. `make build.ui-rebuild` — UI rebuilt
6. Manual browser test (after restart):
   - Create Gemini alias (auto-filled base URL, api-key field, no extras section)
   - `/bodhi/v1/models` returns Gemini models with bare `baseModelId` and derived `name: "models/{baseModelId}"`
   - `/v1/models` returns Gemini aliases with prefixed bare ids (e.g. `gem/gemini-flash-latest`)
   - Select Gemini alias in chat, send prompt → request URL `/v1beta/models/gem/gemini-flash-latest:streamGenerateContent?alt=sse` (literal `/`, no extra `models/`); body has `maxOutputTokens: 32000`; response streams as SSE chunks; `x-goog-api-key: SENTINEL…` header stripped by middleware; cookie auth succeeds
   - Live E2E `api-live-upstream.spec.mjs` (with `INTEG_TEST_GEMINI_API_KEY`) covers the same end-to-end flow including the prefixed-alias routing path

---

## Lessons captured (see TECHDEBT.md for details)

1. **Don't overload upstream wire format as canonical serde shape**. Originally `GeminiModel` deserialized `name: "models/X"` into `id` and the same custom serde reserialized for output. This bled the `models/` prefix into internal API surfaces. Final design separates the two layers (`base_model_id` internal source of truth; `name` derived for SDK compat) and concentrates translation in one explicit boundary.

2. **Wildcard routes for action-style URLs with prefixed identifiers**. Axum's `{id}` is single-segment. Prefixed aliases produce multi-segment paths. Either URL-encode or use `{*name}` — the test that used `%2F` masked the production behavior of literal `/`. Always exercise prefix scenarios with literal slashes.

3. **Forward query params on opaque proxies**. The `@google/genai` SDK signals SSE via `?alt=sse`. Dropping it changes Google's response shape from event-stream to JSON array, which the client-side SSE parser fails on. Opaque proxies must forward query params (and SSE-relevant headers) verbatim.

4. **pi-ai's `Math.min(model.maxTokens, 32000)` fallback**. When the slider is OFF and `model.maxTokens === 0`, Gemini rejects `maxOutputTokens: 0`. Setting a sensible per-model default at the chat-store boundary works around it; a longer-term fix is in pi-ai (`??` instead of `||`) — tracked in TECHDEBT.

5. **Plans should be verified, not trusted**. Phase 6's "extend parameterized cases (all already use `#[case(ApiFormat::…)]`)" was wrong — those test files weren't parameterized. The grep takes 30 seconds; the failed sub-agent invocation cost much more. Always verify the premise before dispatching work.

---

## Post-Review Follow-ups (2026-04-13)

A code review over the squash-merged commit surfaced one correctness bug and several important gaps. Fixes landed in a follow-up change (plan: `ai-docs/claude-plans/202604/witty-hopping-piglet.md`; reviews: `ai-docs/claude-plans/202604/20260411-anthropic-oauth/reviews/index.md`). This section folds the substantive changes into the as-built record so the Gemini doc stays the single source of truth.

### 1. Fixed: Gemini alias matching (routing correctness)

**Symptom**: Non-`forward_all_with_prefix` Gemini aliases with a prefix (e.g. `gmn/`) could not be matched by client requests. The stored `GeminiModel.name` had the alias prefix baked in at `create()`/`update()` time (`models/gmn/gemini-2.5-flash`); `model_id()` stripped only `models/` returning `gmn/gemini-2.5-flash`; then `ApiAlias::matchable_models()` prepended the prefix again producing `gmn/gmn/gemini-2.5-flash`. Clients sending `gmn/gemini-2.5-flash` never matched.

**Fix**: Stop baking the alias prefix into `GeminiModel.name` at create/update. Storage now always holds the bare Google-canonical form: `name = "models/gemini-2.5-flash"`, `base_model_id = "gemini-2.5-flash"` (or for `GeminiModel` variants that expose `model_id()`, the bare id). The alias-level `prefix` alone drives matching via `ApiAlias::matchable_models()`.

**Downstream adjustments**:
- `crates/services/src/models/api_model_service.rs` — removed the `if let Some(prefix) = form.prefix { ... m.name = format!("models/{prefix}{bare}") }` blocks in both `create()` and `update()`.
- `crates/routes_app/src/gemini/routes_gemini.rs` — `resolve_gemini_alias` now matches against `alias.matchable_models()` instead of `m.model_id()` directly.
- `crates/routes_app/src/gemini/routes_gemini.rs` — `gemini_models_list` and `gemini_models_get` now project the stored bare model into the wire shape at response time: `name = "models/{prefix}{model_id}"`. New helper `gemini_model_to_json(m, prefix)` centralises the rewrite. Dedup key in `list` uses `{prefix}{model_id}` (not bare) so two Gemini aliases with different prefixes both surface.

**Storage invariant (final)**:

| Format | Alias prefix | Stored | `m.id()` / `m.model_id()` | `matchable_models()` |
|---|---|---|---|---|
| Gemini | none | `name: "models/gemini-2.5-flash"` | `gemini-2.5-flash` | `["gemini-2.5-flash"]` |
| Gemini | `gmn/` | `name: "models/gemini-2.5-flash"` | `gemini-2.5-flash` | `["gmn/gemini-2.5-flash"]` |
| OpenAI | none | `id: "gpt-4.1"` | `gpt-4.1` | `["gpt-4.1"]` |
| OpenAI | `oai/` | `id: "gpt-4.1"` | `gpt-4.1` | `["oai/gpt-4.1"]` |

**Test coverage**: Renamed `test_create_gemini_bakes_prefix_into_name` → `test_create_gemini_preserves_bare_name` (and the update equivalent), asserting the bare name + `matchable_models() == ["gmn/gemini-2.5-flash"]` + `supports_model("gmn/gemini-2.5-flash")` + `!supports_model("gmn/gmn/gemini-2.5-flash")` (the regression guard). Repository round-trip test `test_api_model_alias_gemini_prefixed_roundtrip` updated similarly. The route-level `test_gemini_models_list_returns_prefixed_name` now documents that the bare stored model is projected to the prefixed wire form at read time.

### 2. Fixed: 5xx error message leakage (GeminiApiError + mirrored Anthropic)

**Symptom**: `From<ApiError> for GeminiApiError` passed `AppError::to_string()` through for all statuses, so 5xx error bodies could carry internal DB/service detail in the Gemini-SDK-facing `error.message` field. Same pattern existed in `AnthropicApiError`.

**Fix**: For `http_status >= 500`, substitute the generic string `"internal server error"`; 4xx paths keep actionable messages (both provider SDKs rely on those). Applied symmetrically to `crates/routes_app/src/gemini/gemini_api_schemas.rs` and `crates/routes_app/src/anthropic/anthropic_api_schemas.rs`. New unit tests `test_5xx_message_is_generic_not_internal_detail` + `test_4xx_message_is_preserved` in both `test_gemini_api_schemas.rs` and `test_anthropic_api_schemas.rs`.

**Scope-of-envelope confirmation**: grep verified that `GeminiApiError` is used only under `crates/routes_app/src/gemini/` (and `AnthropicApiError` only under `anthropic/`). All other routes use `ApiError` / `OpenAIApiError`.

### 3. Fixed: `x-goog-*` header forwarding

**Symptom**: `gemini_action_handler` extracted `HeaderMap` as `_headers: HeaderMap` and discarded it. Google SDK clients set `x-goog-api-client` and `x-goog-request-params`; these never reached upstream.

**Fix**: New helper `extract_gemini_headers(&HeaderMap) -> Option<Vec<(String, String)>>` in `routes_gemini.rs` (case-insensitive `x-goog-*` filter; mirrors `extract_anthropic_headers`). The handler now calls it and threads `client_headers` into `forward_remote_with_params`. Test `test_action_handler_forwards_x_goog_headers` asserts both inclusion of `x-goog-*` headers and exclusion of non-`x-goog-*` headers (e.g. `content-type`).

### 4. Added: Provider test matrix covers Gemini

`crates/services/src/ai_apis/test_ai_api_provider_matrix.rs`:
- `test_prompt_401_unauthorized` gained a Gemini case. The matrix column now carries both `path` and `model` so the Gemini row can use `/models/gemini-2.5-flash:generateContent` + bare model id while the other rows keep `/chat/completions` etc. + `some-model`.
- `test_forward_passthrough` gained a Gemini case (no parameter changes — same `/chat/completions` passthrough semantics).

### 5. Simplified: UI `handleApiFormatChange` (reset-on-dirty semantics)

**User-stated rule**: no dirty-tracking, no revert-to-initial — any format change resets the form to the selected format's preset defaults. User must cancel or refresh to restore stored values.

**Change** (`crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`): `handleApiFormatChange` is now a single unconditional reset block. Dropped the `isEditMode && initialData && apiFormat !== initialData.api_format` branch that previously forced `useApiKey=true` for the same-format-revert case. `base_url`, `extra_headers`, `extra_body` are now always reset — `preset?.baseUrl ?? ''` (empty when no preset), similarly for extras. This closes the case where switching to a preset-less format silently inherited the previous `base_url`.

**Test update**: `ApiModelForm.extras.test.tsx` test `switching api_format in edit mode forces useApiKey=true (cannot Keep stored key)` renamed to `switching api_format in edit mode resets form to preset defaults` and now asserts `useApiKey` is unchecked after the switch (plus the preset-derived extras still appear).

### 6. Rejected findings (recorded)

- **I1 (useApiKey revert)**: User confirmed reset-on-dirty is the correct behavior; no revert-to-initial logic will be added. The implemented behavior in §5 matches this rule.
- **I7 (`.env` quote stripping in `sync-*-openapi.mjs`)**: Sync failing loudly on malformed env lines is the correct signal. No silent quote-stripping will be added.

### Verification after follow-ups

- `cargo test -p services -p routes_app --lib` — 938 + 778 green (previously 906 + 766 before new tests landed)
- `cd crates/bodhi && npm test` — 903 pass (1 skipped pre-existing)
- New test inventory:
  - services: `test_create_gemini_preserves_bare_name` + matchable_models/supports_model assertions
  - services: `test_update_gemini_preserves_bare_name` + matchable_models/supports_model assertions
  - services: `test_prompt_401_unauthorized::case_5_gemini`
  - services: `test_forward_passthrough::case_5_gemini`
  - routes_app: `test_5xx_message_is_generic_not_internal_detail` + `test_4xx_message_is_preserved` (×2 — Gemini + Anthropic)
  - routes_app: `test_action_handler_forwards_x_goog_headers`
  - ui: `switching api_format in edit mode resets form to preset defaults` (rewritten)

### Remaining nice-to-haves

**Closed 2026-04-14.** All 13 N-items + T7 from the review landed in a follow-up pass. Changes by area:

- **services (N1)**: `GeminiModel.version` changed from `String` → `Option<String>` so missing upstream values stay `None` instead of becoming `""`. Test fixture and serde round-trip tests updated.
- **routes_app (N2)**: Added `test_action_handler_returns_not_found_for_unknown_alias` exercising `From<ApiError> for GeminiApiError` → 404 with `NOT_FOUND` grpc status.
- **routes_app (N3)**: Removed dead `GeminiApiError::forbidden` constructor and its test — the blanket `From<ApiError>` handles 403s.
- **ui (N4)**: `getApiModelId` now discriminates via `m.provider !== 'gemini'` instead of `'id' in m` — robust if Gemini ever grows an `id` field upstream.
- **ui (N5)**: `AliasSelector` memoizes `selectedAlias` / `selectedApiFormat` via `useMemo(… , [modelToAliasMap, model])` and the `useEffect` dep-array was narrowed to the derived values, eliminating spurious `setApiFormat` writes on every query refetch.
- **ui (N6)**: `agentStore.restoreMessagesForChat` now uses an `isAgentMessage(x)` type predicate requiring `role` + `api` + `provider`. Structurally incomplete stored messages are dropped (with a `console.warn`) via `flatMap` instead of being cast unchecked. New test in `agentStore.test.ts`.
- **ts-client (N7)**: Removed redundant dynamic `await import('node:fs')` in both `sync-gemini-openapi.mjs` and `sync-anthropic-openapi.mjs` — `unlinkSync` now comes from the existing top-level import and cleanup failures propagate (stale tmp files won't linger silently).
- **ts-client (N8)**: Added `- schemas` to `gemini-openapi-filter.yaml` `unusedComponents`. Confirmed safe because stub injection for `ToolType` / `MediaResolution` runs **after** filtering — filter strips unused schemas, then the script re-detects dangling `$ref` targets and injects stubs.
- **ts-client (N9)**: Sync script now warns (`[sync-gemini] Schema <Name> now ships upstream — remove stub injection`) if Google starts shipping `ToolType` or `MediaResolution` — prevents stubs silently shadowing real schemas.
- **e2e (N10)**: `api-gemini-embeddings.spec.mjs` and `chat-gemini.spec.mjs` wrap their bodies in `try/finally` so the model-delete always runs even on mid-test failure.
- **e2e (N11)**: Moved `createAllFormatModels` (and OAuth `Obtain OAuth app token`) into the `try` block in both `api-live-upstream.spec.mjs` tests so `finally → deleteAllModels` runs on partial creation failures.
- **e2e (N12)**: Replaced `page.waitForTimeout(1000)` calls in `ApiModelFormComponent` `fetchAndSelectModels` and `testConnection` retry loops with `locator.waitFor({ state: 'visible' })`; preserved existing `timeout: 20000` on `toBeEnabled` checks.
- **e2e (T7)**: Added `:streamGenerateContent` to Gemini `primaryEndpoints` + a new `streamingEndpoints` fixture field. New helper `fetchWithBearerSSE` in `api-model-helpers.mjs` parses both JSON-array streams (default `:streamGenerateContent` without `?alt=sse`) and SSE `data:` lines; returns all chunks as `data.chunks`. Gemini's `extractPrimaryResponse` concatenates text across chunks when `chunks` is present.

### Verification (2026-04-14)

- `cargo test -p services -p routes_app --lib` — green
- `make test.ui.unit` — 904 passed, 6 skipped, 0 failed
- `make test.napi.standalone` — full suite green after the streaming-fixture fix (initial run failed T7 streaming with empty response because the helper assumed SSE; fixed by handling both SSE and JSON-array formats)
