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
- `UserManagementError` -- Admin user operations (ListFailed, RoleChangeFailed, RemoveFailed)
- `ApiTokenError` -- Token lifecycle (AppRegMissing, AccessTokenMissing, PrivilegeEscalation, InvalidScope, InvalidRole)
- `AppServiceError` -- Setup flow (AlreadySetup, ServerNameTooShort)
- `MetadataError` -- Model metadata operations (InvalidRepoFormat, ListAliasesFailed, AliasNotFound, ExtractionFailed, EnqueueFailed)
- `ModelError` -- Model listing (MetadataFetchFailed)
- `ToolsetValidationError` -- Toolset CRUD validation
- `SettingsError` -- Settings management (NotFound, BodhiHome, Unsupported)
- `CreateAliasError` -- Model alias creation
- `UserInfoError` -- User info endpoint (InvalidHeader, EmptyToken)

All enums use the `#[error_meta(trait_to_impl = AppError)]` pattern from `errmeta_derive`, mapping each variant to an `ErrorType` that determines the HTTP status code via the `ApiError` conversion in `objs`.

### Typed Axum Extractors from auth_middleware
The crate uses **typed Axum extractors** from `auth_middleware` instead of manual `HeaderMap` parsing for user identity. This eliminates a class of bugs where handlers could forget to check for missing headers or parse them incorrectly.

**Extractors used throughout the crate:**
- `ExtractUserId(user_id)` -- Extracts user ID from `X-BodhiApp-User-Id` header; returns `ApiError` if missing
- `ExtractToken(token)` -- Extracts auth token from `X-BodhiApp-Token` header; returns `ApiError` if missing
- `ExtractUsername(username)` -- Extracts username from `X-BodhiApp-Username` header
- `ExtractRole(role)` -- Extracts parsed `ResourceRole` from `X-BodhiApp-Role` header
- `MaybeRole(role)` -- Extracts `Option<ResourceRole>`, allowing handlers to check if user has any role

**Important nuance**: Some handlers that use typed extractors also accept `HeaderMap` as a parameter. This is intentional -- the `HeaderMap` is used for auxiliary functions like `is_oauth_auth()` that inspect scope and role headers for filtering logic, not for extracting user identity. The pattern is: extractors for identity, HeaderMap for auxiliary inspection.

**Migration note**: The helper functions `extract_user_id_from_headers` and `extract_token_from_headers` that previously existed in `routes_toolsets.rs` have been removed in favor of the typed extractors.

## Architecture Position

The `routes_app` crate sits in the **API layer** of BodhiApp's architecture:
- **Depends on**: `objs` (domain types, errors), `services` (business logic), `commands` (CLI orchestration), `auth_middleware` (extractors, session helpers), `server_core` (RouterState)
- **Consumed by**: `routes_all` (route composition), `server_app` (standalone server), `bodhi` (Tauri app)
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
Toolset routes use a dual-auth model:
- Session auth (role headers) grants full access to all toolset types
- OAuth auth (scope headers) restricts access to toolset types matching `scope_toolset-*` scopes in the token
- The `is_oauth_auth()` helper distinguishes these two auth modes by checking header presence

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

## Extension Patterns

### Adding New Application Endpoints
1. Define a domain-specific error enum with `#[error_meta(trait_to_impl = AppError)]` variants for every distinct failure mode
2. Use typed extractors (`ExtractUserId`, `ExtractToken`, etc.) for user identity -- only add `HeaderMap` if auxiliary header inspection is needed
3. Add `#[utoipa::path(...)]` annotations with comprehensive request/response examples and security requirements
4. Coordinate through `RouterState` for service access
5. For complex multi-service operations, consider delegating to the `commands` crate

### Adding New Error Variants
When extending existing error enums:
1. Add the variant with `#[error("...")]` message template and `#[error_meta(error_type = ErrorType::...)]`
2. The error code is auto-generated from the enum name and variant name (e.g., `LoginError::MissingState` -> `login_error-missing_state`)
3. Use `#[error(transparent)]` with `#[from]` for wrapping upstream service errors
4. Test the error code and HTTP status code in integration tests
