# Keycloak Integration

> **Purpose**: Full round-trip trace of all Keycloak interactions in the access request feature.
> Covers: SPI endpoints, consent registration, token exchange, dynamic scopes, test-oauth-app reference implementation.

---

## KC SPI Endpoints (BodhiApp -> Keycloak)

### 1. Register Access Request Consent

**Called by**: `AccessRequestService::approve_request()` via `AuthService::register_access_request_consent()`

**File**: `crates/services/src/auth/auth_service.rs`

**Endpoint**: `POST /realms/{realm}/bodhi/users/request-access`

**Authorization**: Bearer `{user_access_token}` (the approver's KC session token)

**Request Body**:
```json
{
  "app_client_id": "my-third-party-app",
  "access_request_id": "01HXYZ-ulid",
  "description": "- builtin-exa-search\n- MCP: https://mcp.example.com/mcp"
}
```

**Responses**:

| Status | Meaning | Handling |
|--------|---------|----------|
| 201 Created | First registration | Extract `access_request_scope` from response |
| 200 OK | Idempotent retry | Same as 201 (same response body) |
| 409 Conflict | UUID collision (different context) | Mark access request as `Failed` in DB |
| 400/401/other | Invalid request or expired token | Return `AuthServiceApiError` |

**Response Body** (201/200):
```json
{
  "access_request_id": "01HXYZ-ulid",
  "access_request_scope": "scope_access_request:01HXYZ-ulid"
}
```

**What KC Does**:
- Registers a dynamic scope `scope_access_request:<id>` at the realm level
- Associates the description with this scope (shown on consent screen)
- Records user's consent for the specified `app_client_id` to use this scope
- When the app later requests this scope in an OAuth flow, KC includes it in the token

### 2. Get App Client Info

**Called by**: Review handler (optional enrichment)

**File**: `crates/services/src/auth/auth_service.rs`

**Endpoint**: `GET /realms/{realm}/bodhi/users/apps/{app_client_id}/info`

**Authorization**: Bearer `{user_token}`

**Response**:
```json
{
  "name": "My Third Party App",
  "description": "A chat client for BodhiApp"
}
```

Used to populate `app_name` and `app_description` in the review response.

---

## Token Exchange (External App -> KC -> BodhiApp)

### RFC 8693 Token Exchange

**Called by**: `DefaultTokenService::handle_external_client_token()` via `AuthService::exchange_app_token()`

**File**: `crates/services/src/auth/auth_service.rs`

**KC Endpoint**: `POST /realms/{realm}/protocol/openid-connect/token`

**Parameters**:
```
grant_type: urn:ietf:params:oauth:grant-type:token-exchange
subject_token: <external_app_token>
subject_token_type: urn:ietf:params:oauth:token-type:access_token
audience: <bodhi_instance_client_id>
scope: scope_access_request:01HXYZ-ulid openid email profile roles
client_id: <bodhi_instance_client_id>
client_secret: <bodhi_instance_client_secret>
```

**What KC Does**:
- Validates the external app's token
- Issues a new access token scoped to the BodhiApp instance
- Injects `access_request_id` claim (from the registered consent)
- Includes `scope_access_request:<id>` in the token's scope claim
- Sets `resource_access` with user's roles for the BodhiApp client

**Response**: Standard OAuth token response with `access_token` containing custom claims.

### Token Claims (Exchanged Token)

```json
{
  "sub": "user-123",
  "azp": "external-app-client",
  "aud": "bodhi-instance-client",
  "iss": "https://kc.example.com/realms/bodhi",
  "scope": "scope_access_request:01HXYZ-ulid openid email profile",
  "access_request_id": "01HXYZ-ulid",
  "resource_access": {
    "bodhi-instance-client": {
      "roles": ["resource_user"]
    }
  }
}
```

---

## Dynamic Scope Lifecycle

1. **Registration**: When user approves an access request, `register_access_request_consent()` creates `scope_access_request:<id>` in KC
2. **Authorization**: External app includes this scope in its OAuth authorize request
3. **Consent**: KC shows the consent screen with the registered description
4. **Token Issuance**: KC includes the scope and `access_request_id` claim in issued tokens
5. **Validation**: BodhiApp's token service extracts the scope, looks up the DB record, validates everything matches

---

## Service Account Token

**Method**: `AuthService::get_client_access_token()`

**Endpoint**: `POST /realms/{realm}/protocol/openid-connect/token`

**Parameters**: `grant_type=client_credentials`, `client_id`, `client_secret`, `scope=service_account`

Used for admin operations (role assignment, user management) — not directly part of the access request flow but used by related auth operations.

---

## Test OAuth App (Reference Implementation)

**Location**: `crates/lib_bodhiserver_napi/test-oauth-app/`

**Framework**: React 18 + TypeScript + Vite

**Port**: 55173 (configured in Playwright)

### App Pages and Flow

```
ConfigPage -> AccessCallbackPage -> OAuthCallbackPage -> TokenPage -> RestPage/ChatPage
```

### 1. ConfigPage — Access Request Creation

**File**: `test-oauth-app/src/components/ConfigForm.tsx`

User configures:
- `bodhiServerUrl`, `authServerUrl`, `realm`
- `clientId` (the test app's KC client ID)
- `isConfidential` (PKCE vs client_secret)
- `requestedRole` — dropdown: `scope_user_user` or `scope_user_power_user`
- `requested` — JSON textarea for toolset types and MCP servers

On submit:
1. Parse `requested` JSON
2. POST to `{bodhiServerUrl}/bodhi/v1/apps/request-access` with:
   ```json
   {
     "app_client_id": "<clientId>",
     "flow_type": "redirect",
     "redirect_url": "{origin}/access-callback",
     "requested_role": "<selected>",
     "requested": { "toolset_types": [...], "mcp_servers": [...] }
   }
   ```
3. Store `access_request_id` in localStorage
4. Redirect browser to `review_url` (BodhiApp's review page)

### 2. AccessCallbackPage — Status Polling

**File**: `test-oauth-app/src/pages/AccessCallbackPage.tsx`

After user approves in BodhiApp and redirects back:

1. Extract `id` from URL `?id=<access_request_id>`
2. Poll `GET {bodhiServerUrl}/bodhi/v1/apps/access-requests/{id}?app_client_id={clientId}`
3. On `status: "approved"`:
   - Extract `access_request_scope` from response
   - Append to OAuth scope: `"openid profile email roles" + " " + access_request_scope`
4. Generate PKCE code_verifier + code_challenge
5. Build KC authorization URL:
   ```
   {authServerUrl}/realms/{realm}/protocol/openid-connect/auth
     ?client_id={clientId}
     &redirect_uri={origin}/callback
     &response_type=code
     &scope={scope_with_access_request}
     &state={random}
     &code_challenge={challenge}
     &code_challenge_method=S256
   ```
6. Redirect to KC for authorization

### 3. OAuthCallbackPage — Token Exchange

**File**: `test-oauth-app/src/pages/OAuthCallbackPage.tsx`

1. Extract `code` and `state` from URL
2. Validate state against stored value
3. Exchange code for token:
   ```
   POST {authServerUrl}/realms/{realm}/protocol/openid-connect/token
     grant_type=authorization_code
     code=<auth_code>
     client_id=<clientId>
     redirect_uri=<redirect>
     code_verifier=<verifier>   (for PKCE / public clients)
     client_secret=<secret>      (for confidential clients)
   ```
4. Save token to localStorage
5. Redirect to `/rest` page

### 4. RestPage — API Calls

Uses the obtained token as `Authorization: Bearer <token>` for BodhiApp API calls.

The token contains:
- `scope_access_request:<id>` — triggers access request validation in BodhiApp's token service
- `access_request_id` claim — verified against DB record
- `sub` (user_id) — must match the user who approved

### OAuth Utility Functions

**File**: `test-oauth-app/src/utils/oauth.ts`

- `buildAuthUrl(config, codeChallenge, state)` — constructs KC authorization URL
- `exchangeCodeForToken(config, code)` — performs token exchange
- `generateCodeVerifier()` / `generateCodeChallenge()` — PKCE helpers
- `generateState()` — CSRF state parameter

---

## Test MCP OAuth Server

**Location**: `crates/lib_bodhiserver_napi/test-mcp-oauth-server/`

**Port**: 55174 (standard), 55175 (DCR mode)

This is a separate concern — it simulates an OAuth2 provider for MCP server authentication, NOT the access request flow. It provides:
- `/.well-known/oauth-authorization-server` metadata
- `/authorize` + `/token` endpoints
- `/register` (Dynamic Client Registration, DCR mode)
- PKCE verification (S256)

Relevant to access requests only insofar as MCP instances accessed via access requests may themselves require OAuth authentication to the MCP server.

---

## End-to-End Integration Trace

```
1. External App (test-oauth-app)
   POST /bodhi/v1/apps/request-access
   -> BodhiApp creates Draft (no KC call)

2. User opens review URL in browser
   GET /bodhi/v1/access-requests/{id}/review
   -> BodhiApp returns review data (may call KC for app info)

3. User approves
   PUT /bodhi/v1/access-requests/{id}/approve
   -> BodhiApp calls KC: POST /realms/{realm}/bodhi/users/request-access
   <- KC returns: { access_request_scope: "scope_access_request:..." }
   -> BodhiApp stores scope in DB, marks Approved

4. External App polls
   GET /bodhi/v1/apps/access-requests/{id}?app_client_id=...
   <- Gets: { status: "approved", access_request_scope: "scope_access_request:..." }

5. External App initiates OAuth
   Redirect to KC: /auth?scope=...+scope_access_request:...
   KC shows consent screen (description from registration)
   User grants consent (or KC auto-grants if already consented)
   KC redirects with authorization code

6. External App exchanges code
   POST KC /token (grant_type=authorization_code)
   <- Gets access_token with access_request_id claim

7. External App calls BodhiApp API
   Authorization: Bearer <token_with_scope_access_request>
   -> BodhiApp token_service:
      a. Extract scope_access_request:<id> from token
      b. DB lookup -> validate status, app_client_id, user_id
      c. KC token exchange (RFC 8693)
      d. Verify access_request_id claim in exchanged token
      e. Read approved_role from DB
      f. Build AuthContext::ExternalApp
   -> api_auth_middleware: check UserScope
   -> access_request_auth_middleware: check entity in approved JSON
   <- API response
```

---

## Key Files

| File | Role |
|------|------|
| `crates/services/src/auth/auth_service.rs` | register_access_request_consent(), get_app_client_info(), exchange_app_token() |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | Token exchange orchestration |
| `crates/lib_bodhiserver_napi/test-oauth-app/src/components/ConfigForm.tsx` | Access request creation UI |
| `crates/lib_bodhiserver_napi/test-oauth-app/src/pages/AccessCallbackPage.tsx` | Status polling + OAuth initiation |
| `crates/lib_bodhiserver_napi/test-oauth-app/src/pages/OAuthCallbackPage.tsx` | Token exchange |
| `crates/lib_bodhiserver_napi/test-oauth-app/src/utils/oauth.ts` | OAuth utility functions |
| `crates/lib_bodhiserver_napi/test-mcp-oauth-server/src/server.ts` | MCP OAuth server (separate concern) |
