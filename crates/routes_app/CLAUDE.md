# routes_app -- CLAUDE.md

**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, file index, error enum reference, Route Group Architecture table, Domain Module Structure, Handler/JSON Extraction Conventions, Proxy sections, Key Workflow Gotchas
- `src/middleware/CLAUDE.md` -- Middleware-specific documentation (auth, authorization, token service)
- `src/middleware/PACKAGE.md` -- Middleware module index and error enum reference
- `TESTING.md` -- Test patterns, mocking strategy, router construction
- `TECHDEBT.md` -- Known issues and planned refactors

## Purpose

API orchestration layer: HTTP endpoint handlers for all BodhiApp application routes. Defines the canonical `BodhiErrorResponse` + `BodhiError` envelope in `shared/api_error.rs`; the OpenAI wire-format `OaiApiError` lives in `src/oai/api_error.rs` (the only module that imports `async-openai` error types). Includes authentication/authorization middleware (merged from former `auth_middleware` crate) in `src/middleware/`. Consumes `AuthContext` via the `AuthScope` extractor.

## Architecture Position

```
services + server_core
         |
    routes_app        <-- this crate (includes middleware module)
    /     |      \
server_app  lib_bodhiserver  bodhi/src-tauri
```

State type: `Arc<dyn AppService>` (not `RouterState` -- that was removed).

## Route Groups

5 permissive + 5 restrictive CORS groups. See `PACKAGE.md` for the full per-group permissions table.

**Chat UI sentinel**: `SENTINEL_API_KEY` (`"bodhiapp_sentinel_api_key_ignored"`) — placed by chat UI in pi-ai SDKs; `anthropic_auth_middleware` + `openai_auth_middleware` strip it so session-cookie auth takes over. See `src/middleware/PACKAGE.md`.

## AuthScope Extractor (Critical Pattern)

All route handlers use `AuthScope` (`src/shared/auth_scope_extractor.rs`), a newtype around `AuthScopedAppService`. Replaces the old `Extension<AuthContext>` + `State(state)` dual-extractor.

**Handler signature**: see `src/shared/auth_scope_extractor.rs:19` for the type definition.

Key methods on `AuthScope` (via `Deref` to `AuthScopedAppService`):
- `auth_context()` -- raw `AuthContext` enum
- `require_user_id()` / `require_client_id()` / `require_tenant_id()` -- return `Result<&str, AuthContextError>`
- `tokens()`, `mcps()`, `users()` -- auth-scoped sub-services
- `inference()` -- `Arc<dyn InferenceService>`
- `data_service()`, `setting_service()` -- passthrough accessors (no auth scoping)

Falls back to `AuthContext::Anonymous { deployment: DeploymentMode::Standalone }` when no auth middleware has populated the extension.

**AuthContext**: 5 variants — `Anonymous`, `Session`, `MultiTenantSession`, `ApiToken`, `ExternalApp`. `Session.role` and `MultiTenantSession.role` are `ResourceRole` (not `Option`). Full variant details in `crates/services/CLAUDE.md`.

## Error Handling Chain

Two return types depending on handler audience:

- **Bodhi handlers + middleware**: `Result<_, BodhiErrorResponse>` (`shared/api_error.rs`). Emits `{error: {message, type, code, params?, param?}}` JSON — a wire-level superset of OpenAI's `Error` shape. `params` is the structured HashMap; `param` is its JSON-encoded string form (populated by `BodhiError::new`), so OpenAI-only clients can still read `param`. `BodhiErrorResponse` has a blanket `From<T: AppError>` so service / domain errors convert via `?`.
- **OAI handlers** (only under `src/oai/`): `Result<_, OaiApiError>` (`oai/api_error.rs`). Converts to async-openai's `WrappedError` for OpenAI SDK compatibility. The `param` field is a joined `key=value` string (OpenAI wire spec) instead of a HashMap — deliberately different from `BodhiError`'s JSON-encoded `param`.

Provider proxies (Anthropic, Gemini) wrap `BodhiErrorResponse` into provider-specific envelopes (`AnthropicApiError`, `GeminiApiError`).

Domain error enums wrap service errors via `#[error(transparent)]` + `#[from]`. Error codes auto-generated: `model_route_error-alias_not_found`.

## OpenAPI Registration Checklist

Every new route must:
1. Add `#[utoipa::path(...)]` to handler (generates `__path_<handler>` symbol)
2. Add `API_TAG_<DOMAIN>` constant to `src/shared/constants.rs` (if new domain)
3. Register in `src/shared/openapi.rs`: imports, tags, schemas, paths
4. Regenerate: `cargo run --package xtask openapi`
5. Build TS client: `make build.ts-client`
6. Import from `@bodhiapp/ts-client` in frontend (not hand-rolled types)
7. Verify: `cargo test -p routes_app -- openapi` and `cd crates/bodhi && npm test`

## Anthropic Proxy: LlmLibertyOauth Path

`src/anthropic/routes_anthropic.rs` proxies `/anthropic/v1/messages` and `/anthropic/v1/models*`. The resolver `resolve_anthropic_alias` returns an `AnthropicAliasResolution` enum with two arms:

1. `Native { alias, api_key }` — `Anthropic` and `AnthropicOAuth` formats. Handler forwards via `auth_scope.inference().forward_remote_with_params(...)`.
2. `Liberty { alias, creds }` — `LlmLibertyOauth` format. Liberty credentials are resolved via `providers::resolve_llm_liberty_credentials` (delegates to `services::ai_apis::llm_liberty::ensure_fresh_credentials` — per-alias mutex, skew-window refresh). The handler then builds a `LibertyAnthropicClient` via `auth_scope.ai_api().for_resolved_credentials(&creds, &alias, tenant_id, user_id)?` and calls `forward_request_with_method(...)`. The Liberty client owns the upstream call **and** the 401-retry-with-force-refresh — the route handler does not retry.

**Provider verification** (`creds.provider == "anthropic"`) lives inside the factory's `for_resolved_credentials`; it returns `AiApiClientFactoryError::LibertyProviderUnsupported(...)` (BadRequest) for mis-routed envelopes (e.g. an `openai-codex` envelope on a misconfigured alias). The route handler does not duplicate this check.

**401-retry-with-force-refresh** lives inside `LibertyAnthropicClient::forward_request_with_method`: it makes the upstream call, and on `StatusCode::UNAUTHORIZED` calls the injected `LlmLibertyRefresh::force_refresh(tenant_id, user_id, alias_id)` (which bypasses the skew check), mutates the client's bound access_token in place, and retries once. A second 401 surfaces to the user untouched. This handles the case where Anthropic invalidates access tokens before `expires_at` (e.g. third-party-usage flagging).

`tenant_id`/`user_id` for the Liberty arm come from `auth_scope.require_tenant_id()? / require_user_id()?` so missing auth context surfaces as a typed `AuthContextError` (auto-converted to `AnthropicApiError`) rather than silently producing a malformed refresh call.

## Key Workflow Gotchas (Critical)

**Session clearing on role change**: When a user's role changes, all sessions must be cleared via `session_service`. The handler logs but does not fail if clearing errors.

**AppStatus values**: `Setup` (default), `Ready`, `ResourceAdmin`. `TenantSelection` was removed -- Anonymous{MultiTenant} and MultiTenantSession{client_id: None} with memberships now return `Ready`.

**Single-step app access-request flow**: `POST /apps/request-access` takes only `{app_client_id, requested_role, requested}` (no `flow_type`/`redirect_url`). The app forwards its pre-built Keycloak authorize URL + error URL to the review **page** as query params (backend never sees/stores them). `GET /access-requests/{id}/review` returns `auth_endpoint` (from `build_authorize_endpoint()`) so the page can validate the app-supplied URL. `approve` returns `{status, access_request_scope}`; the page appends the scope to the authorize URL and redirects to Keycloak. `deny` returns `{status}`; the page redirects to the app's error URL with `error=access_denied&error_source=bodhi`. The `token_service` scope-validation + `access_request_id` claim path is unchanged.

See `PACKAGE.md` for full Key Workflow Gotchas list.

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module (e.g., `mcps`, `tokens`)
- `cargo test -p routes_app -- openapi` -- verify OpenAPI spec matches
