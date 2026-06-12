# Backend Tests Review

## Files Reviewed

- `crates/services/src/db/test_mcp_repository.rs` (572 lines) - DB repository CRUD tests for auth headers, OAuth configs, and OAuth tokens
- `crates/services/src/mcp_service/tests.rs` (1116 lines) - McpService business logic tests: auth types, discovery, DCR, tool fetch/execute with auth
- `crates/routes_app/src/routes_mcp/test_auth_configs.rs` (221 lines) - Route handler tests for unified auth config CRUD endpoints
- `crates/routes_app/src/routes_mcp/test_oauth_utils.rs` (365 lines) - Route handler tests for OAuth discovery, DCR, and token CRUD handlers
- `crates/routes_app/src/routes_mcp/test_mcps.rs` (766 lines) - Route handler tests for MCP instance CRUD + integration tests with real DB
- `crates/routes_app/src/routes_mcp/test_servers.rs` (363 lines) - Route handler tests for MCP server CRUD with atomic auth config creation
- `crates/server_app/tests/test_live_mcp.rs` (497 lines) - Live integration tests: full CRUD flow, tool execution, auth lifecycle
- `crates/services/src/test_utils/db.rs` (1066 lines) - TestDbService and MockDbService with full McpRepository delegation
- `crates/routes_app/src/routes_mcp/auth_configs.rs` (337 lines) - Production handler code for auth configs and OAuth login/token exchange (read for coverage analysis)
- `crates/routes_app/src/routes_mcp/oauth_utils.rs` (232 lines) - Production handler code for discovery, DCR, token CRUD (read for coverage analysis)

## Test Coverage Summary

| Critical Path | Tested? | Test Location | Notes |
|---|---|---|---|
| MCP metadata discovery (success) | Yes | `mcp_service/tests.rs:955` `test_mcp_service_discover_mcp_oauth_metadata_success` | Two mockito servers simulate PRS + AS endpoints |
| MCP metadata discovery (failure) | Yes | `mcp_service/tests.rs:1017` `test_mcp_service_discover_mcp_oauth_metadata_prs_404` | PRS 404 case covered |
| AS metadata discovery (success) | Yes | `mcp_service/tests.rs:893` `test_mcp_service_discover_oauth_metadata_success` | RFC 8414 well-known endpoint |
| AS metadata discovery (failure) | Yes | `mcp_service/tests.rs:928` `test_mcp_service_discover_oauth_metadata_not_found` | 404 case covered |
| DCR success | Yes | `mcp_service/tests.rs:1044` `test_mcp_service_dynamic_register_client_success` | Full response field validation |
| DCR failure | Yes | `mcp_service/tests.rs:1088` `test_mcp_service_dynamic_register_client_failure` | 400 error case |
| Auth code flow with state validation | **NO** | -- | `oauth_login_handler` and `oauth_token_exchange_handler` have ZERO unit tests |
| Invalid state (CSRF prevention) | **NO** | -- | State mismatch path (line 249 of auth_configs.rs) is untested |
| Token exchange success (with PKCE) | **NO** | -- | No test exercises the full PKCE code_verifier -> code_challenge -> token exchange flow |
| Token exchange failures | **NO** | -- | No test for token exchange HTTP failure, invalid response, missing access_token |
| Token refresh proactive | **NO** | -- | `resolve_oauth_token` (service.rs:428-563) has zero tests; proactive refresh (60s before expiry) is untested |
| Token refresh reactive | **NO** | -- | No reactive refresh path exists in code (proactive only with 60s window), but the refresh failure path is untested |
| Auth config CRUD (header) | Yes | `test_auth_configs.rs:50,122,164,188` | Create, get, delete, list for header type |
| Auth config CRUD (OAuth pre-reg) | Partial | `test_servers.rs:240` | Only tested via atomic server+config creation, not standalone `POST /mcps/auth-configs` |
| Auth config CRUD (OAuth DCR) | **NO** | -- | No test creates an OAuth config with `registration_type: "dynamic-registration"` |
| Ownership check enforcement | Partial | `test_oauth_utils.rs:319` `test_get_oauth_token_handler_wrong_user_returns_404` | OAuth token ownership tested; auth header/config ownership not tested at route level |
| Encryption roundtrip | Yes | `test_mcp_repository.rs:48` `test_db_service_get_decrypted_auth_header_roundtrip` and `test_mcp_repository.rs:423` `test_init_service_get_decrypted_client_secret` | Header and OAuth client secret encryption roundtrip verified |
| OAuth token encryption roundtrip | Partial | `test_mcp_repository.rs:460` | Verifies stored != plaintext, but no explicit decrypt roundtrip assertion for tokens |
| Secrets never exposed in API | Yes | `test_live_mcp.rs:401-411` | Live test asserts `auth_header_value` and `encrypted_auth_header_value` absent from response |

## Findings

### Finding 1: OAuth login and token exchange handlers have zero test coverage

- **Priority**: Critical
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs`
- **Location**: `oauth_login_handler` (line 140) and `oauth_token_exchange_handler` (line 216)
- **Issue**: These two handlers implement the core OAuth 2.1 auth code flow with PKCE S256 and CSRF state validation. Neither has any unit or integration test. The `oauth_login_handler` builds the authorization URL with PKCE code_challenge, stores state+code_verifier in session, and appends the `resource` parameter. The `oauth_token_exchange_handler` validates CSRF state, retrieves decrypted client credentials, sends the token exchange HTTP request with code_verifier, and stores the resulting tokens. None of these paths are tested.
- **Recommendation**: Add tests for:
  1. `oauth_login_handler` success: verify authorization URL contains correct query params (response_type=code, client_id, code_challenge, code_challenge_method=S256, state, scope, resource)
  2. `oauth_login_handler` with missing config (404)
  3. `oauth_token_exchange_handler` success: mock session with state+code_verifier, mock HTTP token endpoint, verify token stored
  4. `oauth_token_exchange_handler` state mismatch (CSRF rejection)
  5. `oauth_token_exchange_handler` missing session data (login not initiated)
  6. `oauth_token_exchange_handler` HTTP failure from token endpoint
- **Rationale**: This is the highest-risk untested code path. CSRF protection, PKCE verification, and token storage are security-critical. A regression in state validation would silently break CSRF protection.

### Finding 2: Token refresh logic (`resolve_oauth_token`) has zero test coverage

- **Priority**: Critical
- **File**: `crates/services/src/mcp_service/service.rs`
- **Location**: `resolve_oauth_token` method (lines 428-563)
- **Issue**: The proactive token refresh logic (triggers 60 seconds before expiry) is untested. This method handles: mutex-based concurrency guard, expiry check with 60s window, encrypted refresh token decryption, client credential lookup, HTTP refresh request, response parsing, new token encryption, and DB update. The existing `test_mcp_service_execute_with_oauth_auth_type` test uses a non-expired token, so it only exercises the happy path of decrypt-and-return (lines 552-562).
- **Recommendation**: Add service-level tests for:
  1. Token not expired: returns decrypted access token (this path IS tested indirectly)
  2. Token expired + refresh token present: mock HTTP refresh endpoint, verify new tokens stored
  3. Token expired + no refresh token: verify `OAuthTokenExpired` error
  4. Token expired + refresh HTTP failure: verify `OAuthRefreshFailed` error
  5. Concurrency: verify mutex prevents duplicate refresh
- **Rationale**: Token refresh is invoked on every tool fetch/execute with OAuth auth. A regression would silently break all OAuth-protected MCP tool usage once tokens expire.

### Finding 3: Duplicate router setup functions across route test files

- **Priority**: Important
- **File**: `crates/routes_app/src/routes_mcp/test_auth_configs.rs`, `test_oauth_utils.rs`, `test_mcps.rs`, `test_servers.rs`
- **Location**: `test_router_for_auth_configs` (line 22), `test_router_for_oauth_discovery` (line 25), `test_router_for_oauth_tokens` (line 251), `test_router_for_crud` (line 53), `test_router_for_mcp_servers` (line 38)
- **Issue**: Five nearly-identical async functions each construct a `Router` with `MockMcpService` -> `AppServiceStubBuilder` -> `DefaultRouterState`. They differ only in which routes are mounted. This is 15-20 lines of boilerplate repeated 5 times.
- **Recommendation**: Extract a shared helper function `build_mcp_test_router(mock: MockMcpService, routes: impl FnOnce(Router) -> Router) -> Router` into a test-utils module within `routes_mcp/`. Each test file would then supply only the route registration closure.
- **Rationale**: Reduces maintenance burden and ensures consistent test router construction. If `DefaultRouterState` construction changes, only one location needs updating.

### Finding 4: `Utc::now()` used directly in route test fixtures

- **Priority**: Important
- **File**: `crates/routes_app/src/routes_mcp/test_auth_configs.rs`, `test_mcps.rs`, `test_servers.rs`, `test_oauth_utils.rs`
- **Location**: Lines 69-70, 137-138, 203-204 in `test_auth_configs.rs`; line 36 in `test_mcps.rs`; lines 24, 186-187, 268-269 in `test_servers.rs`; line 237 in `test_oauth_utils.rs`
- **Issue**: Multiple test fixture functions use `Utc::now()` directly instead of a deterministic time source. The project convention (documented in CLAUDE.md and services/CLAUDE.md) mandates `TimeService` for testability. While these are mock return values (not timestamps passed to production code), the inconsistency could mask time-dependent bugs if tests are extended to verify timestamp ordering or expiry logic.
- **Recommendation**: Use a fixed constant like `DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")` or import `FrozenTimeService::default().utc_now()` for deterministic timestamps in mock return values.
- **Rationale**: Follows project convention. Prevents potential flakiness if snapshot tests or ordering assertions are added later.

### Finding 5: No test for creating an OAuth auth config via standalone `POST /mcps/auth-configs`

- **Priority**: Important
- **File**: `crates/routes_app/src/routes_mcp/test_auth_configs.rs`
- **Location**: Only `test_create_auth_config_header_success` exists (line 50)
- **Issue**: The `CreateMcpAuthConfigRequest` is a discriminated union supporting `Header` and `Oauth` variants. Only the `Header` variant is tested via the standalone auth-configs endpoint. The `Oauth` variant (both pre-registered and dynamic-registration subtypes) is only tested indirectly through the atomic server creation path in `test_servers.rs:240` (`test_create_mcp_server_with_oauth_prereg_auth_config`). There is no test for creating an OAuth auth config via `POST /mcps/auth-configs` directly.
- **Recommendation**: Add `test_create_auth_config_oauth_prereg_success` and `test_create_auth_config_oauth_dcr_success` tests in `test_auth_configs.rs` that exercise the `Oauth` variant of the discriminated union body.
- **Rationale**: The standalone `POST /mcps/auth-configs` endpoint is the primary way the frontend creates OAuth configs. Testing only the atomic server+config path leaves the standalone path unverified for OAuth type.

### Finding 6: Missing `update_mcp_oauth_token` and `get_decrypted_oauth_bearer` DB tests

- **Priority**: Important
- **File**: `crates/services/src/db/test_mcp_repository.rs`
- **Location**: End of file (line 572)
- **Issue**: The `McpRepository` trait includes `update_mcp_oauth_token` and `get_decrypted_oauth_bearer` methods. Neither has a dedicated DB-level test. The `update_mcp_oauth_token` is called by `resolve_oauth_token` during token refresh. The `get_decrypted_oauth_bearer` is used by `fetch_tools_for_server` when an `auth_uuid` is provided. The `TestDbService` delegates to these correctly (lines 776-812 of db.rs), but there are no tests verifying the SQL and encryption roundtrip.
- **Recommendation**: Add:
  1. `test_init_service_update_mcp_oauth_token`: create token, update with new encrypted values, verify decrypted values match
  2. `test_init_service_get_decrypted_oauth_bearer`: create config + token, call `get_decrypted_oauth_bearer`, verify returns `("Authorization", "Bearer <plaintext>")` tuple
- **Rationale**: These are on the critical path for token refresh and OAuth tool execution. DB-level roundtrip tests catch SQL/encryption bugs that mock-based tests cannot.

### Finding 7: Repetitive McpServerRow/McpRow construction in services tests

- **Priority**: Nice-to-have
- **File**: `crates/services/src/mcp_service/tests.rs`
- **Location**: `setup_server` (line 13), and repeated `service.create(...)` calls with 10 positional parameters throughout the file
- **Issue**: The `McpService::create` method takes 10 positional parameters. Every test that creates an MCP instance repeats a 15-line call with mostly-identical arguments. Similarly, `create_oauth_config` takes 13 positional parameters. This makes tests verbose and hard to read.
- **Recommendation**: Create builder-pattern test helpers: `TestMcpBuilder::new("user-1", "server-1").name("My MCP").auth_type(McpAuthType::Header).build(&service)`. Similarly for `TestOAuthConfigBuilder`. These could live in `services/src/test_utils/mcp.rs`.
- **Rationale**: Reduces test verbosity from 15 lines to 3-4 lines per MCP creation, improves readability, and makes it clearer which parameters are being varied in each test case.

### Finding 8: `delete_oauth_token_handler` test does not actually verify deletion

- **Priority**: Important
- **File**: `crates/routes_app/src/routes_mcp/test_oauth_utils.rs`
- **Location**: `test_delete_oauth_token_handler_success` (line 348)
- **Issue**: The test creates a `MockMcpService::new()` with no expectations configured, sends a DELETE request, and asserts `StatusCode::NO_CONTENT`. Since the handler calls `db_service.delete_mcp_oauth_token(user_id, &token_id)` (not `mcp_service`), and the test's `AppServiceStubBuilder` creates a default stub DB service, the test does not verify that the delete method was actually called. The handler would return 204 even if the delete call silently failed (it maps errors to `McpValidationError`).
- **Recommendation**: Either:
  (a) Use a `MockDbService` with `.expect_delete_mcp_oauth_token()` to verify the call, or
  (b) Use the integration test pattern (like `test_mcps.rs` integration tests) with a real DB to verify the token is actually gone after deletion.
- **Rationale**: The test gives false confidence. A regression that removes the delete call from the handler would not be caught.

### Finding 9: No rstest `#[case::]` or `#[values]` usage in new tests

- **Priority**: Nice-to-have
- **File**: All test files reviewed
- **Location**: Throughout
- **Issue**: The new MCP OAuth test files use individual test functions for each scenario rather than parameterized tests. For example, `test_mcp_service_discover_oauth_metadata_success` and `test_mcp_service_discover_oauth_metadata_not_found` could be a single `#[rstest]` with `#[case::success(200, true)]` and `#[case::not_found(404, false)]`. Similarly, auth type switching tests (public->header, header->public) could use `#[values(McpAuthType::Header, McpAuthType::Oauth)]`.
- **Recommendation**: Consider parameterizing:
  - Discovery success/failure cases with `#[case::]`
  - Auth type switching tests with `#[values]` for auth type combinations
  - CRUD happy path tests with `#[case::]` for different auth config types
- **Rationale**: Reduces test function count while maintaining coverage. Makes it easier to add new test cases (e.g., when a new auth type is added).

### Finding 10: No test coverage for the `resource` parameter in OAuth authorization URL

- **Priority**: Important
- **File**: `crates/routes_app/src/routes_mcp/auth_configs.rs`
- **Location**: Lines 185-192 in `oauth_login_handler`
- **Issue**: The handler appends a `resource` parameter to the authorization URL by looking up the MCP server URL from the config's `mcp_server_id`. This is a key OAuth 2.1 security requirement (the `resource` parameter ensures the token is audience-restricted). Since `oauth_login_handler` has no tests at all (Finding 1), this specific behavior is also untested.
- **Recommendation**: When adding tests per Finding 1, specifically assert that the returned `authorization_url` contains `resource=<mcp_server_url>` as a query parameter.
- **Rationale**: The `resource` parameter is required by RFC 9728 for MCP OAuth flows. If the server URL lookup fails silently or the parameter is dropped, tokens would be minted without audience restriction.

### Finding 11: Test data factory functions are file-local rather than shared

- **Priority**: Nice-to-have
- **File**: `crates/services/src/db/test_mcp_repository.rs` and `crates/services/src/mcp_service/tests.rs`
- **Location**: `test_mcp_server_row()` (test_mcp_repository.rs:12), `setup_server()` (tests.rs:13), `test_oauth_config_row()` (test_mcp_repository.rs:238), `test_oauth_token_row()` (test_mcp_repository.rs:265)
- **Issue**: Both files define their own `McpServerRow` factory functions with slightly different signatures. `test_mcp_repository.rs` has `test_mcp_server_row(now: i64)` while `tests.rs` has `setup_server(db: &dyn DbService)`. Similarly, `test_oauth_config_row` and `test_oauth_token_row` are only in the repository test file but would be useful for service-level tests.
- **Recommendation**: Extract shared factory functions into `crates/services/src/test_utils/mcp.rs` (new file):
  - `pub fn test_mcp_server_row(now: i64) -> McpServerRow`
  - `pub fn test_oauth_config_row(encryption_key: &[u8], now: i64) -> McpOAuthConfigRow`
  - `pub fn test_oauth_token_row(encryption_key: &[u8], config_id: &str, now: i64) -> McpOAuthTokenRow`
  - `pub async fn setup_server(db: &dyn DbService) -> McpServerRow`
- **Rationale**: Eliminates duplication and ensures consistent test data across DB and service-level tests. When adding tests for `resolve_oauth_token` (Finding 2), these factories would be needed.

## Summary

The test suite provides solid coverage of the DB persistence layer and discovery/DCR service operations. The main gap is the complete absence of tests for the OAuth authorization code flow (login + token exchange handlers) and the token refresh mechanism. These are the two most security-critical and complex code paths in the feature. The route-level tests for auth configs only cover the Header variant, leaving the OAuth variant partially tested.

**Priority breakdown:**
- **Critical (2)**: OAuth login/token exchange handlers (Finding 1), token refresh logic (Finding 2)
- **Important (5)**: Duplicate router setup (Finding 3), `Utc::now()` in tests (Finding 4), missing OAuth auth config test (Finding 5), missing DB tests (Finding 6), delete handler verification (Finding 8), resource parameter (Finding 10)
- **Nice-to-have (3)**: Builder pattern for test helpers (Finding 7), rstest parameterization (Finding 9), shared test factories (Finding 11)
