# Anthropic Messages API — Phase 1 Follow-ups (Rethink)

## Implementation Status

**All tasks A–J shipped in HEAD commit `c9e3568a`** (squashed from Phase 1A–1F + rethink + chat completions work).

| Task | Description                                                                   | Status |
| ---- | ----------------------------------------------------------------------------- | ------ |
| A    | Frontend typecheck fix (`getFetchDisabledReason` + `apiFormat`)               | ✅ Done |
| B    | Remove `anthropic-api-types` workspace dependency                             | ✅ Done |
| C    | Check in Anthropic OpenAPI spec (`resources/openapi-anthropic.json`)          | ✅ Done |
| D    | Local `AnthropicApiError` / `AnthropicErrorResponse` types                    | ✅ Done |
| E    | Strip `#[utoipa::path]` annotations, remove `API_TAG_ANTHROPIC`               | ✅ Done |
| F    | Opaque proxy: remove `validate_messages_request`, adopt `AnthropicApiError`   | ✅ Done |
| G    | Register Anthropic spec with `SwaggerUi` (`/api-docs/openapi-anthropic.json`) | ✅ Done |
| H    | Update handler tests (delete 2 obsolete, assert Anthropic envelope shape)     | ✅ Done |
| I    | TECHDEBT.md (5 deferred items)                                                | ✅ Done |
| J    | Verification gates (unit tests, live tests, manual swagger-ui check)          | ✅ Done |

Deferred items are tracked in `TECHDEBT.md` in this directory.

## Context

Phase 1 of the Anthropic Messages API (commits `ef590f80` → `933fb595`) shipped a working pass-through proxy at `/anthropic/v1/*` plus chat UI routing via pi-ai's `anthropic-messages` provider. During the pending-task review the user decided to **rethink the provider-proxy architecture**:

- Future AI API provider proxies (Anthropic today, Gemini/Vertex/etc. later) follow one pattern: **a checked-in, pre-filtered OpenAPI JSON spec served via swagger-ui** plus a **minimally-intrusive, opaque reverse proxy** in Rust. No `utoipa` annotations, no upstream type crate coupling, no body validation.
- For Anthropic specifically:
  - Check in the filtered spec from `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/anthropic-types/generated/anthropic/filtered-openapi.json` into this repo and wire it into the existing `SwaggerUi` registration at `crates/routes_app/src/routes.rs:538-540`.
  - Drop the workspace dependency on `anthropic-api-types` entirely (currently a path dep added in Phase 1A).
  - Strip `#[utoipa::path]` annotations from `routes_oai_anthropic.rs` — the swagger docs come from the checked-in JSON, not from handler metadata.
  - Delete `validate_messages_request` — no validation of `messages`, `max_tokens`, `stream`, etc. The only field the handler touches is `model`, purely to resolve which `ApiAlias` to forward to. Everything else is opaque bytes.
  - Define a **tiny local `AnthropicError` / `AnthropicErrorResponse` pair** in BodhiApp (public fields, `serde::Serialize`) so BodhiApp-local errors (missing alias, missing `model` field, etc.) surface to native Anthropic SDK clients in the `{"type":"error","error":{"type":"...","message":"..."}}` envelope. No dependency on the upstream crate's private-field struct.

Also in this batch:

- **Frontend typecheck regression** (`useApiModelForm.ts:289`): `fetchModels.getFetchDisabledReason({apiKey, baseUrl})` is missing the `apiFormat` field that `FetchModelsData` gained in Phase 1B. `npm run test:typecheck` fails. Single-line fix.
- **TECHDEBT.md** at `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md` for explicitly deferred items: 3rd-party providers (Bedrock/Vertex/OpenRouter native auth), full `ModelInfo` metadata on `/anthropic/v1/models` (currently synthetic `{id,type:"model"}` stubs), and Claude-optimized `contextWindow`/`maxTokens` defaults in the chat UI.
- **Completed** (no action): `@bodhiapp/anthropic-api-types` npm package already at v0.1.3 per the user.

The already-bumped `ts-client/package.json` (`0.1.28-dev`) is intentional and left alone.

## Architecture Decisions

### 1. Checked-in Anthropic OpenAPI spec

**Source**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/anthropic-types/generated/anthropic/filtered-openapi.json` (7401 lines, already filtered to `/v1/messages` and `/v1/models`, includes all referenced component schemas).

**Target**: `crates/routes_app/resources/openapi-anthropic.json` (new `resources/` directory inside the crate — mirrors the test-resource pattern used elsewhere).

**Path rewrite**: The upstream file uses Anthropic's native paths (`/v1/messages`, `/v1/models`). BodhiApp proxies them under `/anthropic/v1/*`. Rather than edit every path key on every request, we inject a `servers` array at the JSON root when loading:

```json
"servers": [{"url": "/anthropic"}]
```

Swagger UI prepends the server URL to all relative paths, so end users see the correct proxied URLs and "Try it out" works out of the box.

**Loading strategy**: `include_str!("../resources/openapi-anthropic.json")` at compile time. At server boot (inside `build_routes()` in `routes.rs`), parse the string once into `serde_json::Value`, inject/override `servers`, cache the result as a `String` (or `serde_json::Value`), and register it with `SwaggerUi` via `.url("/api-docs/openapi-anthropic.json", ...)`.

`utoipa_swagger_ui::SwaggerUi::url()` expects a `utoipa::openapi::OpenApi`. We have two options:
- **Option A** — parse the JSON into `utoipa::openapi::OpenApi` via `serde_json::from_str`. `utoipa::openapi::OpenApi` derives `Deserialize`, so this works.
- **Option B** — serve the JSON from a plain axum handler and register it with `SwaggerUi` via `.external_url(...)` (if supported).

Option A is cleaner and integrates with the existing pattern — we use it.

### 2. Opaque proxy — remove all body validation

`routes_oai_anthropic.rs` currently has `validate_messages_request` (checks `model`, `messages`, `max_tokens`, `stream`) and `validate_model_id` (URL path safety). In the new plan:

- **Delete `validate_messages_request`**. Handler only extracts `model` via `request.get("model").and_then(|v| v.as_str())`. If absent, return `AnthropicApiError::missing_model()` (400 / `invalid_request_error`). Everything else (including missing `messages`, `max_tokens`) is Anthropic's problem — forward verbatim and surface the upstream error to the client.
- **Keep `validate_model_id`**. This is path-parameter safety (not body validation) — it rejects non-ASCII, slash, and control characters that could cause URL-injection or path-traversal issues downstream. Keep the same allowlist (`a-zA-Z0-9._-`, max 128).

### 3. Remove `anthropic-api-types` dependency entirely

- Root `Cargo.toml` `[workspace.dependencies]`: delete the `anthropic-api-types = { path = ... }` entry added in Phase 1A.
- No crate currently pulls it (I confirmed in Phase 1: services/Cargo.toml and routes_app/Cargo.toml don't list it). So removal is a single deletion.
- The local `AnthropicError` / `AnthropicErrorResponse` types replace the upstream `ErrorResponse` type (which had private fields and required a JSON round-trip workaround in the previous plan draft — that workaround is no longer needed).

### 4. Local minimal Anthropic error types

**New file**: `crates/routes_app/src/shared/anthropic_error.rs`

```rust
use crate::shared::api_error::ApiError;
use axum::{body::Body, response::{IntoResponse, Response}};
use serde::Serialize;
use services::AppError;

/// Anthropic's native error-envelope wire format. Matches
/// https://docs.claude.com/en/api/errors exactly.
///
/// Local mirror of `anthropic_api_types::ErrorResponse` — we define it
/// in-repo to avoid depending on the upstream crate (whose fields are
/// private, blocking direct construction).
#[derive(Debug, Serialize)]
pub struct AnthropicErrorResponse {
  #[serde(rename = "type")]
  pub envelope_type: &'static str, // always "error"
  pub error: AnthropicErrorBody,
}

#[derive(Debug, Serialize)]
pub struct AnthropicErrorBody {
  #[serde(rename = "type")]
  pub error_type: &'static str, // "invalid_request_error", "api_error", etc.
  pub message: String,
}

/// IntoResponse wrapper for `/anthropic/v1/*` handlers.
#[derive(Debug)]
pub struct AnthropicApiError {
  pub status: u16,
  pub body: AnthropicErrorResponse,
}

impl AnthropicApiError {
  pub fn missing_model() -> Self { ... }
  pub fn not_found(message: impl Into<String>) -> Self { ... }
  pub fn invalid_request(message: impl Into<String>) -> Self { ... }
}

impl From<ApiError> for AnthropicApiError { ... }

impl<T: AppError + 'static> From<T> for AnthropicApiError {
  fn from(value: T) -> Self { Self::from(ApiError::from(value)) }
}

impl IntoResponse for AnthropicApiError { ... }
```

**BodhiApp → Anthropic error type mapping** (used by `From<ApiError>`):

| BodhiApp serialized `error_type` | Anthropic `error.type`  |
| -------------------------------- | ----------------------- |
| `invalid_request_error`          | `invalid_request_error` |
| `authentication_error`           | `authentication_error`  |
| `forbidden_error`                | `permission_error`      |
| `not_found_error`                | `not_found_error`       |
| `internal_server_error`          | `api_error`             |
| `service_unavailable`            | `overloaded_error`      |
| `unprocessable_entity_error`     | `invalid_request_error` |
| anything else                    | `api_error` (fallback)  |

HTTP status is preserved as-is from `ApiError.status` (which comes from `ErrorType::status()`).

Export via `crates/routes_app/src/shared/mod.rs`:
```rust
mod anthropic_error;
pub use anthropic_error::{AnthropicApiError, AnthropicErrorBody, AnthropicErrorResponse};
```

### 5. Strip `#[utoipa::path]` annotations

`routes_oai_anthropic.rs` currently has three `#[utoipa::path(...)]` blocks (lines ~140, ~193, ~246) tagging handlers with `API_TAG_ANTHROPIC`. None of these are registered in `BodhiOAIOpenAPIDoc` or `BodhiOpenAPIDoc` (verified: no `__path_anthropic_*` references in `openapi_oai.rs` or `openapi.rs`), so they're orphaned metadata. Remove them:

- Delete the 3 `#[utoipa::path(...)]` blocks
- Remove `use crate::API_TAG_ANTHROPIC;` from `routes_oai_anthropic.rs`
- Remove the `pub const API_TAG_ANTHROPIC: &str = "anthropic";` line from `crates/routes_app/src/shared/constants.rs`

### 6. Handler return type migration

`routes_oai_anthropic.rs` handlers currently return `Result<Response, OaiApiError>`. Change to `Result<Response, AnthropicApiError>`. The `?` operator lifts:

- `ApiError` (from `resolve_anthropic_alias`) → `AnthropicApiError` (direct `From<ApiError>`)
- `InferenceError` (from `forward_remote_with_params`) → `ApiError` (existing `.map_err(ApiError::from)`) → `AnthropicApiError`
- Local validation errors are constructed via `AnthropicApiError::missing_model()` etc., returning directly

The `anthropic_models_list_handler` also moves to `Result<Json<Value>, AnthropicApiError>` for consistency (even though it currently can only return OK).

### 7. Swagger UI registration

`routes.rs:538-540` currently:

```rust
.merge(
  SwaggerUi::new("/swagger-ui")
    .url("/api-docs/openapi.json", openapi)
    .url("/api-docs/openapi-oai.json", openapi_oai),
)
```

Add a third `.url(...)` for Anthropic. The doc is loaded from the embedded JSON string, parsed into `utoipa::openapi::OpenApi`, and server URL overridden:

```rust
// Near the top of build_routes(), alongside other openapi doc prep
const ANTHROPIC_OPENAPI_JSON: &str = include_str!("../resources/openapi-anthropic.json");

let openapi_anthropic: utoipa::openapi::OpenApi = {
  let mut doc: utoipa::openapi::OpenApi = serde_json::from_str(ANTHROPIC_OPENAPI_JSON)
    .expect("anthropic openapi spec must be valid");
  // Override `servers` so all relative paths resolve under /anthropic.
  doc.servers = Some(vec![
    utoipa::openapi::ServerBuilder::new().url("/anthropic").build(),
  ]);
  doc
};

// ... later:
.merge(
  SwaggerUi::new("/swagger-ui")
    .url("/api-docs/openapi.json", openapi)
    .url("/api-docs/openapi-oai.json", openapi_oai)
    .url("/api-docs/openapi-anthropic.json", openapi_anthropic),
)
```

If `include_str!` compile fails on spec invalidity, we get a helpful error at build time. If `serde_json::from_str` panics at startup, we get a clear boot-time error — acceptable since the spec is a checked-in asset.

### 8. Frontend typecheck fix

`crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts:289` — add `apiFormat: watchedValues.api_format as ApiFormat` to the inline object passed to `fetchModels.getFetchDisabledReason(...)`. Same shape already used at line 217 for `handleFetchModels`. `ApiFormat` is already imported into this file.

## Execution Plan

### Task A — Frontend typecheck fix

**File**: `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts`

Add `apiFormat` to the `getFetchDisabledReason({...})` call near line 289.

**Verify**: `cd crates/bodhi && npm run test:typecheck` → clean.

### Task B — Remove `anthropic-api-types` workspace dependency

**File**: `Cargo.toml` (workspace root)

Delete the `anthropic-api-types = { path = "../../anthropic-api-types/rust" }` line from `[workspace.dependencies]`.

**Verify**: `cargo check -p routes_app -p services -p server_app` compiles clean (the dep is currently unused by any crate's `[dependencies]` — confirmed).

### Task C — Check in the Anthropic OpenAPI spec

**New file**: `crates/routes_app/resources/openapi-anthropic.json`

Copy from `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/anthropic-types/generated/anthropic/filtered-openapi.json`. Do not edit. Add a header comment in `resources/README.md` (also new) noting the upstream source and that updates are via re-copy (not hand-edit).

**Verify**: `serde_json::from_str::<serde_json::Value>` parses it cleanly. Can quickly smoke-test via `cargo run --bin bodhi -- serve` + `curl http://localhost:1135/api-docs/openapi-anthropic.json` after Task G lands.

### Task D — Define local Anthropic error types

**New file**: `crates/routes_app/src/shared/anthropic_error.rs`

Full contents per section 4 above — `AnthropicErrorResponse`, `AnthropicErrorBody`, `AnthropicApiError`, conversion impls, `IntoResponse`.

**Key details**:
- Error type mapping is a private `fn map_error_type(bodhi_type: &str) -> &'static str` with the table in section 4 plus a default arm returning `"api_error"`.
- `AnthropicApiError::missing_model()` → status 400, `{error_type: "invalid_request_error", message: "Field 'model' is required and must be a string."}`.
- `AnthropicApiError::not_found(msg)` / `::invalid_request(msg)` are constructor helpers used in the handlers.
- `IntoResponse` returns `Response::builder().status(self.status).header("Content-Type", "application/json").body(Body::from(serde_json::to_string(&self.body).unwrap()))`.

**Export**: add `mod anthropic_error;` and `pub use anthropic_error::{AnthropicApiError, AnthropicErrorBody, AnthropicErrorResponse};` to `crates/routes_app/src/shared/mod.rs`.

**New test file**: `crates/routes_app/src/shared/test_anthropic_error.rs` covering the mapping table and envelope shape:

- `test_from_bad_request_maps_to_invalid_request_error` — 400 + `invalid_request_error`
- `test_from_not_found_maps_to_not_found_error` — 404 + `not_found_error`
- `test_from_forbidden_maps_to_permission_error`
- `test_from_internal_server_maps_to_api_error`
- `test_from_service_unavailable_maps_to_overloaded_error`
- `test_from_unknown_error_type_falls_back_to_api_error`
- `test_missing_model_constructor` — asserts status + error_type + message text
- `test_into_response_envelope_shape` — builds an error, calls `into_response`, deserializes body, asserts `body["type"]=="error"`, `body["error"]["type"]`, `body["error"]["message"]`

**Verify**: `cargo test -p routes_app --lib -- anthropic_error`.

### Task E — Strip utoipa annotations, remove API_TAG_ANTHROPIC

**Files**:
- `crates/routes_app/src/oai/routes_oai_anthropic.rs` — delete the 3 `#[utoipa::path(...)]` blocks and the `use crate::API_TAG_ANTHROPIC;` import.
- `crates/routes_app/src/shared/constants.rs` — delete the `pub const API_TAG_ANTHROPIC: &str = "anthropic";` line.
- Search (`grep -rn API_TAG_ANTHROPIC crates/`) to confirm no other references remain; if any, clean up.

**Verify**: `cargo check -p routes_app`.

### Task F — Opaque proxy: remove validation, adopt `AnthropicApiError`

**File**: `crates/routes_app/src/oai/routes_oai_anthropic.rs`

1. Delete the `validate_messages_request(&serde_json::Value)` function.
2. Keep `validate_model_id(&str)` (URL path-param safety).
3. Keep `extract_anthropic_headers(&HeaderMap)` — `anthropic-*` pass-through.
4. Keep `resolve_anthropic_alias` unchanged — returns `Result<(ApiAlias, Option<String>), ApiError>`.
5. Keep `list_user_anthropic_aliases` unchanged.
6. Update imports:
   - Remove `use super::error::OAIRouteError;` (no longer referenced after deleting `validate_messages_request`)
   - Remove `use crate::{... JsonRejectionError, OaiApiError};` → `use crate::{ApiError, AnthropicApiError, JsonRejectionError};`
7. Rewrite `anthropic_messages_create_handler`:
   ```rust
   pub async fn anthropic_messages_create_handler(
     auth_scope: AuthScope,
     headers: HeaderMap,
     WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
   ) -> Result<Response, AnthropicApiError> {
     let model = request
       .get("model")
       .and_then(|v| v.as_str())
       .ok_or_else(AnthropicApiError::missing_model)?
       .to_string();

     let (api_alias, api_key) = resolve_anthropic_alias(&auth_scope, &model).await?;
     let client_headers = extract_anthropic_headers(&headers);

     let response = auth_scope
       .inference()
       .forward_remote_with_params(
         LlmEndpoint::AnthropicMessages,
         request,
         &api_alias,
         api_key,
         None,
         client_headers,
       )
       .await
       .map_err(ApiError::from)?;

     Ok(response)
   }
   ```
8. Update `anthropic_models_list_handler` return type to `Result<Json<serde_json::Value>, AnthropicApiError>`.
9. Update `anthropic_models_get_handler` return type to `Result<Response, AnthropicApiError>`. Replace the `OAIRouteError::InvalidRequest` branch in `validate_model_id` with an `AnthropicApiError::invalid_request("Invalid model_id format.")` return (or keep `validate_model_id` returning a `&'static str` error code and convert at the caller).

   Simplest: change `validate_model_id` signature to `Result<(), AnthropicApiError>` and return `AnthropicApiError::invalid_request("Invalid model_id format.")` directly.

### Task G — Register Anthropic spec with SwaggerUi

**File**: `crates/routes_app/src/routes.rs`

1. Add `const ANTHROPIC_OPENAPI_JSON: &str = include_str!("../resources/openapi-anthropic.json");` near the top of the file (module level).
2. Inside `build_routes` (near line 526 where `openapi_oai` is prepared), parse the JSON into `utoipa::openapi::OpenApi` and inject the `servers: [{"url": "/anthropic"}]` override (per section 7 above).
3. Add `.url("/api-docs/openapi-anthropic.json", openapi_anthropic)` to the `SwaggerUi::new("/swagger-ui")` chain at line 538-540.

**Verify**: `cargo check -p routes_app`. Start the server and open `http://localhost:1135/swagger-ui`; the spec dropdown should list `openapi-anthropic.json`; selecting it shows the Anthropic endpoints; "Try it out" against `/v1/messages` should hit `http://localhost:1135/anthropic/v1/messages`.

### Task H — Update handler tests

**File**: `crates/routes_app/src/oai/test_oai_anthropic.rs`

1. **Delete** tests made obsolete by removing body validation:
   - `test_messages_create_missing_messages` — body no longer validated; upstream would return its own error, which we don't mock here.
   - `test_messages_create_missing_max_tokens` — same reason.
2. **Update** `test_messages_create_missing_model` (lines ~84-90) to assert the Anthropic error envelope:
   ```rust
   assert_eq!(StatusCode::BAD_REQUEST, response.status());
   let body = response.json::<serde_json::Value>().await?;
   assert_eq!("error", body["type"].as_str().unwrap());
   assert_eq!("invalid_request_error", body["error"]["type"].as_str().unwrap());
   assert!(body["error"]["message"].as_str().unwrap().contains("model"));
   ```
3. **Update** `test_messages_create_model_not_found` — assert `body["error"]["type"] == "not_found_error"` + envelope shape.
4. **Update** `test_messages_create_rejects_non_anthropic_alias` — assert `body["error"]["type"] == "invalid_request_error"` while keeping the `message.contains("anthropic")` check. The message now comes from `AnthropicApiError::invalid_request(format!(...))` instead of `OAIRouteError::InvalidRequest`.
5. **Update** `test_models_get_invalid_model_id_slash` — assert the Anthropic envelope shape (`body["error"]["type"] == "invalid_request_error"`).
6. The 5 happy-path tests stay unchanged (they only assert `StatusCode::OK` or mocked body contents).

**Verify**: `cargo test -p routes_app --lib -- test_oai_anthropic`.

### Task I — TECHDEBT.md

**New file**: `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md`

Sections:
1. **Header**: title, status ("Phase 1 shipped, opaque-proxy rethink applied"), links to Phase 1 plan at `ai-docs/claude-plans/202603/transient-puzzling-hoare.md` and consolidated recommendation at `ai-docs/claude-plans/202604/20260407-responses-api/12-anthropic-consolidated-recommendation.md`.

2. **Deferred: 3rd-party Anthropic-compatible providers**
   - AWS Bedrock (SigV4 signing, regional endpoints)
   - Google Vertex AI (GCP IAM tokens)
   - OpenRouter / Helicone (Bearer-auth passthrough — should already work via existing format-aware headers in `forward_request_with_method`, but untested)
   - Phase 1 supports `api.anthropic.com` direct only

3. **Deferred: `/anthropic/v1/models` full `ModelInfo` metadata**
   - Handler currently returns synthetic `{id: "...", type: "model"}` stubs.
   - Anthropic's `ModelInfo` includes `display_name`, `created_at` (RFC 3339), `max_input_tokens`, `max_tokens`, `capabilities` — none of which are cached by BodhiApp.
   - Root cause: `ApiAlias.models` + `models_cache` store `Vec<String>` IDs only.
   - Fix path: extend the schema (e.g., `models_meta: JsonVec<ModelMetadata>`), populate via upstream `GET /v1/models` during alias creation, emit full metadata on list.
   - OpenAI `/v1/models` remains OpenAI-format — this concerns only `/anthropic/v1/models`.

4. **Deferred: Claude-optimized `contextWindow`/`maxTokens` defaults in chat UI**
   - `crates/bodhi/src/stores/agentStore.ts::buildModel` hardcodes `contextWindow: 128000, maxTokens: 4096` (OpenAI-shaped).
   - Claude Sonnet 3.5 has 200k context / 8192 max output tokens.
   - Not breaking — only affects the local UI limit-calculation — but sub-optimal.
   - Depends on item 3 (once `ModelInfo` metadata is cached per-alias, read from there).

5. **Deferred: upstream `anthropic-api-types` crate coupling**
   - Phase 1 plan briefly pulled this crate as a workspace dep (both path and 0.1.4 crates.io). Rethink drops the dep entirely because the crate's structs have private fields, blocking direct construction.
   - If BodhiApp later wants to import typed request/response structs (e.g., for a conversion layer in Phase 2), this will need either an upstream fix (pub fields or constructor methods) or re-introducing a JSON round-trip wrapper.
   - Reference repo: `https://github.com/BodhiSearch/anthropic-types` (locally at `~/Documents/workspace/src/github.com/BodhiSearch/anthropic-types`).

6. **Deferred: OpenAPI spec refresh automation**
   - The Anthropic spec at `crates/routes_app/resources/openapi-anthropic.json` is a manual copy from the upstream filtered generator. No automation detects when the upstream changes. Add a `make` target or CI check when it becomes a problem.

### Task J — Verification gates

Run in order; fix any failure before proceeding.

```bash
# Frontend (Task A)
cd crates/bodhi && npm run test:typecheck
cd crates/bodhi && npm test -- --run 2>&1 | grep -E "Test Files|Tests "

# Rust compile (Tasks B–G)
cargo check -p services -p routes_app -p server_app 2>&1 | tail -10

# Unit tests (Tasks D, E, F, H)
cargo test --lib -p services -p routes_app -p server_app 2>&1 | grep -E "test result|FAILED"

# Live Anthropic tests (body envelope shape is not asserted in live tests — should stay green)
cargo test --test test_live_anthropic -p server_app 2>&1 | tail -10

# Manual: swagger-ui integration (Task G)
make app.run   # or cargo run --bin bodhi -- serve --port 1135
# Open http://localhost:1135/swagger-ui, verify the dropdown lists three specs
# including "openapi-anthropic.json", and that the Anthropic endpoints render.
```

**Expected test deltas vs last session baseline**:

| Suite                 | Baseline        | Expected        | Delta                                                                                                                                                    |
| --------------------- | --------------- | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `routes_app --lib`    | 683             | ~688            | +8 new anthropic_error unit tests, -2 deleted body-validation tests, -1 test that referenced OAIRouteError wire format (converted to envelope assertion) |
| `server_app --lib`    | 8               | 8               | unchanged                                                                                                                                                |
| `services --lib`      | 863             | 863             | unchanged                                                                                                                                                |
| `test_live_anthropic` | 5               | 5               | unchanged                                                                                                                                                |
| Frontend              | 881 / 6 skipped | 881 / 6 skipped | unchanged                                                                                                                                                |

No commits created by this plan — the user reviews implementation diff first.

## Critical Files Summary

| File                                                              | Change                                                                                                                                                                                              |
| ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Cargo.toml` (workspace root)                                     | **Remove** `anthropic-api-types` from `[workspace.dependencies]`                                                                                                                                    |
| `crates/bodhi/src/components/api-models/hooks/useApiModelForm.ts` | Add `apiFormat: watchedValues.api_format as ApiFormat` to `getFetchDisabledReason` call (~line 289)                                                                                                 |
| `crates/routes_app/resources/openapi-anthropic.json`              | **NEW** — copied verbatim from `~/Documents/workspace/src/github.com/BodhiSearch/anthropic-types/generated/anthropic/filtered-openapi.json`                                                         |
| `crates/routes_app/resources/README.md`                           | **NEW** — documents the source path and refresh procedure for the spec                                                                                                                              |
| `crates/routes_app/src/shared/anthropic_error.rs`                 | **NEW** — `AnthropicApiError`, `AnthropicErrorResponse`, `AnthropicErrorBody`, `From<ApiError>`, `IntoResponse`, helper constructors                                                                |
| `crates/routes_app/src/shared/test_anthropic_error.rs`            | **NEW** — 8 unit tests for mapping + envelope shape                                                                                                                                                 |
| `crates/routes_app/src/shared/mod.rs`                             | Declare + re-export new module                                                                                                                                                                      |
| `crates/routes_app/src/shared/constants.rs`                       | **Delete** `API_TAG_ANTHROPIC` line                                                                                                                                                                 |
| `crates/routes_app/src/oai/routes_oai_anthropic.rs`               | Delete `validate_messages_request`, strip `#[utoipa::path]` blocks (3), remove `API_TAG_ANTHROPIC` import, swap handler return types `OaiApiError` → `AnthropicApiError`, simplify model extraction |
| `crates/routes_app/src/oai/test_oai_anthropic.rs`                 | Delete 2 obsolete body-validation tests, update 4 tests to assert Anthropic envelope shape                                                                                                          |
| `crates/routes_app/src/routes.rs`                                 | Add `include_str!` constant, parse + override servers, register third `.url(...)` on `SwaggerUi`                                                                                                    |
| `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md`      | **NEW** — 6-section tech-debt register                                                                                                                                                              |

## Reuse of Existing Code

- `ApiError` (`crates/routes_app/src/shared/api_error.rs:12`) — source for `From<ApiError> for AnthropicApiError`
- `AppError` trait (`crates/errmeta/src/app_error.rs`) — blanket `From<T: AppError + 'static>` follows the same pattern as `ApiError` / `OaiApiError`
- `ErrorType::status()` (`crates/errmeta/src/error_type.rs:30`) — HTTP status reused via `ApiError.status` intermediate
- `resolve_anthropic_alias`, `extract_anthropic_headers`, `list_user_anthropic_aliases`, `validate_model_id` (`routes_oai_anthropic.rs`) — all kept; only the three handler signatures and internal validation change
- `forward_remote_with_params(..., client_headers)` (services, added Phase 1B) — unchanged
- `anthropic_auth_middleware` (`routes_app/src/middleware/anthropic_auth_middleware.rs`) — unchanged
- Phase 1D call-site pattern at `useApiModelForm.ts:217` for `handleFetchModels` — Task A copies the same `apiFormat` shape
- `SwaggerUi::new("/swagger-ui").url(...)` pattern in `routes.rs:538-540` — Task G adds a third `.url()` call alongside
- `utoipa::openapi::OpenApi` / `ServerBuilder` — existing dep, used for the deserialize + server override

## Non-Goals (Explicit De-Scopes)

- 3rd-party providers (Bedrock, Vertex, OpenRouter native auth) → TECHDEBT item 1
- Full `ModelInfo` metadata on `/anthropic/v1/models` → TECHDEBT item 2
- Chat UI Claude-optimized defaults → TECHDEBT item 3
- OpenAI → Anthropic chat completions request/response conversion → out of scope
- Anthropic SSE stream conversion to OpenAI deltas → out of scope (SSE pass-through works; format conversion is not needed)
- Automated spec refresh from upstream → TECHDEBT item 4
- `npm` package publish (`@bodhiapp/anthropic-api-types`) → already done by user at v0.1.3
- Live tests for SSE streaming and upstream error pass-through → TECHDEBT item 5

## Pending Items

All pending items are tracked in `TECHDEBT.md` (5 items):

1. **3rd-party providers** (Bedrock / Vertex) — SigV4 / GCP IAM signing not implemented.
2. **`/anthropic/v1/models` metadata** — stubs only (`{id, type: "model"}`); no `display_name`, `max_tokens`, etc.
3. **Chat UI `contextWindow`/`maxTokens` hardcoded** — still OpenAI-shaped; depends on item 2.
4. **OpenAPI spec refresh automation** — manual copy from upstream; no `make` target or CI drift check.
5. **Live test gaps** — SSE streaming and upstream error pass-through not covered in `test_live_anthropic.rs`.

## Next Phase Items

- **Anthropic on `/v1/chat/completions`** — Anthropic's less-advertised OpenAI-compatible endpoint. This was implemented as part of the same squashed commit (`20260410-anthropic-chat-completions.md`). No format conversion needed — opaque proxy forwards directly and format-aware auth injects the correct headers.
- **Chat UI routing** — Already implemented via pi-ai `anthropic-messages` provider routing through `/anthropic/v1/messages`. Covered by parameterized `api-models.spec.mjs` E2E tests.

## Verification (end-to-end walkthrough)

After implementing all tasks, in addition to the automated gates:

1. `make app.run` — start the server.
2. Open `http://localhost:1135/swagger-ui` — confirm the spec dropdown lists `openapi-anthropic.json` alongside `openapi.json` and `openapi-oai.json`. Select it; confirm `POST /v1/messages` and `GET /v1/models` render with their descriptions and "Try it out" buttons.
3. Create an Anthropic API alias via the UI (Phase 1D flow).
4. `curl -H "Authorization: Bearer bodhiapp_<token>" -H "Content-Type: application/json" -d '{}' http://localhost:1135/anthropic/v1/messages` → expect:
   ```json
   {"type":"error","error":{"type":"invalid_request_error","message":"Field 'model' is required and must be a string."}}
   ```
5. `curl -H "Authorization: Bearer bodhiapp_<token>" -H "Content-Type: application/json" -d '{"model":"nonexistent","max_tokens":1,"messages":[]}' http://localhost:1135/anthropic/v1/messages` → expect the Anthropic `not_found_error` envelope.
6. Send a valid request (real Anthropic key configured on the alias) — verify the upstream body passes through unchanged (no BodhiApp rewrapping).
7. Use `/ui/chat` with an Anthropic alias to confirm the chat flow still works (unchanged — Task F is the only handler path touched, and successful responses pass through the body verbatim).
