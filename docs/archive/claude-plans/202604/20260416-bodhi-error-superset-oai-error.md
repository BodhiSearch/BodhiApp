# BodhiError: `param` → `params` + additional `param` (JSON-string superset)

**Status:** Implemented 2026-04-16

## Context

Commit `711a219f` decoupled `BodhiErrorResponse { error: BodhiError }` (the Bodhi-shaped management error envelope) from OpenAI's `ErrorResponse { error: Error }` (now `OaiApiError` under `routes_app/src/oai/`). The two types had divergent `param` shapes:

| Field  | BodhiError (before)                       | OaiApiError (OpenAI wire)      |
|--------|-------------------------------------------|--------------------------------|
| param  | `Option<HashMap<String, String>>` (object)| `Option<String>`               |

Fully untangling every downstream call site that straddles both shapes is not feasible in one pass. Instead, `BodhiError` becomes a **wire-level superset** of OpenAI's `Error`: the structured map was renamed to `params`, and a new `param: Option<String>` field emits the JSON-encoded form of `params`. Consumers that understand only the OpenAI shape can read `param` as a string (and optionally `JSON.parse` it); consumers that want structured key/value pairs read `params` directly. `/v1/chat/completions` continues returning `OaiApiError` unchanged — the two paths now co-exist without additional code surgery.

**Encoding decisions (settled with user):**
- `param` is `serde_json::to_string(&params)` → object JSON string (e.g. `"{\"var_0\":\"even\"}"`)
- `OaiApiError` was left alone — keeps its existing `"key=value, key=value"` join. The two `param` fields deliberately use different formats (by audience).

The `param` → `params` rename on `BodhiError` was performed by the user via IDE before this plan executed. The work below starts after that rename.

## What changed

### 1. `crates/routes_app/src/shared/api_error.rs`

- Dropped `derive_new::new` from `BodhiError` (kept on `BodhiErrorResponse`). Replaced with a hand-written `BodhiError::new(message, r#type, code, params)` that derives `param` from `params` via `serde_json::to_string`.
- Added new public field `param: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. (Initial attempt with `skip_deserializing` dropped the field from the utoipa schema — removing that attribute kept the field in the generated OpenAPI/TS output while preserving the invariant that constructors populate `param` from `params`.)
- Updated `#[schema(example = ...)]` on both `BodhiError` and `BodhiErrorResponse` to show both `params` (object) and `param` (JSON string).
- Updated the blanket `impl<T: AppError> From<T> for BodhiErrorResponse` to call `BodhiError::new(...)` instead of a struct literal.

### 2. `crates/routes_app/src/shared/error_wrappers.rs`

`JsonRejectionError::into_response` used to hand-build a `serde_json::Value`. Rewrote it to construct a `BodhiError` via `BodhiError::new` and serialize it — removing the drift risk between hand-built JSON and the struct's serialization. Also updated the inline test fixture to assert both `params` (map) and `param` (JSON string).

### 3. `crates/routes_app/src/shared/validated_json.rs`

Replaced the direct `BodhiError { ... }` struct literal with `BodhiError::new(...)`.

### 4. Test fixtures using struct literals

- `crates/routes_app/src/anthropic/test_anthropic_api_schemas.rs` — both `BodhiError { ... }` literals (`test_5xx_message_is_generic_not_internal_detail`, `test_4xx_message_is_preserved`) converted to `BodhiError::new(...)`.
- `crates/routes_app/src/gemini/test_gemini_api_schemas.rs` — same treatment for the two matching fixtures.

### 5. Test JSON assertions

Every fixture that asserted on error wire bodies updated to the new dual shape:

- `crates/routes_app/src/shared/test_api_error.rs` — the two `#[case]` fixtures now assert `"params": {...}` **and** `"param": "<json-string>"`.
- `crates/routes_app/src/shared/test_validated_json.rs` — `test_invalid_field_returns_validation_error` asserts `error["params"]["name"].is_string()` and additionally that `error["param"]` is a JSON-encoded string containing `"name"`.
- `crates/routes_app/src/middleware/auth/test_auth_middleware.rs` — two fixtures (reqwest token refresh failure, invalid-token error) updated to the dual shape.

OAI tests (`crates/routes_app/src/oai/test_api_error.rs`) were intentionally **not** changed — they still assert the OpenAI-wire `"param": "key=value"` string, which is correct for that path.

### 6. Regenerated artifacts

- `openapi.json` — regenerated via `cargo run --package xtask openapi`. `BodhiError` schema now lists both `params` (object) and `param` (string) under `properties`, and the example shows both.
- `ts-client/src/types/types.gen.ts` — regenerated `BodhiError` now has `params?: { [key: string]: string } | null` and `param?: string | null`.
- `ts-client/src/openapi-typescript/openapi-schema.ts` — regenerated.

### 7. Documentation

- `ai-docs/guides/bodhi-app/error-handling.md` — documented the superset relationship; updated the `BodhiAPIError` TypeScript example to expose both `params` and `param`; updated the "bad request field" example to prefer `error.params?.field`.
- `ai-docs/guides/bodhi-app/api-reference.md` — standard error response block now shows both fields with a note that OpenAI-compatible `/v1/*` routes use `OaiApiError` instead.
- `crates/routes_app/CLAUDE.md` — Error Handling Chain section updated to describe the superset envelope `{message, type, code, params?, param?}` and the intentional divergence from `OaiApiError`.
- `crates/routes_app/src/middleware/CLAUDE.md` — same wire-envelope update.

## Out of scope (intentionally unchanged)

- `OaiApiError` encoding stays `"key=value, key=value"`. `/v1/chat/completions`, `/v1/embeddings`, `/v1/models`, `/v1/responses/*` untouched.
- `AnthropicApiError` / `GeminiApiError` `From<BodhiErrorResponse>` impls untouched — they never read `params`/`param`.
- No backwards-compat shim for the old `param: HashMap` field name (per `CLAUDE.md`: BodhiApp prioritizes architectural improvement over BC).

## Verification (all green)

- `cargo test -p routes_app --lib` → 780 passed / 0 failed / 2 ignored.
- `cargo test --workspace` (SQLite + Postgres via docker) → all crates pass, no regressions.
- `cargo run --package xtask openapi` → regenerated cleanly; diff is exactly the expected two-field change.
- `cd ts-client && npm run generate && npm run build && npm test` → generation + build + tests pass.
- `cd crates/bodhi && npm test` → 904 passed / 6 skipped.
- Frontend lint has preexisting (unrelated) errors; no new violations introduced.
- `make ci.ts-client-check` reports uncommitted changes in `ts-client/src` — expected (the regenerated types are part of this commit).

E2E (`make test.e2e`) and `cargo test -p bodhi --features native` were not re-run in this session — no code paths they exercise were touched beyond the superset-field addition, which is fully covered by the routes_app and frontend test suites. Run them before release if desired.

## Critical files (implementation reference)

- `crates/routes_app/src/shared/api_error.rs` — struct, ctor, blanket From, schema examples
- `crates/routes_app/src/shared/validated_json.rs` — constructor call site
- `crates/routes_app/src/shared/error_wrappers.rs` — JsonRejectionError serialization
- `crates/routes_app/src/shared/test_api_error.rs`, `test_validated_json.rs` — core assertions
- `crates/routes_app/src/middleware/auth/test_auth_middleware.rs` — middleware fixtures
- `crates/routes_app/src/anthropic/test_anthropic_api_schemas.rs`, `crates/routes_app/src/gemini/test_gemini_api_schemas.rs` — fixture constructors
- `openapi.json`, `ts-client/src/types/types.gen.ts`, `ts-client/src/openapi-typescript/openapi-schema.ts` — regenerated artifacts
- `ai-docs/guides/bodhi-app/error-handling.md`, `ai-docs/guides/bodhi-app/api-reference.md` — user-facing docs
- `crates/routes_app/CLAUDE.md`, `crates/routes_app/src/middleware/CLAUDE.md` — developer docs
