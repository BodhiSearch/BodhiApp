# CLAUDE.md

This file provides guidance to Claude Code when working with the `routes_app` crate.

See [crates/routes_app/PACKAGE.md](crates/routes_app/PACKAGE.md) for implementation details.

## Purpose

The `routes_app` crate serves as BodhiApp's **application API orchestration layer**, implementing comprehensive HTTP endpoints for model management, authentication, API token management, toolset management, user administration, and application configuration with OpenAPI documentation generation.

## Key Domain Architecture

### Domain-Specific Error Handling Strategy
The crate has deliberately moved away from generic HTTP error wrappers (such as `BadRequestError`, `InternalServerError`, `ConflictError`) in favor of **domain-specific error enums** per module. Each route module defines its own error enum that precisely describes what went wrong in domain terms, rather than merely mapping to HTTP status codes. This decision was made to:

- Produce deterministic, machine-readable error codes (e.g., `access_request_error-already_pending` vs a generic `conflict_error`)
- Enable per-variant error metadata via `errmeta_derive::ErrorMeta`, giving each variant its own `ErrorType`, `code`, and message template
- Eliminate ambiguity about what condition actually triggered an error response
- Support downstream error handling by API clients that need to distinguish between different failure modes

**Domain error enums defined in this crate:**
- `LoginError` -- OAuth flow failures (AppRegInfoNotFound, SessionInfoNotFound, OAuthError, StateDigestMismatch, MissingState, MissingCode)
- `LogoutError` -- Session destruction failures
- `AccessRequestError` -- Access request workflow (AlreadyPending, AlreadyHasAccess, PendingRequestNotFound, InsufficientPrivileges)
- `UserRouteError` -- Admin user operations (ListFailed, RoleChangeFailed, RemoveFailed)
- `ApiTokenError` -- Token lifecycle (AppRegMissing, AccessTokenMissing, PrivilegeEscalation, InvalidScope, InvalidRole)
- `AppServiceError` -- Setup flow (AlreadySetup, ServerNameTooShort)
- `MetadataError` -- Model metadata operations (InvalidRepoFormat, ListAliasesFailed, AliasNotFound, ExtractionFailed, EnqueueFailed)
- `ModelError` -- Model listing (MetadataFetchFailed)
- `ToolsetValidationError` -- Toolset CRUD validation
- `SettingsError` -- Settings management (NotFound, BodhiHome, Unsupported)
- `CreateAliasError` -- Model alias creation

All enums use the `#[error_meta(trait_to_impl = AppError)]` pattern from `errmeta_derive`, mapping each variant to an `ErrorType` that determines the HTTP status code via the `ApiError` conversion in `objs`.

### AuthContext Extension Pattern
Route handlers receive user identity through `Extension<AuthContext>` from Axum, where `AuthContext` is an enum defined in `auth_middleware`. The auth middleware populates this extension before handlers run. This replaces the previous approach of individual typed extractors (`ExtractUserId`, `ExtractToken`, `ExtractRole`, etc.) and manual `HeaderMap` parsing.

**AuthContext variants:**
- `AuthContext::Anonymous` -- Unauthenticated user (used behind `optional_auth_middleware` middleware)
- `AuthContext::Session { user_id, username, role: Option<ResourceRole>, token }` -- Browser session auth
- `AuthContext::ApiToken { user_id, scope: TokenScope, token }` -- API token auth
- `AuthContext::ExternalApp { user_id, scope: UserScope, token, azp, access_request_id: Option<String> }` -- External app OAuth

**Handler pattern -- required auth endpoints:**
```rust
async fn handler(
  Extension(auth_context): Extension<AuthContext>,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<..., ApiError> {
  // Pattern match when you need multiple fields:
  let AuthContext::Session { ref user_id, ref token, .. } = auth_context else {
    return Err(/* appropriate error */);
  };
  // Or use convenience methods when you need just one field:
  let user_id = auth_context.user_id().expect("requires auth middleware");
  let token = auth_context.token().expect("requires auth middleware");
```
The `.expect()` calls are safe on required-auth endpoints because the auth middleware guarantees `AuthContext` is set (non-Anonymous) before the handler runs.

**Optional auth endpoints:**
Handlers behind `optional_auth_middleware` receive `AuthContext::Anonymous` for unauthenticated users. These handlers must handle `None` from `user_id()`/`token()` gracefully and must not use `.expect()`.

**Testing pattern:**
Tests use `RequestAuthContextExt::with_auth_context()` from `auth_middleware` test-utils to inject auth context into test requests:
```rust
use auth_middleware::RequestAuthContextExt;
let request = Request::builder()
  .uri("/some/endpoint")
  .body(Body::empty())
  .unwrap()
  .with_auth_context(AuthContext::test_session("test-user", "testname", ResourceRole::Admin));
```
Factory methods: `AuthContext::test_session()`, `AuthContext::test_session_with_token()`, `AuthContext::test_session_no_role()`, `AuthContext::test_api_token()`, `AuthContext::test_external_app()`.

## Architecture Position

The `routes_app` crate sits in the **API layer** of BodhiApp's architecture:
- **Depends on**: `objs` (domain types, errors), `services` (business logic), `commands` (CLI orchestration), `auth_middleware` (AuthContext extension, session helpers), `server_core` (RouterState)
- **Consumed by**: `server_app` (standalone server), `bodhi` (Tauri app)
- **Parallel to**: `routes_oai` (OpenAI-compatible endpoints)

## Cross-Crate Integration Patterns

### Service Layer Coordination
All route handlers access business logic through `RouterState`, which provides `AppService`. This registry exposes typed service traits:
- `data_service()` -- Local model alias CRUD, unified alias listing (User + Model + API)
- `hub_service()` -- HuggingFace cache scanning, model file listing
- `db_service()` -- SQLite persistence for tokens, access requests, metadata, download tracking
- `auth_service()` -- OAuth2 code exchange, role assignment, user management, client registration
- `secret_service()` -- App registration info, app status lifecycle
- `setting_service()` -- Configuration management, environment detection
- `session_service()` -- Session clearing for role changes
- `tool_service()` -- Toolset CRUD, type management, execution
- `time_service()` -- Testable time source (never use `Utc::now()` directly)
- `queue_producer()` -- Async task enqueueing for metadata refresh

### Command Layer Integration
Model creation and pull operations delegate to the `commands` crate (`CreateCommand`, `PullCommand`) rather than calling services directly. This ensures CLI and HTTP operations share identical business logic and validation.

### Error Translation Chain
Service errors flow through a well-defined chain: service-specific error -> domain error enum (defined in this crate) -> `ApiError` (from `objs`) -> OpenAI-compatible JSON response. Each domain error enum wraps relevant service errors via `#[error(transparent)]` with `#[from]` conversion, while also defining handler-specific error variants.

## API Orchestration Workflows

### OAuth2 Authentication Flow
1. `auth_initiate_handler` -- Generates PKCE challenge, stores state in session, returns authorization URL with dynamic callback URL detection (supports loopback, network IP, and explicit public host configurations)
2. `auth_callback_handler` -- Validates CSRF state, exchanges authorization code, handles `ResourceAdmin` first-login flow (make-resource-admin, token refresh, redirect to download-models), stores tokens in session
3. `logout_handler` -- Destroys session, returns login URL
4. `request_access_handler` -- App-to-app resource access with version-based caching and toolset scope management

### API Token Privilege Escalation Prevention
Token creation enforces a strict privilege matrix:
- `User` role can only create `scope_token_user` tokens
- `PowerUser`, `Manager`, `Admin` can create `scope_token_user` or `scope_token_power_user` tokens
- No role can create `scope_token_manager` or `scope_token_admin` tokens
- Tokens use cryptographic random generation with `bodhiapp_` prefix, SHA-256 hashing, and prefix-based lookup

### User Access Request Workflow
1. User requests access via `user_request_access_handler` (must have no existing role, no pending request)
2. Admin/Manager reviews pending requests via `list_pending_requests_handler`
3. Approval via `approve_request_handler` validates role hierarchy, assigns role via auth service, clears all user sessions
4. Rejection via `reject_request_handler` updates status

### Toolset Management
Toolset routes use a dual-auth model based on `AuthContext` variant:
- `AuthContext::Session` grants full access to all toolset types
- `AuthContext::ExternalApp` restricts access to toolset types matching `scope_toolset-*` scopes in the token
- The handler matches on the `AuthContext` variant to distinguish these two auth modes

### Model Metadata Refresh
Supports two modes via discriminated union request body:
- Bulk async (`{"source": "all"}`) -- Enqueues background task, returns 202 Accepted
- Single sync (`{"source": "model", ...}`) -- Extracts metadata immediately, returns 200 with enriched response

## Important Constraints

### Time Handling
Always use `app_service.time_service().utc_now()` instead of `Utc::now()`. This is critical for testability -- the `TimeService` trait allows tests to inject deterministic timestamps.

### Session Clearing on Role Changes
When a user's role changes (via access request approval or direct role change), all existing sessions for that user must be cleared. This ensures the new role takes effect immediately rather than waiting for session expiry. The `change_user_role_handler` logs but does not fail the operation if session clearing encounters an error.

### Settings Edit Allowlist
Only specific settings can be modified via the API (`BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`). `BODHI_HOME` can only be changed via environment variable. All other settings return `SettingsError::Unsupported`.

### Network Installation Support
The setup and login flows dynamically detect the request host to support network installations where the server is accessed from different machines. When `BODHI_PUBLIC_HOST` is not explicitly configured, the handler extracts the `Host` header to construct callback URLs. When explicitly configured (e.g., RunPod deployment), only the configured host is used.

### Testing Philosophy: Route-Level Unit Tests vs Server Integration Tests
The `routes_app` crate's live tests (`test_live_*`) are **route-level unit tests** that test individual API endpoints in isolation, not full application integration scenarios:

**What routes_app live tests do:**
- Use `build_live_test_router()` to create a `Router` with real services (real llama.cpp, real HF cache)
- Test single API endpoint per test via `router.oneshot(request)`
- Authenticate with `create_authenticated_session()` (session cookie injection, no OAuth flow)
- Assert on single response structure and content
- **Single-turn only** - one request, one response, cleanup

**What routes_app live tests do NOT do:**
- Multiple sequential API calls (multi-turn workflows)
- Full HTTP server lifecycle (TCP listener, signal handling, graceful shutdown)
- OAuth2 authorization code flow (only session-based auth)
- Cross-request state management (session persistence across requests)
- Real-world deployment scenarios (network access, multiple clients)

**When to use server_app integration tests instead:**
Multi-turn workflows requiring multiple API calls belong in `server_app` tests, which run a full HTTP server with TCP listener:
- Multi-turn tool calling (initial tool call → tool result → final answer)
- OAuth2 code exchange flow (initiate → callback → protected endpoint)
- Sequential operations requiring session persistence
- Testing server lifecycle (startup, shutdown, signal handling)
- Testing real HTTP semantics (redirects, cookies, streaming over network)

**Example distinction:**
- ✅ `routes_app`: Single-turn tool calling test - send request with tools, assert response has `finish_reason: "tool_calls"`
- ❌ `routes_app`: Multi-turn tool calling - NOT supported, belongs in `server_app`
- ✅ `server_app`: Multi-turn tool calling - request tool call, extract tool_call_id, send tool result, assert final answer

This distinction keeps `routes_app` tests fast (no TCP listener overhead), focused (route handler logic only), and easy to debug (single request-response cycle).

## Extension Patterns

### Adding New Application Endpoints
1. Define a domain-specific error enum with `#[error_meta(trait_to_impl = AppError)]` variants for every distinct failure mode
2. Accept `Extension(auth_context): Extension<AuthContext>` for user identity -- pattern match the variant or use convenience methods (`user_id()`, `token()`) as appropriate
3. Add `#[utoipa::path(...)]` annotations with comprehensive request/response examples and security requirements
4. Coordinate through `RouterState` for service access
5. For complex multi-service operations, consider delegating to the `commands` crate
6. **Register in OpenAPI and generate TypeScript types** -- see the checklist below

### OpenAPI Registration and TypeScript Client Checklist
Every new route must complete these steps to keep the OpenAPI spec, generated TypeScript client, and frontend in sync:

**Step 1 -- Add `#[utoipa::path]` to every handler**
Annotate each handler function following the pattern in existing modules (e.g., `routes_toolsets/toolsets.rs`, `routes_mcps/mcps.rs`):
```rust
#[utoipa::path(
  get,
  path = ENDPOINT_MCPS,
  tag = API_TAG_MCPS,
  operation_id = "listMcps",
  responses(
    (status = 200, description = "List of user's MCP instances", body = ListMcpsResponse),
  ),
  security(("bearer" = []))
)]
```
The `#[utoipa::path]` macro generates `__path_<handler_name>` symbols used by the OpenAPI derive macro.

**Step 2 -- Add an API tag constant (if new domain)**
Add `pub const API_TAG_<DOMAIN>: &str = "<domain>";` to `crates/objs/src/api_tags.rs`. This is re-exported via `objs::*`.

**Step 3 -- Register in `openapi.rs`**
In `crates/routes_app/src/shared/openapi.rs`, update the `#[derive(OpenApi)]` on `BodhiOpenAPIDoc`:
- **Imports**: Add `use crate::{ <DTOs>, __path_<handler>... };` and `use objs::{ <domain types>, API_TAG_<DOMAIN> };`
- **Tags**: Add `(name = API_TAG_<DOMAIN>, description = "...")` to the `tags(...)` block
- **Schemas**: Add all request/response DTOs and domain types to the `schemas(...)` block inside `components(...)`
- **Paths**: Add all `<handler_name>` entries to the `paths(...)` block

All request/response types must derive `utoipa::ToSchema`. Domain types from `objs` (e.g., `McpServer`, `McpTool`) must also derive `ToSchema`.

**Step 4 -- Regenerate OpenAPI spec**
```bash
cargo run --package xtask openapi
```
This writes `openapi.json` at the project root. The `test_all_endpoints_match_spec` test will fail until this is run.

**Step 5 -- Build the TypeScript client**
```bash
make build.ts-client
```
This generates typed interfaces in `ts-client/src/types/types.gen.ts` and the OpenAPI schema in `ts-client/src/openapi-typescript/openapi-schema.ts`.

**Step 6 -- Use generated types in the frontend**
Import from `@bodhiapp/ts-client` instead of hand-rolling interfaces. Follow the pattern in `crates/bodhi/src/hooks/useMcps.ts` or `useToolsets.ts`:
```typescript
import { McpResponse, CreateMcpRequest, McpTool } from '@bodhiapp/ts-client';
```
Re-export types for consumers of the hook module:
```typescript
export type { McpResponse, CreateMcpRequest, McpTool };
```

**Step 7 -- Verify**
- `cargo test -p routes_app -- openapi` -- ensures OpenAPI spec matches `openapi.json`
- `npm run test` (from `crates/bodhi`) -- ensures frontend compiles and tests pass with generated types
- `cargo fmt --all && npm run format` (from `crates/bodhi`) -- formatting

### Adding New Error Variants
When extending existing error enums:
1. Add the variant with `#[error("...")]` message template and `#[error_meta(error_type = ErrorType::...)]`
2. The error code is auto-generated from the enum name and variant name (e.g., `LoginError::MissingState` -> `login_error-missing_state`)
3. Use `#[error(transparent)]` with `#[from]` for wrapping upstream service errors
4. Test the error code and HTTP status code in integration tests
