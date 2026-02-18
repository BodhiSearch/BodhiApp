# PACKAGE.md - routes_app

This document provides the implementation index and navigation guide for the `routes_app` crate, covering application API endpoints, domain error types, and testing patterns.

*For architectural documentation and design rationale, see [crates/routes_app/CLAUDE.md](crates/routes_app/CLAUDE.md)*

## Module Structure

Entry point: `crates/routes_app/src/lib.rs` -- Re-exports all public modules with conditional `test_utils` compilation.

### Route Modules (organized by domain subdirectory)

| Module | Directory | Purpose |
|--------|-----------|---------|
| `routes_auth` | `src/routes_auth/` | OAuth2 initiate/callback (`login.rs`), logout |
| `routes_users` | `src/routes_users/` | User access requests (`access_request.rs`), admin management (`management.rs`), user info (`user_info.rs`) |
| `routes_apps` | `src/routes_apps/` | App access requests: create, status, review, approve, deny (`handlers.rs`) |
| `routes_api_token` | `src/routes_api_token/` | API token create, update, list with privilege escalation prevention |
| `routes_api_models` | `src/routes_api_models/` | API model config CRUD |
| `routes_models` | `src/routes_models/` | Model alias listing (`aliases.rs`), metadata refresh (`metadata.rs`), model pull (`pull.rs`) |
| `routes_toolsets` | `src/routes_toolsets/` | Toolset CRUD, execution, type management, app config enable/disable (`toolsets.rs`) |
| `routes_mcps` | `src/routes_mcps/` | MCP instance CRUD, tool discovery, tool execution, server allowlist (`mcps.rs`) |
| `routes_oai` | `src/routes_oai/` | OpenAI-compatible chat completions, models, embeddings |
| `routes_ollama` | `src/routes_ollama/` | Ollama-compatible models, show, chat |
| `routes_settings` | `src/routes_settings/` | Settings list, update, reset |
| `routes_setup` | `src/routes_setup/` | App setup, ping, health, app info |
| `routes_dev` | `src/routes_dev.rs` | Development/debug endpoints (db-reset) |
| `routes_ping` | `src/routes_ping.rs` | Ping endpoint |
| `routes_proxy` | `src/routes_proxy.rs` | Request proxy |

### Supporting Modules

| Module | File | Purpose |
|--------|------|---------|
| `shared` | `src/shared/mod.rs` | Shared infrastructure |
| `shared/openapi` | `src/shared/openapi.rs` | OpenAPI spec generation with Utoipa, endpoint constants |
| `shared/common` | `src/shared/common.rs` | `RedirectResponse` DTO |
| `shared/pagination` | `src/shared/pagination.rs` | Pagination params, paginated response types |
| `shared/utils` | `src/shared/utils.rs` | Helper functions (e.g., `extract_request_host`) |
| `api_dto` | `src/api_dto.rs` | API DTOs and endpoint constants |
| `routes` | `src/routes.rs` | Main route composition |

### Test Utilities (`test-utils` feature)

| Module | File | Purpose |
|--------|------|---------|
| `test_utils` | `src/test_utils/mod.rs` | Test helper macros and constants |
| `alias_response` | `src/test_utils/alias_response.rs` | Builder-based test fixtures for `AliasResponse` |
| `router` | `src/test_utils/router.rs` | Test router construction helpers |
| `assertions` | `src/test_utils/assertions.rs` | Assertion helpers for route tests |

## AuthContext Handler Pattern

All handlers receive user identity through `Extension<AuthContext>` from Axum:

```rust
async fn handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<..., ApiError> {
  let user_id = auth_context.user_id().expect("requires auth middleware");
}
```

Test injection via `RequestAuthContextExt::with_auth_context()`:

```rust
use auth_middleware::RequestAuthContextExt;
let request = Request::builder()
  .uri("/some/endpoint")
  .body(Body::empty())
  .unwrap()
  .with_auth_context(AuthContext::test_session("test-user", "testname", ResourceRole::Admin));
```

## Domain Error Enums

Each route module defines its own error enum. All use `errmeta_derive::ErrorMeta`.

| Error Enum | Module | Key Variants |
|------------|--------|-------------|
| `LoginError` | `routes_auth/` | AppRegInfoNotFound, OAuthError, StateDigestMismatch, MissingState, MissingCode |
| `LogoutError` | `routes_auth/` | SessionDelete |
| `AccessRequestError` | `routes_users/` | AlreadyPending, AlreadyHasAccess, PendingRequestNotFound, InsufficientPrivileges |
| `UserRouteError` | `routes_users/` | ListFailed, RoleChangeFailed, RemoveFailed |
| `AppAccessRequestError` | `routes_apps/` | App access request review, approve, deny errors |
| `ApiTokenError` | `routes_api_token/` | AppRegMissing, PrivilegeEscalation, InvalidScope, InvalidRole |
| `AppServiceError` | `routes_setup/` | AlreadySetup, ServerNameTooShort |
| `MetadataError` | `routes_models/` | InvalidRepoFormat, AliasNotFound, ExtractionFailed, EnqueueFailed |
| `ModelError` | `routes_models/` | MetadataFetchFailed |
| `ToolsetValidationError` | `routes_toolsets/` | Validation |
| `McpValidationError` | `routes_mcps/` | MCP CRUD validation errors |
| `SettingsError` | `routes_settings/` | NotFound, BodhiHome, Unsupported |
| `CreateAliasError` | `routes_models/` | AliasNotPresent, transparent wrappers |
| `UserInfoError` | `routes_users/` | InvalidHeader, EmptyToken |

## API Token Privilege Matrix

Defined in `routes_api_token/`, the `create_token_handler` enforces:

| User Role | Allowed Scopes |
|-----------|---------------|
| `User` | `scope_token_user` only |
| `PowerUser`, `Manager`, `Admin` | `scope_token_user`, `scope_token_power_user` |
| Any | Cannot create `scope_token_manager` or `scope_token_admin` |

## Toolset Dual-Auth Model

In `routes_toolsets/toolsets.rs`, handlers match on `AuthContext` variant:
- `AuthContext::Session` -- full access to all toolset types and configs
- `AuthContext::ExternalApp` -- filtered by `scope_toolset-*` scopes in the token
- `AuthContext::ApiToken` -- not allowed for toolset endpoints

## MCP Route Auth Model

In `routes_mcps/mcps.rs`:
- Instance CRUD and tool operations: session-only (no API tokens, no OAuth)
- List MCPs: session + OAuth
- Server allowlist enable/disable: admin-only

## OpenAPI Specification

`src/shared/openapi.rs` defines `BodhiOpenAPIDoc` with:
- All endpoint paths registered via `#[openapi(paths(...))]`
- All request/response schemas via `#[openapi(components(schemas(...)))]`
- Tags: system, auth, api_keys, api_models, models, settings, openai, ollama, users, access_requests, toolsets, mcps
- `OpenAPIEnvModifier` for environment-specific server URL and bearer auth security scheme

## Testing Patterns

### Router Construction in Tests

Each route module constructs test routers with mocked services:

```rust
fn test_router(mock_service: MockToolService) -> Router {
  let app_service = AppServiceStubBuilder::default()
    .with_tool_service(Arc::new(mock_service))
    .build().unwrap();
  let state: Arc<dyn RouterState> = Arc::new(
    DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    )
  );
  routes_toolsets(state)
}
```

### Auth Context Injection in Tests

Tests use `RequestAuthContextExt::with_auth_context()` from `auth_middleware` test-utils:

```rust
let request = Request::builder()
  .uri("/bodhi/v1/toolsets")
  .body(Body::empty())
  .unwrap()
  .with_auth_context(AuthContext::test_session("user", "name", ResourceRole::Admin));
```

Factory methods: `AuthContext::test_session()`, `AuthContext::test_session_with_token()`, `AuthContext::test_session_no_role()`, `AuthContext::test_api_token()`, `AuthContext::test_external_app()`.

### Integration Tests with Real Database

Access request and session tests use real SQLite databases via `test_db_service` fixtures. Session clearing tests create real session records and verify they are removed after role changes.

## Commands

**Run all tests**: `cargo test -p routes_app`
**Run specific module tests**: `cargo test -p routes_app routes_toolsets` (or any module name)
**Run with test-utils feature**: `cargo test -p routes_app --features test-utils`
