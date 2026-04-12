# routes_app -- CLAUDE.md

**Companion docs** (load as needed):
- `PACKAGE.md` -- Implementation details, file index, error enum reference, Route Group Architecture table, Domain Module Structure, Handler/JSON Extraction Conventions, Proxy sections, Key Workflow Gotchas
- `src/middleware/CLAUDE.md` -- Middleware-specific documentation (auth, authorization, token service)
- `src/middleware/PACKAGE.md` -- Middleware module index and error enum reference
- `TESTING.md` -- Test patterns, mocking strategy, router construction
- `TECHDEBT.md` -- Known issues and planned refactors

## Purpose

API orchestration layer: HTTP endpoint handlers for all BodhiApp application routes. Defines `ApiError`/`OpenAIApiError`/`ErrorBody` in `shared/` (moved from `services`). Includes authentication/authorization middleware (merged from former `auth_middleware` crate) in `src/middleware/`. Consumes `AuthContext` via the `AuthScope` extractor.

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

Service error -> domain `<X>RouteError` (this crate) -> `ApiError` (`shared/api_error.rs`) -> OpenAI-compatible JSON.

`ApiError`, `OpenAIApiError`, `ErrorBody` are in `routes_app::shared` (import as `use crate::ApiError`, NOT `use services::ApiError`).

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

## Key Workflow Gotchas (Critical)

**Session clearing on role change**: When a user's role changes, all sessions must be cleared via `session_service`. The handler logs but does not fail if clearing errors.

**AppStatus values**: `Setup` (default), `Ready`, `ResourceAdmin`. `TenantSelection` was removed -- Anonymous{MultiTenant} and MultiTenantSession{client_id: None} with memberships now return `Ready`.

See `PACKAGE.md` for full Key Workflow Gotchas list.

## Commands

- `cargo test -p routes_app` -- all tests
- `cargo test -p routes_app -- <module>` -- specific module (e.g., `mcps`, `tokens`)
- `cargo test -p routes_app -- openapi` -- verify OpenAPI spec matches
