# routes_app Error Type Refactor

## Context

`crates/routes_app/` currently mixes two error systems that overlap and confuse the boundary between BodhiApp's canonical error envelope and the OpenAI-compatible wire format consumed by external SDKs:

- **async-openai** types (`ApiError`, `WrappedError`, re-aliased as `OaiErrorBody`/`OaiWrappedError`) — used for OpenAI-compatible error responses.
- **Bodhi** types — `BodhiErrorResponse`/`BodhiError` (in `shared/error_oai.rs`) for the canonical app envelope, plus a duplicated intermediate `ApiError` struct + an OAI-wire wrapper `OaiApiError` (both in `shared/api_error.rs`). `MiddlewareError` (in `middleware/error.rs`) duplicates the same fields a third time.

Problems this causes today:
1. Two structs (`ApiError`, `BodhiErrorResponse`) carry the same fields with different purposes.
2. `MiddlewareError` re-implements the same JSON shape inline.
3. `async-openai` imports leak from `src/oai/` into `shared/api_error.rs` and `shared/openapi_oai.rs`.
4. `OaiApiError` (only used by OAI handlers) lives in `shared/`, not where it belongs.

**Outcome:** A single `BodhiErrorResponse` is used everywhere outside `src/oai/` (handlers, middleware, extractors). `OaiApiError` and all `async-openai` imports live exclusively under `src/oai/`. `BodhiErrorResponse` and `BodhiError` are prefixed `Bodhi` for clarity. No backwards-compat shims; this is a clean cut.

## Design

### `crates/routes_app/src/shared/api_error.rs` (consolidated, replaces `error_oai.rs`)

```rust
pub struct BodhiError {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, String>>,
}  // ToSchema

pub struct BodhiErrorResponse {
    pub error: BodhiError,
    #[serde(skip)] pub status: u16,
}  // ToSchema, Display

impl<T: AppError + 'static> From<T> for BodhiErrorResponse { ... }  // direct, no ApiError hop
impl IntoResponse for BodhiErrorResponse { ... }
```

### `crates/routes_app/src/oai/api_error.rs` (new)

```rust
use async_openai::error::{ApiError, WrappedError};  // no aliases inside oai/

pub struct OaiApiError {
    pub message: String,
    pub error_type: String,
    pub status: u16,
    pub code: String,
    pub param: Option<String>,  // joined "k=v, k2=v2" for OAI wire format
}

impl<T: AppError + 'static> From<T> for OaiApiError { ... }
impl From<OaiApiError> for WrappedError { ... }
impl IntoResponse for OaiApiError { ... }
```

`OaiApiError` becomes a flat struct (no `ApiError` wrapper hop). `param`-joining stays in OAI-only code. `BodhiErrorResponse`'s `param` (HashMap) and `OaiApiError`'s `param` (joined String) are kept entirely separate — no cross-contamination.

## Phased Plan

Phases land independently; run `cargo check -p routes_app` after each. Run `cargo test -p routes_app --lib` after Phases 3, 4, 6, 7. `make test.backend` at the end.

### Phase 1 — Add direct `From<T: AppError>` and `IntoResponse` to `BodhiErrorResponse`
Files:
- `src/shared/api_error.rs` — add the new impls for `BodhiErrorResponse` (keep existing `ApiError`/`OaiApiError` in place; both compile in parallel)
- Move `BodhiError`/`BodhiErrorResponse` from `shared/error_oai.rs` into `shared/api_error.rs`
- Delete `src/shared/error_oai.rs`; update `shared/mod.rs`

### Phase 2 — Move `OaiApiError` → `src/oai/api_error.rs`
Files:
- New: `src/oai/api_error.rs` — flat `OaiApiError` with direct `From<T: AppError>` and `From<OaiApiError> for WrappedError` (use `async_openai::error::{ApiError, WrappedError}` without aliases)
- `src/oai/mod.rs` — `mod api_error; pub use api_error::*;`
- `src/shared/api_error.rs` — delete the `OaiApiError` struct + its impls; remove `async_openai` imports
- `src/shared/mod.rs` — drop the `OaiApiError` re-export

### Phase 3 — Migrate non-OAI handlers: `ApiError` → `BodhiErrorResponse`
Uniform mechanical change in each file (28 files): swap import + handler return type.

Files (verified in exploration):
`tokens/routes_tokens.rs`, `settings/routes_settings.rs`, `apps/routes_apps.rs`, `models/routes_models.rs`, `models/routes_models_metadata.rs`, `models/api/routes_api_models.rs`, `models/files/routes_files.rs`, `models/files/routes_files_pull.rs` (also explicit `let api_error: ApiError = e.into();` at ~line 218), `gemini/routes_gemini.rs`, `gemini/gemini_api_schemas.rs`, `anthropic/routes_anthropic.rs`, `anthropic/anthropic_api_schemas.rs`, `users/routes_users.rs`, `users/routes_users_info.rs`, `users/routes_users_access_request.rs`, `auth/routes_auth.rs`, `mcps/routes_mcps.rs`, `mcps/routes_mcps_auth.rs`, `mcps/routes_mcps_oauth.rs`, `mcps/routes_mcps_servers.rs`, `mcps/test_mcps.rs`, `tenants/routes_tenants.rs`, `tenants/routes_dashboard_auth.rs`, `setup/routes_setup.rs`, `routes_dev.rs`, `shared/auth_scope_extractor.rs` (`type Rejection = BodhiErrorResponse`).

Special cases:
- **`oai/routes_oai_responses.rs`**: change `resolve_responses_alias` helper to return `Result<_, OaiApiError>`; `map_err(ApiError::from)?` calls (lines 158, 203, 248, 293, 338) → `map_err(OaiApiError::from)?`.
- **`gemini/gemini_api_schemas.rs`**: delete `From<ApiError> for GeminiApiError` (line ~73). Rewrite the blanket `From<T: AppError>` (line ~96) to read `value.error_type()`, `value.to_string()`, etc. directly — no `ApiError` hop.

### Phase 4 — Replace `MiddlewareError` with `BodhiErrorResponse`
`MiddlewareError::into_response` emits exactly the same JSON shape as `BodhiErrorResponse`, so swap is wire-compatible.

Files:
- Delete `src/middleware/error.rs`; update `src/middleware/mod.rs`
- `src/middleware/auth/auth_middleware.rs` — `auth_middleware`, `optional_auth_middleware`
- `src/middleware/apis/api_middleware.rs` — `api_auth_middleware`
- `src/middleware/access_requests/access_request_middleware.rs` — `access_request_auth_middleware`

### Phase 5 — Move `shared/openapi_oai.rs` → `src/oai/openapi.rs`
Confines the last `async-openai` import out of `shared/`.

Files:
- New: `src/oai/openapi.rs` — `BodhiOAIOpenAPIDoc`; replace aliased imports with `use async_openai::error::{ApiError, WrappedError};` and register schemas as `WrappedError, ApiError` (no `Oai` aliases inside `oai/`)
- `src/oai/mod.rs` — `pub mod openapi; pub use openapi::*;`
- Delete `src/shared/openapi_oai.rs`; update `src/shared/mod.rs`

`__path_*` symbols still resolve (handlers stay in `src/oai/`). Public re-export path `routes_app::BodhiOAIOpenAPIDoc` is preserved via `lib.rs`'s `pub use oai::*;`.

### Phase 6 — Delete the now-dead `ApiError` struct from `shared/api_error.rs`
Verify zero remaining call sites (`grep -rn "ApiError" crates/routes_app/src --include="*.rs"` should only show `OaiApiError` in `oai/` and `async_openai::error::ApiError`). Remove the `ApiError` struct, its `Display`, `From<T: AppError>`, `From<ApiError> for BodhiErrorResponse`, and `IntoResponse`.

### Phase 7 — Test migration
- `src/shared/test_api_error.rs` → rename to test `BodhiErrorResponse` directly (handler returns `Result<_, BodhiErrorResponse>`); JSON assertions unchanged
- New `src/oai/test_oai_api_error.rs` — exercise `OaiApiError::into_response` and verify `WrappedError` JSON shape with joined `param` string

### Phase 8 — Doc updates
- `crates/CLAUDE.md` — Error Layer Separation: replace `ApiError`/`OpenAIApiError`/`ErrorBody` with `BodhiErrorResponse`/`OaiApiError`/`BodhiError`; drop `MiddlewareError`
- `crates/routes_app/CLAUDE.md` — Purpose, Error Handling Chain: relocate `OaiApiError` to `oai/`; canonical handler error is `BodhiErrorResponse`
- `crates/routes_app/PACKAGE.md` — update Error Enum Reference and file index
- `crates/routes_app/src/middleware/CLAUDE.md` — drop the `MiddlewareError` section; note middleware now returns `BodhiErrorResponse`

## Critical Files

- `crates/routes_app/src/shared/api_error.rs` (consolidated)
- `crates/routes_app/src/shared/error_oai.rs` (deleted)
- `crates/routes_app/src/shared/openapi_oai.rs` (deleted; moves to `oai/openapi.rs`)
- `crates/routes_app/src/middleware/error.rs` (deleted)
- `crates/routes_app/src/oai/api_error.rs` (new)
- `crates/routes_app/src/oai/openapi.rs` (new)
- `crates/routes_app/src/oai/routes_oai_responses.rs` (mixed-error cleanup)
- `crates/routes_app/src/gemini/gemini_api_schemas.rs` (drop `ApiError` intermediate)

## Risks

1. **`gemini_api_schemas.rs`** — `From<ApiError>` reads raw struct fields (`value.error_type`, `value.name`); rewrite to use `AppError` trait methods directly.
2. **`routes_oai_responses.rs`** — currently imports both `ApiError` and `OaiApiError`; helper return types must convert cleanly.
3. **`AuthScope` Rejection type** — changes from `ApiError` to `BodhiErrorResponse`; both implement `IntoResponse`, drop-in.
4. **`JsonRejectionError`** (`shared/error_wrappers.rs`) — independent of this chain, leave as-is. Optional follow-up: delegate to `BodhiErrorResponse::into_response`.
5. **utoipa `__path_*` symbols** — generated in handler module, unaffected by moving `BodhiOAIOpenAPIDoc`.

## Verification

After each phase:
```bash
cargo check -p routes_app 2>&1 | tail -5
```

After Phases 3, 4, 6, 7:
```bash
cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"
cargo test -p routes_app -- openapi 2>&1 | grep -E "test result|FAILED"  # after Phase 5
```

End-to-end (per BodhiApp layered methodology):
```bash
make test.backend           # full Rust validation
make build.ts-client        # confirm no OpenAPI shape regressions
cd crates/lib_bodhiserver && npm run test:playwright   # E2E sanity (error responses)
```

JSON wire-format compatibility check (manual): hit a 4xx endpoint via `make app.run` and confirm both `/bodhi/v1/...` returns `{"error":{"message":...,"type":...,"code":...,"param":{...}}}` and `/v1/chat/completions` returns OpenAI-style `{"error":{"message":...,"type":...,"code":...,"param":"k=v"}}`.
