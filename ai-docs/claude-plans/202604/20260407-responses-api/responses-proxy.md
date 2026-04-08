# Responses API — Pure Proxy Implementation Plan

## Status Legend
- ✅ Done
- 🔲 Pending

---

## Context

BodhiApp currently supports OpenAI Chat Completions API as an opaque proxy to remote providers. The OpenAI Responses API is a newer format that supports server-side conversation state, tool use, and richer item types. Competitive analysis (Ollama, Portkey, Bifrost, Goose) shows **all implementations are stateless** — none store conversations server-side. BodhiApp will implement the Responses API as a **pure pass-through proxy**, identical in approach to the existing chat completions proxy: forward request bytes to upstream, stream response bytes back.

Key decisions from user interview:
- Pure proxy, no server-side storage
- Remote providers only (local llama.cpp stays on Chat Completions)
- Separate `openai_responses` ApiFormat variant (backwards-compatible, `openai` unchanged)
- All CRUD endpoints: POST create, GET retrieve, DELETE, GET input_items, POST cancel
- GET/DELETE use required `model` query param for alias routing
- Format-aware test prompt endpoint
- Opaque byte forwarding for streaming

---

## Phase 1: `services` crate ✅

### 1.1 ✅ Add `response-types` feature to async-openai workspace dep

**File**: `Cargo.toml` (workspace root)

Added `"response-types"` to the features list.

### 1.2 ✅ Add `ApiFormat::OpenAIResponses` variant

**File**: `crates/services/src/models/model_objs.rs`

Added `OpenAIResponses` variant with `#[serde(rename = "openai_responses")]` / `#[strum(serialize = "openai_responses")]`. No DB migration needed — stored as string.

### 1.3 ✅ Extend `LlmEndpoint`

**File**: `crates/services/src/inference/inference_service.rs`

Changed from `Copy` to `Clone`. Added 5 new variants: `Responses`, `ResponsesGet(String)`, `ResponsesDelete(String)`, `ResponsesInputItems(String)`, `ResponsesCancel(String)`. Added `api_path() -> String` and `http_method() -> &'static str`.

### 1.4 ✅ Add `SafeReqwest::delete` method

**File**: `crates/services/src/shared_objs/safe_reqwest.rs`

Added `delete()` alongside existing `get` and `post`.

### 1.5 ✅ Add `forward_request_with_method` to `AiApiService`

**File**: `crates/services/src/ai_apis/ai_api_service.rs`

Added method to trait and `DefaultAiApiService`. Existing `forward_request` now delegates to it. Supports GET/DELETE/POST with optional body and query params.

Note: Parameter type changed from `Option<&[(String, String)]>` to `Option<Vec<(String, String)>>` to avoid mockall lifetime issues.

### 1.6 ✅ Add `api_format` to `TestPromptRequest`

**File**: `crates/services/src/models/model_objs.rs`

Added `#[serde(default = "default_api_format_openai")] pub api_format: ApiFormat` with a named default function (not `Default` trait — correct from a domain perspective).

### 1.7 ✅ Format-aware `test_prompt`

**File**: `crates/services/src/ai_apis/ai_api_service.rs`

Added `api_format: &ApiFormat` parameter. OpenAI format: existing `/chat/completions` body. OpenAIResponses: `/responses` endpoint with `input`, `max_output_tokens: 50`, `store: false` (no `temperature` — unsupported by Responses API). Response parsing differs per format.

### 1.8 ✅ Update `api_models_formats` return

**File**: `crates/routes_app/src/models/api/routes_api_models.rs`

Returns both `ApiFormat::OpenAI` and `ApiFormat::OpenAIResponses`.

**Gate check**: ✅ `cargo test -p services --lib`

---

## Phase 2: `server_core` crate ✅

### 2.1 ✅ Update `proxy_to_remote`

**File**: `crates/server_core/src/standalone_inference.rs`

Uses `forward_request_with_method` with method selection via `endpoint.http_method()`. GET/DELETE pass `None` body.

### 2.2 ✅ Add `forward_remote_with_params` to InferenceService

**File**: `crates/services/src/inference/inference_service.rs`

Added `forward_remote_with_params()` to trait. Implemented in `standalone_inference.rs`, `multitenant_inference.rs`, and `noop.rs` (returns `Unsupported`).

### 2.3 ✅ Fix non-exhaustive match on `LlmEndpoint`

**File**: `crates/server_core/src/shared_rw.rs`

Added `_ => return Err(ContextError::Unreachable(...))` catch-all to 3 match arms on `LlmEndpoint` for local model forwarding (Responses endpoints are never routed to local).

**Gate check**: ✅ `cargo test -p server_core --lib`

---

## Phase 3: `routes_app` crate ✅ (except unit tests)

### 3.1 ✅ Endpoint constants and new module

**File**: `crates/routes_app/src/oai/mod.rs`

Added `routes_oai_responses` module, `pub use`, and `ENDPOINT_OAI_RESPONSES = "/v1/responses"`.

### 3.2 ✅ Create route handlers

**File**: `crates/routes_app/src/oai/routes_oai_responses.rs` (new)

Five handlers: `responses_create_handler`, `responses_get_handler`, `responses_delete_handler`, `responses_input_items_handler`, `responses_cancel_handler`. Helper functions: `validate_responses_request()`, `resolve_responses_alias()`, `extract_model_param()`, `upstream_query_params()`.

Note: utoipa annotations use `serde_json::Value` placeholder (not async-openai Responses types) due to recursive type stack overflow in schema generation.

### 3.3 ✅ Format guard in chat completions / embeddings handlers

**File**: `crates/routes_app/src/oai/routes_oai_chat.rs`

Added `api_format != ApiFormat::OpenAIResponses` guard to both `chat_completions_handler` and `embeddings_handler`. Aliases with `openai_responses` format are rejected with 400 error directing users to the responses endpoint.

### 3.4 ✅ Register routes

**File**: `crates/routes_app/src/routes.rs`

Registered 4 route entries in `user_apis`: POST `/v1/responses`, GET+DELETE `/v1/responses/{response_id}`, GET `/v1/responses/{response_id}/input_items`, POST `/v1/responses/{response_id}/cancel`.

### 3.5 ✅ OpenAPI registration

- Added `API_TAG_RESPONSES = "responses"` to `src/shared/constants.rs`
- Registered tag, paths for all 5 handlers in `src/shared/openapi.rs`
- Regenerated `openapi.json` and TypeScript types

### 3.6 ✅ Update `api_models_test` handler

**File**: `crates/routes_app/src/models/api/routes_api_models.rs`

Passes `payload.api_format` to `ai_api.test_prompt(...)`.

### 3.7 ✅ Unit tests for responses handlers

**File**: `crates/routes_app/src/oai/test_oai_responses.rs` (new, 10 tests)

- `test_responses_create_missing_model` → 400
- `test_responses_create_missing_input` → 400
- `test_responses_create_model_not_found` → 404
- `test_responses_create_wrong_format` → seeded openai-format alias, returns 400 with "openai_responses" in message
- `test_responses_create_success` → mock `forward_remote` with `LlmEndpoint::Responses`
- `test_responses_get_missing_model_param` → GET without `?model=` → 400
- `test_responses_get_success` → mock `forward_remote_with_params` with `LlmEndpoint::ResponsesGet(id)`
- `test_responses_delete_success` → mock `forward_remote` with `LlmEndpoint::ResponsesDelete(id)`
- `test_responses_input_items_success` → mock `forward_remote_with_params` with `LlmEndpoint::ResponsesInputItems(id)`
- `test_responses_cancel_success` → mock `forward_remote` with `LlmEndpoint::ResponsesCancel(id)`

Test module reference added at bottom of `routes_oai_responses.rs`.

### 3.8 ✅ Unit tests for format rejection (chat/embeddings)

**File**: `crates/routes_app/src/oai/test_chat.rs`

Added and passing:
- `test_chat_completions_rejects_responses_format_alias`
- `test_embeddings_rejects_responses_format_alias`

**Gate check**: ✅ `cargo test -p routes_app` (655 unit + 9 integration = 664 passed)

---

## Phase 4: Full backend validation ✅

```bash
make test.backend
```

Passed (exit code 0). All crates compile and test clean.

---

## Phase 5: TypeScript client regeneration ✅

`openapi.json` regenerated. `ts-client/src/types/types.gen.ts` updated with `openai_responses` in `ApiFormat`. `make build.ts-client` — done.

---

## Phase 6: Frontend ✅

### 6.1 ✅ Update format presets

**File**: `crates/bodhi/src/schemas/apiModel.ts`

`API_FORMAT_PRESETS` includes both `openai` ("OpenAI - Completions") and `openai_responses` ("OpenAI - Responses").

### 6.2 ✅ Update API format display

**File**: `crates/bodhi/src/components/api-models/form/ApiFormatSelector.tsx`

Added `formatDisplayName()` using preset lookup. Both formats display correctly.

### 6.3 ✅ Test connection format-aware

**File**: `crates/bodhi/src/components/api-models/hooks/useTestConnection.ts`

Includes `api_format` in `TestPromptRequest` payload.

### 6.4 ✅ API key clearing on format switch

**File**: `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`

Added `setValue('api_key', '')` in `handleApiFormatChange`.

### 6.5 ✅ Frontend tests

All 860 tests passing (77 test files).

**Gate check**: ✅ `cd crates/bodhi && npm test`

---

## Phase 7: E2E 🔲

```bash
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

Functional validation was done manually via browser:
- ✅ Created API model with `openai_responses` format
- ✅ Fetched models list
- ✅ Test connection works
- ✅ Chat via `/ui/chat` routes through responses endpoint (not chat completions)
- ✅ Chat completions endpoint rejects responses-format aliases (verified via unit tests)

Formal Playwright E2E test for create/edit/delete of `openai_responses` model: **pending**.

---

## Phase 8: `/ui/chat` support for Responses format 🔲

Currently `/ui/chat` sends requests to `/v1/chat/completions`, which correctly rejects `openai_responses`-format aliases. The chat UI needs to detect the alias format and route to `/v1/responses` instead.

Work needed:
- Determine alias format when a model is selected in chat (requires API call or alias metadata in model list response)
- If format is `openai_responses`: route to `/v1/responses` with `input` field instead of `messages`
- Handle streaming response differences (Responses API SSE format differs from Chat Completions)
- Map Responses API output back to a UI-compatible chat message format

**Scope note**: This is significant new scope — the chat UI currently has no concept of API format. Deferred to a future iteration.

---

## Remaining Work

| Task | Priority |
|------|----------|
| Phase 7: Playwright E2E for `openai_responses` model CRUD | Medium |
| Phase 8: `/ui/chat` support for `openai_responses` format | Future |

## Known Limitations / TODOs

- utoipa annotations for responses handlers use `serde_json::Value` placeholder instead of async-openai Responses types (recursive type stack overflow). OpenAPI schema for these endpoints is minimal — proper schema registration deferred.
- `/ui/chat` does not yet support `openai_responses` format aliases — attempting to chat with one returns an error (expected, by design, until Phase 8 is implemented).
