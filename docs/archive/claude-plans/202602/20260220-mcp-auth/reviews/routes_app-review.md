# routes_app Crate Review

## Files Reviewed
- `crates/routes_app/src/routes_mcp/auth_configs.rs` (337 lines) - Unified auth config CRUD handlers (create, list, get, delete) and OAuth flow handlers (login, token exchange)
- `crates/routes_app/src/routes_mcp/oauth_utils.rs` (232 lines) - OAuth discovery handlers (AS + MCP), standalone dynamic client registration, OAuth token get/delete handlers
- `crates/routes_app/src/routes_mcp/types.rs` (317 lines) - All request/response DTOs for MCP servers, instances, auth configs, OAuth flow, discovery, and dynamic registration
- `crates/routes_app/src/routes_mcp/error.rs` (9 lines) - McpValidationError enum
- `crates/routes_app/src/routes_mcp/mod.rs` (23 lines) - Module exports and endpoint constants
- `crates/routes_app/src/routes_mcp/servers.rs` (232 lines) - MCP server CRUD handlers with atomic auth config creation
- `crates/routes_app/src/routes.rs` (lines 158-247, user_session_apis) - Route wiring for all auth config, OAuth, and discovery endpoints

## Findings

### Finding 1: OAuth CSRF state parameter has no TTL
- **Priority**: Important
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `oauth_login_handler` (lines 158-166), `oauth_token_exchange_handler` (lines 226-255)
- **Issue**: The PKCE code_verifier and CSRF state are stored in the user's session under the key `mcp_oauth_{config_id}` with no time-to-live. The session data persists until either (a) the user completes the token exchange (which calls `session.remove` on line 255), or (b) the overall session expires (governed by session middleware configuration, not by the OAuth flow). There is no mechanism to expire the OAuth state independently of the session lifetime. This means a state parameter generated today could be used for a CSRF attack weeks later if the session is still alive and the user never completed the flow.
- **Recommendation**: Store a `created_at` timestamp alongside the `code_verifier` and `state` in the session data, and validate in `oauth_token_exchange_handler` that the state is no older than a reasonable TTL (e.g., 10 minutes). Reject with a clear error message if the state has expired.
- **Rationale**: OAuth 2.1 security best practices recommend short-lived authorization state to limit the window for CSRF and code injection attacks. The current implementation relies entirely on session expiry, which is typically configured for user convenience (hours/days) rather than security (minutes).

### Finding 2: oauth_token_exchange_handler creates a new reqwest::Client instead of using the shared one
- **Priority**: Important
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: Line 289
- **Issue**: The `oauth_token_exchange_handler` creates a new `reqwest::Client::new()` for the token exchange HTTP call. The `DefaultMcpService` already holds a shared `http_client: reqwest::Client` that is used for discovery, DCR, and token refresh operations. Creating a new client for each token exchange request means:
  1. No connection pooling with the shared client (the token endpoint is likely the same server used for refresh).
  2. A new TLS session is established for every token exchange.
  3. The handler bypasses any future shared client configuration (timeouts, proxy, custom CA certs).
- **Recommendation**: Make the shared `http_client` from `McpService` accessible via a method (e.g., `mcp_service.http_client()`) or move the token exchange logic into the `McpService` itself (which already handles the similar token refresh POST in `resolve_oauth_token`). The latter approach is preferable as it consolidates all OAuth HTTP calls behind the service abstraction.
- **Rationale**: The token exchange handler performs the same kind of HTTP POST to the token endpoint as the token refresh flow in `McpService`. Having two different HTTP clients for the same token endpoint is inconsistent and misses connection reuse benefits.

### Finding 3: redirect_uri is not validated against allowlist
- **Priority**: Important
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `oauth_login_handler` line 177, `oauth_token_exchange_handler` line 277
- **Issue**: The `redirect_uri` is accepted from the client request and passed through to the authorization URL (login) and token exchange (code exchange) without any validation or allowlisting. The `OAuthLoginRequest` (types.rs line 244) accepts an arbitrary `redirect_uri: String`. A malicious client could set `redirect_uri` to an attacker-controlled URL and, if the authorization server does not enforce strict redirect URI matching, intercept the authorization code. While the authorization server is ultimately responsible for redirect URI validation, the client application should also enforce known redirect URIs as defense-in-depth.
- **Recommendation**: Consider validating the `redirect_uri` against a known set of acceptable patterns (e.g., must start with the app's own origin, or match one of a set of pre-configured redirect URIs). At minimum, validate that the redirect_uri is a well-formed URL using `url::Url::parse()`.
- **Rationale**: OAuth 2.1 (Section 4.1.2.1) requires exact redirect URI matching. While the authorization server enforces this, the client should also validate to prevent misuse of its own login endpoint as an open redirector. If the authorization server has a lax redirect URI policy, the client becomes the last line of defense.

### Finding 4: list/get/delete auth config handlers do not extract AuthContext
- **Priority**: Nice-to-have
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `list_auth_configs_handler` (lines 60-67), `get_auth_config_handler` (lines 84-94), `delete_auth_config_handler` (lines 111-118)
- **Issue**: Three of the four auth config handlers do not extract `Extension<AuthContext>`. While all these endpoints are behind `api_auth_middleware(ResourceRole::User, None, None, ...)` which guarantees a valid session, the handlers themselves do not access the user identity. This means:
  - `list_auth_configs_handler` returns all auth configs for a server regardless of who created them (acceptable for shared admin resources).
  - `get_auth_config_handler` returns any auth config by ID regardless of the requester.
  - `delete_auth_config_handler` allows any authenticated user to delete any auth config by ID, without checking if the user is the creator or an admin.

  The `create_auth_config_handler` correctly extracts `user_id` and passes it to the service layer. The design doc states auth configs are "admin-managed resources," so the lack of per-user scoping on list/get may be intentional. However, the delete handler allowing any user to delete admin-managed configs is a concern.
- **Recommendation**: At minimum, add ownership or role checking in `delete_auth_config_handler` -- either verify the requester created the config or require an admin/manager role. The list/get handlers are acceptable as-is for shared resources. Document the intentional lack of ownership checks for list/get operations.
- **Rationale**: Auth configs are admin-managed shared resources, so broad read access is reasonable. However, allowing any authenticated user to delete shared configs contradicts the "admin-managed" design principle. The create handler records `created_by`, but delete does not check it.

### Finding 5: McpValidationError is a single-variant catch-all error enum
- **Priority**: Nice-to-have
- **File**: crates/routes_app/src/routes_mcp/error.rs
- **Location**: Entire file (9 lines)
- **Issue**: `McpValidationError` has a single variant `Validation(String)` that maps to `ErrorType::BadRequest`. All error conditions in the MCP OAuth handlers funnel through this one variant with free-form string messages. This produces a single error code `mcp_validation_error-validation` for all failure modes: session errors, URL parse errors, CSRF mismatches, token exchange failures, missing fields, etc. Consumers cannot distinguish between error conditions programmatically.
- **Recommendation**: Split into domain-specific variants following the pattern of other error enums in the crate (e.g., `LoginError`, `AccessRequestError`). Suggested variants:
  - `CsrfStateMismatch` - state validation failure (400)
  - `SessionDataMissing` - OAuth session not found (400)
  - `TokenExchangeFailed(String)` - upstream token endpoint error (502 or 400)
  - `InvalidUrl(String)` - URL parse error (400)
  - `ConfigNotOAuth` - operation requires OAuth config but found header config (400)
  This would produce distinct, machine-readable error codes like `mcp_validation_error-csrf_state_mismatch`.
- **Rationale**: The crate's CLAUDE.md explicitly describes the design philosophy of "domain-specific error enums" that produce "deterministic, machine-readable error codes." The current single-variant enum undermines this goal. Frontend error handling (e.g., showing a specific "please re-initiate login" message for session expiry vs. a generic "validation error") requires distinguishing between error types.

### Finding 6: DynamicRegisterResponse exposes client_secret and registration_access_token in plaintext
- **Priority**: Nice-to-have
- **File**: crates/routes_app/src/routes_mcp/types.rs
- **Location**: `DynamicRegisterResponse` (lines 306-317)
- **Issue**: The `DynamicRegisterResponse` includes `client_secret: Option<String>` and `registration_access_token: Option<String>` as plaintext strings in the API response. This is inconsistent with the rest of the auth config API, where secrets are masked with boolean flags (`has_client_secret`, `has_registration_access_token`). The `standalone_dynamic_register_handler` (oauth_utils.rs lines 152-165) passes these values directly from the DCR endpoint response to the API caller.
- **Recommendation**: This is likely intentional -- the DCR endpoint is a "discovery" operation where the frontend needs the actual credentials to create the auth config. The secrets only exist transiently in the HTTP response and are never stored by this handler (they would be stored later via `create_auth_config_handler` which encrypts them). Document this intentional difference from the CRUD response pattern to avoid confusion for future maintainers.
- **Rationale**: The DCR response is a passthrough from an external registration endpoint. The frontend needs the actual values to populate the "create OAuth config" form. This is functionally equivalent to the user typing in credentials manually. The risk is limited because the response is only sent to the authenticated user who initiated the registration.

### Finding 7: Discovery handlers accept empty-string URLs without URL format validation
- **Priority**: Nice-to-have
- **File**: crates/routes_app/src/routes_mcp/oauth_utils.rs
- **Location**: `oauth_discover_as_handler` (line 38), `oauth_discover_mcp_handler` (line 85), `standalone_dynamic_register_handler` (lines 134, 139)
- **Issue**: The discovery and DCR handlers validate that input strings are not empty (`if request.url.is_empty()`) but do not validate that they are well-formed URLs. An input like `"not-a-url"` will pass the empty check and be forwarded to the service layer, where it will fail with a confusing HTTP client error from `reqwest`. The `oauth_login_handler` (auth_configs.rs line 170-172) correctly validates URL format using `url::Url::parse()`.
- **Recommendation**: Add `url::Url::parse()` validation for `url`, `mcp_server_url`, and `registration_endpoint` fields in the discovery/DCR handlers, consistent with the validation already present in `oauth_login_handler`. Return a clear `McpValidationError::Validation("invalid URL format: ...")` on parse failure.
- **Rationale**: Fail-fast validation at the handler level provides better error messages than relying on downstream HTTP client failures. The inconsistency with `oauth_login_handler` (which does validate) suggests this was an oversight rather than a design choice.

### Finding 8: oauth_token_exchange_handler accesses db_service directly instead of going through mcp_service
- **Priority**: Nice-to-have
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: Lines 257-261
- **Issue**: The handler calls `state.app_service().db_service().get_decrypted_client_secret(&config_id)` directly, bypassing the `mcp_service` abstraction. All other auth config operations in this file go through `mcp_service`. This creates a layering violation where the route handler reaches past the service layer to the database layer for one specific operation. The same handler also calls `mcp_service.get_oauth_config()` and `mcp_service.store_oauth_token()`, making the direct `db_service` call inconsistent.
- **Recommendation**: Add a `get_decrypted_client_secret(config_id) -> Result<Option<(String, String)>>` method to `McpService` that delegates to `db_service`, or better yet, move the entire token exchange logic into `McpService` (see Finding 2). This would keep the route handler thin and maintain the service abstraction boundary.
- **Rationale**: Route handlers should coordinate through the service layer, not reach into the database layer directly. This pattern is followed by all other handlers in the file and is a core architectural principle described in the crate's CLAUDE.md.

### Finding 9: PKCE implementation follows OAuth 2.1 correctly
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `oauth_login_handler` lines 153-155, `oauth_token_exchange_handler` lines 237-278
- **Issue**: None. The PKCE implementation correctly:
  1. Generates a 43-character random `code_verifier` using `generate_random_string(43)` (line 153).
  2. Computes the S256 challenge as `BASE64URL(SHA256(code_verifier))` using `general_purpose::URL_SAFE_NO_PAD` (line 154-155).
  3. Sends `code_challenge` and `code_challenge_method=S256` in the authorization URL (lines 178-179).
  4. Stores `code_verifier` in the server-side session -- never exposed to the client (line 162).
  5. Sends `code_verifier` in the token exchange request (line 278).
  6. Removes session data after use to prevent replay (line 255).
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation that PKCE is implemented per RFC 7636 with S256, which is required by OAuth 2.1.

### Finding 10: CSRF state validation is correctly implemented
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `oauth_login_handler` line 156, `oauth_token_exchange_handler` lines 244-253
- **Issue**: None. The CSRF state parameter correctly:
  1. Generates a UUID v4 as the state value (line 156).
  2. Stores the state in the server-side session (line 164).
  3. Validates the state from the token exchange request matches the stored value (line 249).
  4. Returns a clear error on mismatch: "OAuth state mismatch (CSRF protection)" (line 251).
  5. One-time use: session data is removed after validation (line 255).
- **Recommendation**: No changes needed (aside from Adding TTL per Finding 1).
- **Rationale**: Positive confirmation of CSRF protection via the state parameter. The state is server-stored (not client-stored), preventing client-side tampering.

### Finding 11: Serde conventions correctly applied across all DTOs
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes_mcp/types.rs
- **Location**: Throughout (317 lines)
- **Issue**: None. All DTOs consistently apply:
  - `#[serde(default)]` on optional request fields.
  - `#[serde(skip_serializing_if = "Option::is_none")]` on optional response fields.
  - `#[serde(flatten)]` on `CreateAuthConfigBody.config` to merge discriminated union fields with `mcp_server_id`.
  - `utoipa::ToSchema` derived on all types for OpenAPI registration.
  - `From` impls for converting domain types to response DTOs (e.g., `McpOAuthToken` -> `OAuthTokenResponse`, `Mcp` -> `McpResponse`).
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation of adherence to the project's serde conventions documented in the crate's CLAUDE.md.

### Finding 12: Route wiring correctly places all OAuth endpoints behind session-only auth
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes.rs
- **Location**: Lines 158-247 (`user_session_apis`)
- **Issue**: None. All MCP auth config, OAuth login/token, discovery, DCR, and OAuth token endpoints are wired in the `user_session_apis` group, which uses `api_auth_middleware(ResourceRole::User, None, None, ...)`. The `None` values for `TokenScope` and `UserScope` mean these endpoints only accept session-based authentication (no API tokens, no external app OAuth). This is the correct security posture for:
  - Auth config CRUD (admin operations)
  - OAuth login/token exchange (requires session for PKCE state storage)
  - Discovery/DCR (utility operations initiated from the UI)
  - OAuth token management (user-specific resources)
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation that OAuth flow endpoints are not accessible via API tokens or external app OAuth, which would be a security concern since the PKCE flow relies on session state.

### Finding 13: OpenAPI registration is complete for all auth config and OAuth endpoints
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs, oauth_utils.rs, types.rs
- **Location**: `#[utoipa::path]` annotations on all handlers, `ToSchema` derives on all DTOs
- **Issue**: None. All 10 MCP auth-related handlers have complete `#[utoipa::path]` annotations with correct operation IDs, request body types, response types, parameter descriptions, and security requirements. All DTOs derive `ToSchema`. The endpoint paths are defined using the endpoint constants from `mod.rs`.
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation that the OpenAPI spec will include all auth config and OAuth endpoints, ensuring the TypeScript client generation and Swagger UI documentation are complete.

### Finding 14: resource parameter included in both authorization and token exchange
- **Priority**: Nice-to-have (positive finding)
- **File**: crates/routes_app/src/routes_mcp/auth_configs.rs
- **Location**: `oauth_login_handler` lines 185-192, `oauth_token_exchange_handler` lines 268-279
- **Issue**: None. Both the authorization URL and the token exchange request include the `resource` parameter set to the MCP server URL. This correctly implements RFC 8707 (Resource Indicators for OAuth 2.0), which allows the authorization server to scope tokens to a specific resource.
- **Recommendation**: No changes needed.
- **Rationale**: Positive confirmation of RFC 8707 compliance. Including the resource parameter in both steps ensures the access token is appropriately scoped to the MCP server.
