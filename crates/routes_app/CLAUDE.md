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
- `LoginError` -- OAuth flow failures (AppInstanceNotFound, SessionInfoNotFound, OAuthError, StateDigestMismatch, MissingState, MissingCode)
- `LogoutError` -- Session destruction failures
- `AccessRequestError` -- Access request workflow (AlreadyPending, AlreadyHasAccess, PendingRequestNotFound, InsufficientPrivileges)
- `UserRouteError` -- Admin user operations (ListFailed, RoleChangeFailed, RemoveFailed)
- `ApiTokenError` -- Token lifecycle (AppRegMissing, AccessTokenMissing, PrivilegeEscalation, InvalidScope, InvalidRole)
- `AppServiceError` -- Setup flow (AlreadySetup, ServerNameTooShort)
- `MetadataError` -- Model metadata operations (InvalidRepoFormat, ListAliasesFailed, AliasNotFound, ExtractionFailed, EnqueueFailed)
- `ModelError` -- Model listing (MetadataFetchFailed)
- `ToolsetValidationError` -- Toolset CRUD validation
- `McpValidationError` -- MCP CRUD validation: `Validation(String)` (generic catch-all), `CsrfStateMismatch` (OAuth state param doesn't match session, BadRequest), `CsrfStateExpired` (OAuth state older than 10 min, BadRequest), `SessionDataMissing` (OAuth session data not found, BadRequest), `TokenExchangeFailed(String)` (token exchange HTTP error, InternalServer), `InvalidUrl(String)` (URL validation failure, BadRequest), `InvalidRedirectUri(String)` (redirect_uri validation failure, BadRequest)
- `AppAccessRequestError` -- App access request workflow (review, approve, deny)
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

**Upstream dependencies** (crates this depends on):
- [`objs`](../objs/CLAUDE.md) -- domain types, errors, API tag constants
- [`services`](../services/CLAUDE.md) -- business logic (AppService, ToolService, McpService, etc.)
- [`auth_middleware`](../auth_middleware/CLAUDE.md) -- `AuthContext` extension, session helpers
- [`server_core`](../server_core/CLAUDE.md) -- `RouterState`, `SharedContext`

**Downstream consumers** (crates that depend on this):
- [`server_app`](../server_app/CLAUDE.md) -- standalone HTTP server composes routes
- [`lib_bodhiserver`](../lib_bodhiserver/CLAUDE.md) -- embeddable library composes routes
- [`bodhi/src-tauri`](../bodhi/src-tauri/CLAUDE.md) -- Tauri desktop app composes routes

## Cross-Crate Integration Patterns

### Service Layer Coordination
All route handlers access business logic through `RouterState`, which provides `AppService`. This registry exposes typed service traits:
- `data_service()` -- Local model alias CRUD, unified alias listing (User + Model + API)
- `hub_service()` -- HuggingFace cache scanning, model file listing
- `db_service()` -- SQLite persistence for tokens, access requests, metadata, download tracking
- `auth_service()` -- OAuth2 code exchange, role assignment, user management, client registration
- `app_instance_service()` -- App registration info (OAuth client credentials) and app status lifecycle
- `setting_service()` -- Configuration management, environment detection
- `session_service()` -- Session clearing for role changes
- `tool_service()` -- Toolset CRUD, type management, execution
- `mcp_service()` -- MCP server CRUD, tool discovery, execution
- `access_request_service()` -- User and app access request workflows
- `network_service()` -- Network availability checks
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
Token creation enforces a strict privilege matrix. `TokenScope` has two variants: `User` and `PowerUser`.
- `User` role can only create `scope_token_user` tokens
- `PowerUser` and higher roles can create `scope_token_user` or `scope_token_power_user` tokens
- The match is exhaustive over these two `TokenScope` variants, with no catch-all arm
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

### MCP Server Management
All MCP routes are unified under the `/bodhi/v1/mcps/` prefix in the `routes_mcp/` module:
- **Instance CRUD**: list, create, get, update, delete (`/bodhi/v1/mcps`)
- **Tool operations**: fetch tools, refresh tools, execute tool (`/bodhi/v1/mcps/{id}/tools`)
- **Server allowlist**: list, create, get, update MCP server URLs (`/bodhi/v1/mcps/servers`)
- **Unified auth configs**: `POST /bodhi/v1/mcps/auth-configs` creates header or OAuth configs via `CreateAuthConfigBody` (discriminated union with `type` field + `mcp_server_id`). `GET /bodhi/v1/mcps/auth-configs?mcp_server_id=xxx` lists configs by server. `GET/DELETE /bodhi/v1/mcps/auth-configs/{id}` for single config operations.
- **OAuth login/token**: `POST /bodhi/v1/mcps/auth-configs/{id}/login` and `/token` for OAuth flows
- **OAuth discovery**: `POST /bodhi/v1/mcps/oauth/discover-as`, `POST /bodhi/v1/mcps/oauth/discover-mcp`
- **Standalone DCR**: `POST /bodhi/v1/mcps/oauth/dynamic-register` for Dynamic Client Registration
- **OAuth tokens**: `GET/DELETE /bodhi/v1/mcps/oauth-tokens/{id}`
- Auth: session-only for CRUD and tool ops; server enable/disable is admin-only
- OAuth token exchange validates CSRF `state` parameter from session
- Authorization URL uses proper URL encoding via `url::Url` (not string concatenation)
- Auth header and OAuth token handlers enforce ownership via `user_id`
- API types use `McpAuthType` enum (`Public`, `Header`, `Oauth`) — OAuth pre-registered vs dynamic is distinguished by `registration_type` field, not separate enum variants
- `OAuthTokenExchangeRequest` includes `state: String` for CSRF

**Test organization**: All route modules use `test_*.rs` sibling files (not `tests/` subdirectories). Reference implementation: `routes_mcp/` module. Convention:
- Each handler file declares `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;` (Pattern A)
- Auth tier tests are declared from `mod.rs` as `test_<module>_auth.rs` (Pattern B)
- Large test files are split by concern: `test_<handler>_crud.rs`, `test_<handler>_validation.rs`, `test_<handler>_auth.rs`
- Shared test infrastructure lives in `test_utils/` (router builders, auth helpers, assertions)

### App Access Request Workflow
App access request routes (`routes_apps/`) handle external application resource access using a role-based model. The flow centers on `requested_role` (what the app asks for) and `approved_role` (what the user grants):

1. External app creates access request via `create_access_request_handler` (`POST /bodhi/v1/apps/request-access`) -- always creates a "draft" status request with `requested_role`, optionally requested tool types and MCP servers. Returns `review_url` for user approval. No auto-approve logic; all requests go through user review.
2. External app polls status via `get_access_request_status_handler` (`GET /bodhi/v1/apps/access-requests/{id}`) -- returns `requested_role`, `approved_role` (populated when approved), and `access_request_scope`. Requires `app_client_id` query param for verification.
3. User reviews via `get_access_request_review_handler` (`GET /bodhi/v1/access-requests/{id}/review`) -- returns full request details including `requested_role`, tool type info with user's configured instances, and MCP server info with user's connected instances. Requires session auth.
4. User approves via `approve_access_request_handler` (`PUT /bodhi/v1/access-requests/{id}/approve`) -- request body includes `approved_role` (the role to grant) and `approved` resource selections (toolset/MCP instance mappings). Validates instance ownership, enablement, and API key configuration before delegating to service.
5. User denies via `deny_access_request_handler` (`POST /bodhi/v1/access-requests/{id}/deny`)

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

### Service Mocking Strategy in Route Tests

**Default: prefer `build_test_router()` with real services.** Only mock at external-system boundaries where a real implementation requires hardware, network access, or hard-to-control external state.

#### What `build_test_router()` provides
Real implementations wired automatically:
- **SQLite DB** (`DbService`) -- in-memory via tempfile; records persist within the test
- **SessionService** -- real SQL-backed session store; `create_authenticated_session()` inserts a JWT-bearing session
- **DataService** -- file-based alias storage in the temp home dir
- **AppInstanceService** -- real SQLite-backed app instance persistence (OAuth client credentials, app status)
- **HubService** -- HuggingFace cache scanning (reads `~/.cache/huggingface/hub`)

Stubbed/mocked because they cross external boundaries:
- `MockSharedContext` -- no LLM; requires a real llama.cpp binary and downloaded model
- `StubNetworkService` -- returns a fixed IP instead of querying the network
- `StubQueue` -- no-op task queue instead of a real background worker

Test setup pattern:
```rust
let (router, app_service, _temp) = build_test_router().await?;
let admin_cookie = create_authenticated_session(
  app_service.session_service().as_ref(), &["resource_admin"],
).await?;
// Use setup helpers to create prerequisite DB rows via the API:
let server_id = setup_mcp_server_in_db(&router, &admin_cookie).await?;
```

#### When to use service mocks (`MockMcpService`, `MockToolService`, …)
Use a service-level mock **instead of** `build_test_router()` only when:

1. **The service makes external HTTP calls** that would hit real servers in a test -- e.g., `McpService::discover_oauth_metadata()`, `McpService::fetch_tools_for_server()`, `McpService::dynamic_register_client()`. These call external URLs that tests cannot control.
2. **You need a specific error path** that is impractical to reproduce via real DB state -- e.g., `McpServerError::UrlAlreadyExists` (simpler to return from a mock than to insert a duplicate row through the API).
3. **Testing a single handler in complete isolation** from all infrastructure -- useful when the test value is entirely in verifying the mapping between service output and HTTP response shape.

Build a minimal router with just the routes under test:
```rust
async fn test_router_for_crud(mock: MockMcpService) -> anyhow::Result<Router> {
  let state = build_mcp_test_state(mock).await?;
  Ok(Router::new()
    .route("/mcps", post(create_mcp_handler))
    // ... only routes being tested
    .with_state(state))
}
```
Inject auth via `RequestAuthContextExt::with_auth_context()` (bypasses the session layer entirely):
```rust
let request = Request::builder()
  .method("POST").uri("/mcps").body(Body::from(body))?
  .with_auth_context(AuthContext::test_session("user123", "testuser", ResourceRole::User));
```

**Do not mock** the DB service, session service, or any other service that has a real, fast in-memory implementation -- using the real implementation gives higher confidence and simpler tests.

#### OAuth flow tests with a real session layer
When two API calls must share session state (e.g., OAuth login → token exchange), build a custom router that includes the session layer but still mocks the external MCP service:
```rust
let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
let app_service = AppServiceStubBuilder::default()
  .mcp_service(Arc::new(mock_mcp_service))
  .with_default_session_service(session_service.clone())
  .build().await?;
let router = Router::new()
  .route("/mcps/auth-configs/{id}/login", post(oauth_login_handler))
  .route("/mcps/auth-configs/{id}/token", post(oauth_token_exchange_handler))
  .layer(app_service.session_service().session_layer())  // required for session persistence
  .with_state(state);
```
Return `session_service` alongside the router so tests can inspect or pre-populate session records directly via `session_service.get_session_store().create(&mut record).await?`.

#### Summary decision table

| What you are testing | Pattern |
|---|---|
| CRUD workflow persisted in DB | `build_test_router()` |
| Single handler, service has no external calls | `build_test_router()` (preferred) or mock |
| Single handler, service calls external HTTP | Mock the service (`MockMcpService`, etc.) |
| Specific service error path (hard to trigger via DB) | Mock the service |
| LLM inference / streaming responses | `MockSharedContext.expect_forward_request()` |
| OAuth login → token exchange (two-call flow) | Custom router with real `DefaultSessionService` |
| Full LLM stack (real llama.cpp) | `build_live_test_router()` |

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
