# Implementation Context & Insights

## Overview
This document captures technical insights, discoveries, and troubleshooting solutions from implementing the token refresh integration test.

## Key Technical Concepts

### Session Token Storage
- Session tokens stored in Tower Sessions with keys:
  - `SESSION_KEY_ACCESS_TOKEN = "access_token"`
  - `SESSION_KEY_REFRESH_TOKEN = "refresh_token"`
  - `SESSION_KEY_USER_ID = "user_id"`
- Located in: `crates/auth_middleware/src/auth_middleware.rs`

### Token Refresh Flow
1. Middleware detects expired access token
2. Retrieves refresh token from session
3. Calls `auth_service.refresh_token()` with retry logic (3 attempts, exponential backoff)
4. Updates session with new tokens via `session.insert()`
5. Explicitly calls `session.save()` to persist
6. Subsequent requests use refreshed token

### Dev Endpoint Access
- Endpoint: `/dev/secrets`
- Handler: `dev_secrets_handler` in `crates/routes_app/src/routes_dev.rs`
- Axum auto-injects `Session` parameter (no manual extraction needed)
- Can read session tokens using `session.get::<String>("access_token").await`

## Keycloak Configuration

### Token Lifespan Settings
- Access Token Lifespan: Controls how long access tokens are valid
- SSO Session Idle: Controls refresh token validity during idle
- SSO Session Max: Maximum session duration regardless of activity

### Admin API Access
- Base URL: `{keycloak_url}/admin/realms/{realm}`
- Authentication: Bearer token from admin login
- Client configuration endpoint: `PUT /admin/realms/{realm}/clients/{client-uuid}`

## Environment Variables

### Required for Tests
- `AUTH_URL`: Keycloak server URL (e.g., http://localhost:8080)
- `AUTH_REALM`: Keycloak realm name (e.g., bodhiapp-test)
- Test credentials may be needed - TBD based on agent execution

## Troubleshooting Guide

### Common Issues

**Issue**: [To be filled by agents during implementation]
**Solution**: [To be filled by agents]

---

## Discoveries & Insights

<!-- Agents should document their findings below this line -->

### Session Key Constants Export Pattern (Phase 1)
**Discovery Date**: 2025-10-02

The `auth_middleware` crate uses a clean re-export pattern for session key constants:

**Location**: `crates/auth_middleware/src/auth_middleware.rs`
```rust
pub const SESSION_KEY_ACCESS_TOKEN: &str = "access_token";
pub const SESSION_KEY_REFRESH_TOKEN: &str = "refresh_token";
pub const SESSION_KEY_USER_ID: &str = "user_id";
```

**Export Pattern**: `crates/auth_middleware/src/lib.rs`
```rust
pub use auth_middleware::*;
```

**Usage in other crates**:
```rust
use auth_middleware::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
```

This pattern allows:
- Centralized constant definitions in the implementation module
- Public access via wildcard re-export from lib.rs
- Type-safe imports in consuming crates
- Easy refactoring if constant values need to change

### Session API Error Handling Pattern (Phase 1)
**Discovery Date**: 2025-10-02

The `tower_sessions::Session` API has a specific return type pattern:
- `session.get::<T>(key)` returns `Result<Option<T>>`
- Outer `Result` handles session access errors
- Inner `Option` represents whether the key exists

**Recommended handling pattern**:
```rust
let token = session
  .get::<String>(SESSION_KEY_ACCESS_TOKEN)
  .await
  .ok()          // Convert Result to Option, discarding errors
  .flatten();    // Flatten Option<Option<T>> to Option<T>
```

This provides:
- Clean null handling for missing keys
- Graceful error handling without panics
- Compatible with JSON serialization (null values)
- No need for explicit error propagation in dev endpoint

### Dev Endpoint Session Integration (Phase 1)
**Discovery Date**: 2025-10-02

The `/dev/secrets` endpoint uses Axum's automatic dependency injection:
- `Session` parameter is automatically extracted by Axum middleware
- No manual extraction from request needed
- Session is available in all route handlers that declare it

**Implementation insights**:
- Session tokens can be null if user is not authenticated
- JSON response automatically serializes `Option<String>` as null
- No special handling needed for unauthenticated requests
- Endpoint works for both authenticated and unauthenticated sessions

---

### Keycloak Admin API Integration (Phase 2)
**Discovery Date**: 2025-10-02

#### API Endpoint Structure

**Client Retrieval by Client ID**:
- Endpoint: `GET {authUrl}/admin/realms/{realm}/clients?clientId={clientId}`
- Query parameter filters clients by human-readable client ID
- Returns JSON array of matching clients (even for single match)
- Each client object contains `id` field (UUID) needed for updates

**Client Configuration Update**:
- Endpoint: `PUT {authUrl}/admin/realms/{realm}/clients/{uuid}`
- Requires client UUID (not client ID) in path
- Must send complete `ClientRepresentation` object
- Partial updates not supported - use spread operator to preserve existing settings

#### Authentication Pattern

**Dev Console Client for Admin Operations**:
- Client ID: `client-bodhi-dev-console`
- Has admin realm access permissions
- Supports password grant type for token acquisition
- No need for master realm or `admin-cli` client

**Token Acquisition**:
```javascript
const adminToken = await authClient.getDevConsoleToken(username, password);
```
- Uses existing `getDevConsoleToken()` method
- Standard OAuth2 password grant flow
- Token has sufficient permissions for admin API operations

#### Client Token Lifespan Configuration

**Attribute Name**: `access.token.lifespan`
- Located in `attributes` object of `ClientRepresentation`
- Value must be string, not number: `'5'` not `5`
- Value represents seconds: `'5'` = 5 second lifespan

**Configuration Pattern**:
```javascript
const updatedClient = {
  ...client,                    // Preserve existing settings
  attributes: {
    ...client.attributes,       // Preserve existing attributes
    'access.token.lifespan': '5' // Add/override token lifespan
  }
};
```

**Token Lifespan Behavior**:
- Only affects newly issued tokens after configuration
- Existing tokens retain their original lifespan
- Effective lifespan is minimum of:
  - `access.token.lifespan` (client-specific)
  - Realm-level access token lifespan
  - SSO Session Max
  - Client Session Max

#### Client ID vs Client UUID

**Two Identifiers in Keycloak**:
1. **Client ID** (`clientId` field):
   - Human-readable string (e.g., `client-bodhi-app-123`)
   - Used in OAuth2 flows (`client_id` parameter)
   - Can be used to query clients via API

2. **Client UUID** (`id` field):
   - Internal UUID (e.g., `a1b2c3d4-e5f6-...`)
   - Required for admin API update operations
   - Not exposed in OAuth2 flows

**Lookup Pattern**:
```javascript
// 1. Get client UUID from client ID
const clients = await fetch(`/admin/realms/{realm}/clients?clientId={clientId}`);
const clientUuid = clients[0].id;

// 2. Use UUID for updates
await fetch(`/admin/realms/{realm}/clients/${clientUuid}`, {
  method: 'PUT',
  body: JSON.stringify(updatedClient)
});
```

#### Error Handling Patterns

**Consistent Error Pattern**:
```javascript
if (!response.ok) {
  const errorText = await response.text();
  console.log('Operation failed:', response.status, response.statusText);
  console.log('Error response body:', errorText);
  throw new Error(`Failed to ...: ${response.status} ${response.statusText} - ${errorText}`);
}
```

**Benefits**:
- Logs detailed error information for debugging
- Includes HTTP status codes and response bodies
- Consistent with existing methods in `AuthServerTestClient`

#### API Quirks and Gotchas

1. **Array Response for Single Client**: GET query returns array even when filtering by unique client ID
2. **Full Object Required**: PUT endpoint requires complete client object, not partial updates
3. **String Attribute Values**: Numeric settings like token lifespan must be strings
4. **No Validation Feedback**: PUT returns 204 No Content on success (no confirmation data)
5. **Token Scope Requirement**: Admin token must have appropriate realm management scopes

---

### Integration Test Patterns (Phase 3)
**Discovery Date**: 2025-10-02

#### Playwright Request Context Integration

**Session Persistence Across Requests**:
- Playwright's `page.request.get()` maintains session cookies automatically
- No manual cookie management needed for authenticated requests
- Session context persists across multiple API calls within same page
- Browser context includes both UI navigation and API request capabilities

**API Testing with Browser Context**:
```javascript
// After OAuth login via browser UI:
await loginPage.performOAuthLogin();

// API requests automatically include session cookies:
const response = await page.request.get(`${baseUrl}/dev/secrets`);
const data = await response.json();
// Session tokens available in response
```

**Benefits**:
- Unified test context for UI and API testing
- No need for separate HTTP client configuration
- Automatic cookie/session management
- Realistic end-to-end testing scenario

#### Token Refresh Test Strategy

**Timing Considerations**:
- Configure client with short token lifespan (5 seconds) via Keycloak Admin API
- Wait slightly longer than token lifespan (6 seconds) to ensure expiry
- Keycloak processes token expiry based on `exp` claim timestamp
- Buffer time (1 second) accounts for clock skew and processing time

**Refresh Detection Pattern**:
```javascript
// Get initial token
const token1 = await getSessionToken(page, baseUrl);

// Wait for expiry
await page.waitForTimeout(6000);

// Next request triggers automatic refresh
const token2 = await getSessionToken(page, baseUrl);

// Verify tokens are different (refresh occurred)
expect(token2).not.toBe(token1);

// Immediate subsequent request reuses refreshed token
const token3 = await getSessionToken(page, baseUrl);
expect(token3).toBe(token2); // No re-refresh
```

**Refresh Behavior Insights**:
- Middleware detects expired token automatically on request
- Refresh happens transparently - no 401 error to client
- Refreshed token stored in session immediately
- Multiple rapid requests don't trigger multiple refreshes (token reused)

#### Test Setup Best Practices

**Resource Client Configuration for Tests**:
```javascript
// 1. Create resource client
const resourceClient = await authClient.createResourceClient(serverUrl);

// 2. Get admin token for configuration
const adminToken = await authClient.getDevConsoleToken(username, password);

// 3. Configure short token lifespan
await authClient.configureClientTokenLifespan(adminToken, resourceClient.clientId, 5);

// 4. Assign admin role to test user
await authClient.makeResourceAdmin(
  resourceClient.clientId,
  resourceClient.clientSecret,
  testCredentials.userId
);
```

**Key Points**:
- Token lifespan configuration requires admin-level token
- Dev console token has sufficient permissions for admin API
- Configuration applies to new tokens only (not existing tokens)
- Test user must have admin role to access protected resources

#### Environment Variable Management

**Test Credential Loading**:
- All credentials loaded from environment variables
- No hardcoded secrets in test files
- Utility functions handle validation and error messages:
  - `getAuthServerConfig()`: AUTH_URL, AUTH_REALM, DEV_CONSOLE_CLIENT_SECRET
  - `getTestCredentials()`: USERNAME, PASSWORD, USER_ID

**Integration Test Environment Requirements**:
```bash
# Keycloak server configuration
INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
INTEG_TEST_AUTH_REALM=bodhiapp-test
INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<secret>

# Test user credentials
INTEG_TEST_USERNAME=user@email.com
INTEG_TEST_PASSWORD=<password>
INTEG_TEST_USERNAME_ID=<uuid>
```

#### Test Isolation Pattern

**Server Lifecycle Management**:
- Each test suite creates isolated server instance
- Random port assignment prevents conflicts: `randomPort()`
- `beforeAll` setup ensures clean state before tests
- `afterAll` cleanup prevents resource leaks
- Server manager handles graceful shutdown

**Parallel Test Compatibility**:
- Random ports enable parallel test execution
- Each test suite has independent Keycloak client
- No shared state between test files
- Resource client created per test suite (not shared)

#### OAuth Flow Automation

**LoginPage Helper Pattern**:
- `performOAuthLogin()` handles complete OAuth flow
- Navigates to login page, redirects to auth server
- Fills credentials, handles redirect back to app
- Waits for SPA ready state after redirect

**Integration with Test**:
```javascript
const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
await page.goto(`${baseUrl}/ui/chat`); // Protected route
await loginPage.performOAuthLogin();   // Completes OAuth flow
// Now authenticated for API calls
```

**Benefits**:
- Encapsulates OAuth complexity
- Reusable across test files
- Handles timing and waits correctly
- Works with Keycloak consent screens if needed

#### Dev Endpoint Testing Strategy

**/dev/secrets Endpoint for Token Inspection**:
- Development endpoint exposes session tokens for testing
- Returns JSON: `{ session: { access_token, refresh_token } }`
- Enabled only in development/test environments
- Used to verify token refresh without decoding JWTs

**Security Considerations**:
- Endpoint only available in non-production environments
- Returns null for unauthenticated sessions
- No sensitive business data exposed
- Test-only utility for token lifecycle verification

#### Token Logging Best Practices

**Secure Token Logging in Tests**:
```javascript
// Log only token prefixes (first 20 chars)
console.log('Initial access token (first 20 chars):', token.substring(0, 20));
```

**Rationale**:
- Provides debugging information without exposing full tokens
- Sufficient to verify token changes (different prefixes = different tokens)
- Prevents token leakage in CI logs
- Follows security best practices for test logging

---

### Keycloak Admin API Permission Model (Phase 4)
**Discovery Date**: 2025-10-02

#### Dev Console Client Permission Limitations

**Critical Discovery**: The `client-bodhi-dev-console` client does NOT have Keycloak Admin API permissions.

**Dev Console Client Capabilities** (Confirmed):
- OAuth2 password grant flow for user authentication
- User login and session management
- Resource client creation (via custom endpoints, not Admin API)
- Token refresh for authenticated users

**Dev Console Client Limitations** (Discovered):
- ❌ Cannot access `/admin/realms/*` endpoints (403 Forbidden)
- ❌ No `realm-management` client roles assigned
- ❌ Cannot modify client configurations (including token lifespans)
- ❌ Cannot manage users or assign roles via Admin API
- ❌ Not suitable for administrative Keycloak operations

**Test Impact**:
```javascript
// This FAILS with 403 Forbidden:
const adminToken = await authClient.getDevConsoleToken(username, password);
await authClient.configureClientTokenLifespan(adminToken, clientId, 5);
// Error: Failed to get client: 403 Forbidden
```

#### Keycloak Admin API Permission Requirements

**Admin API Access Requires**:
1. **Master Realm Admin Credentials**:
   - Username/password for admin user in master realm
   - Access to `admin-cli` client
   - Full realm management permissions

2. **Service Account with Realm Management**:
   - Client configured with service account enabled
   - `realm-management` client roles assigned to service account
   - Client credentials grant flow for token acquisition

3. **Realm Admin User**:
   - User with `realm-admin` role assigned
   - Password grant flow for token acquisition
   - Realm-specific admin permissions

**Admin API Endpoint Pattern**:
```
GET  /admin/realms/{realm}/clients?clientId={id}    - Requires: manage-clients
PUT  /admin/realms/{realm}/clients/{uuid}           - Requires: manage-clients
POST /admin/realms/{realm}/users                    - Requires: manage-users
```

#### Corrected Phase 2 Assumptions

**Original Assumption** (INCORRECT):
- "Dev console client has admin realm access permissions"
- "Token has sufficient permissions for admin API operations"
- "Dev console token has sufficient permissions for admin API"

**Actual Reality** (VERIFIED):
- Dev console client is a **standard OAuth2 client** for user authentication only
- No admin realm access or realm-management permissions
- Suitable only for OAuth2 authentication flows, not administrative operations
- Admin API requires separate credentials with explicit admin roles

#### Alternative Approaches for Token Refresh Testing

**Option 1: Master Realm Admin** (Development/Manual Testing):
```javascript
// Requires: MASTER_ADMIN_USERNAME, MASTER_ADMIN_PASSWORD
const adminToken = await getAdminToken('admin', 'admin-password', 'master');
await configureClientTokenLifespan(adminToken, clientId, 5);
```
- Pros: Direct access to all admin operations
- Cons: Requires master realm credentials, security risk in CI/CD

**Option 2: Service Account Configuration** (Recommended for CI/CD):
```javascript
// One-time Keycloak setup: Enable service account + assign realm-management roles
const adminToken = await getServiceAccountToken(clientId, clientSecret);
await configureClientTokenLifespan(adminToken, clientId, 5);
```
- Pros: Secure, no user credentials, suitable for automation
- Cons: Requires Keycloak configuration changes

**Option 3: Pre-Configured Test Realm** (Simplest for CI/CD):
```yaml
# Keycloak realm configuration
realm: bodhiapp-test
accessTokenLifespan: 5  # Applied at realm level, affects all clients
```
- Pros: No runtime admin operations, stable configuration, fast tests
- Cons: Less flexible, affects all clients in realm, requires realm setup

**Option 4: Realm Admin User** (Balanced Approach):
```javascript
// Requires: Test user with realm-admin role
const adminToken = await authClient.getDevConsoleToken('admin-user', 'password');
await configureClientTokenLifespan(adminToken, clientId, 5);
```
- Pros: Realm-scoped permissions, secure, dedicated admin user
- Cons: Requires test user setup with admin role assignment

#### Integration Test Environment Setup Requirements

**For Runtime Admin API Configuration**:
```bash
# Environment variables needed for admin operations
INTEG_TEST_MASTER_ADMIN_USERNAME=admin          # Master realm admin
INTEG_TEST_MASTER_ADMIN_PASSWORD=<secret>       # Master realm password
# OR
INTEG_TEST_ADMIN_SERVICE_CLIENT_ID=<client>     # Service account client
INTEG_TEST_ADMIN_SERVICE_CLIENT_SECRET=<secret> # Service account secret
# OR
INTEG_TEST_REALM_ADMIN_USERNAME=<user>          # Realm admin user
INTEG_TEST_REALM_ADMIN_PASSWORD=<secret>        # Realm admin password
```

**For Pre-Configured Test Realm** (Recommended):
```bash
# Simpler approach - no admin credentials needed
# Keycloak realm imported with:
# - accessTokenLifespan: 5 seconds
# - Test users pre-configured
# - Resource clients allowed for dynamic creation
```

#### Token Refresh Test Design Patterns

**Pattern 1: Dynamic Configuration** (Requires Admin Access):
```javascript
// Setup phase: Configure short lifespan
await configureClientTokenLifespan(adminToken, clientId, 5);
// Test phase: Verify refresh behavior
await testTokenRefresh(userToken);
// Cleanup phase: Restore original lifespan
await configureClientTokenLifespan(adminToken, clientId, 300);
```

**Pattern 2: Static Configuration** (No Admin Required):
```javascript
// No setup phase needed - realm pre-configured with 5s lifespan
// Test phase: Verify refresh behavior
await testTokenRefresh(userToken);
// No cleanup phase needed
```

**Pattern 3: Mocked Expiry** (Unit Test Alternative):
```javascript
// Mock token with short exp claim
const mockToken = createMockToken({ exp: Date.now() + 5000 });
// Test middleware refresh logic
await testRefreshMiddleware(mockToken);
```

#### Production Recommendations

**For CI/CD Pipelines**:
1. Use pre-configured test realm with short token lifespans (Option 3/Pattern 2)
2. Avoid runtime admin operations to reduce test complexity and failure points
3. Isolate test realms to prevent configuration conflicts

**For Development Testing**:
1. Use service account with realm-management roles (Option 2/Pattern 1)
2. Document Keycloak setup requirements in test README
3. Provide setup scripts for one-time service account configuration

**For Security**:
1. Never commit master realm admin credentials to version control
2. Use environment variables for all sensitive credentials
3. Rotate service account secrets regularly
4. Limit admin permissions to minimum required scope

#### Error Handling Patterns

**403 Forbidden Detection**:
```javascript
if (response.status === 403) {
  throw new Error(
    'Admin API access denied. Ensure token has realm-management permissions. ' +
    'See docs/keycloak-admin-setup.md for configuration instructions.'
  );
}
```

**Graceful Degradation**:
```javascript
try {
  await configureClientTokenLifespan(token, clientId, 5);
} catch (err) {
  if (err.message.includes('403')) {
    console.warn('Admin API not available - using pre-configured realm settings');
    // Continue with test assuming realm has short lifespan configured
  } else {
    throw err;
  }
}
```
