# middleware -- CLAUDE.md

**Location**: `crates/routes_app/src/middleware/`
**Companion docs**: `PACKAGE.md` in this directory for implementation details, AuthContext full variant listing, middleware step lists, session key constants, DefaultTokenService details, Tenant Resolution Strategy, ExternalApp Role Derivation, and Test Utilities.

Previously the standalone `auth_middleware` crate. Merged into `routes_app` as an internal module.

## Purpose

HTTP authentication and authorization middleware. Validates JWT tokens and sessions, injects `AuthContext` into request extensions, enforces role-based access control.

## Architecture Position

```
services (AuthContext, AuthService, TenantService, etc.)
                  |
     routes_app::middleware      <-- this module
                  |
     routes_app route handlers (via AuthScope extractor)
```

State type: `State<Arc<dyn AppService>>`.

## Which Middleware Runs on Which Path

- `auth_middleware` (strict) — all session-protected route groups; fails with `AuthError::InvalidAccess` on missing/invalid auth
- `optional_auth_middleware` (permissive) — `optional_auth` group; falls back to `AuthContext::Anonymous` on any failure
- `api_auth_middleware` (authorization) — all authenticated groups; enforces `ResourceRole` / `TokenScope` / `UserScope` hierarchy
- `access_request_auth_middleware` (entity-level) — `apps_apis` group only; validates `ExternalApp` against approved access requests
- `anthropic_auth_middleware` — `/anthropic/*` and `/v1/messages`; strips `SENTINEL_API_KEY`, rewrites `x-api-key` → `Authorization: Bearer`
- `gemini_auth_middleware` — `/v1beta/*`; strips `SENTINEL_API_KEY` from `x-goog-api-key`, rewrites non-sentinel `x-goog-api-key` → `Authorization: Bearer`
- `openai_auth_middleware` — `/v1/*`; strips `SENTINEL_API_KEY` from `Authorization` / `x-api-key`

## Critical Gotcha

`AuthContext.role` on `Session` and `MultiTenantSession` is `ResourceRole` (not `Option<ResourceRole>`). `api_auth_middleware` pattern-matches `Session { role: Some(role) }` — `None` means no role assigned, treated as `MissingAuth`. Full variant details in `PACKAGE.md`.

## Chat UI Sentinel

`SENTINEL_API_KEY` (`"bodhiapp_sentinel_api_key_ignored"`) — placed by chat UI in pi-ai SDKs; `anthropic_auth_middleware` + `openai_auth_middleware` strip it so session-cookie auth takes over. See `PACKAGE.md`.

## MiddlewareError

`error.rs` -- captures `AppError` metadata, implements `IntoResponse`. Has blanket `From<T: AppError + 'static>` impl.

No `"param": null` in JSON -- only adds `param` key when args is non-empty.

## Commands

- `cargo test -p routes_app` -- all tests (includes middleware tests)
- `cargo test -p routes_app -- middleware` -- middleware-specific tests
- `cargo test -p routes_app -- test_live_auth_middleware` -- live OAuth2 tests (requires running OAuth2 server)
