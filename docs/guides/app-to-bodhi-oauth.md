# App-to-BodhiApp OAuth Integration

This guide covers how external applications integrate with BodhiApp through its OAuth2 access-request flow, enabling secure API access with explicit per-resource user consent and fine-grained, grant-based permissions.

## Overview

The app-to-BodhiApp OAuth flow lets external applications access BodhiApp APIs on behalf of a user. The flow is **single-step and owner-driven**: your app files an access request, the BodhiApp owner reviews and consents to exactly which models and MCP servers you may use, and only then is an OAuth scope minted. This integration pattern supports:

- **Dynamic Resource Access**: Apps connect to BodhiApp instances without pre-configuration
- **Explicit User Consent**: The owner reviews and approves (or denies) each request, choosing the granted role, models, and MCP servers
- **Grant-Based Permissions**: Fine-grained, fail-closed access control via per-resource grant envelopes (not just a coarse role)
- **Standard OAuth2 Compliance**: Uses Keycloak authorization-code + PKCE; BodhiApp performs an internal RFC 8693 token exchange on every call

## Prerequisites

Before implementing the OAuth flow, ensure you have:

### 1. BodhiApp Server Information
- **BodhiApp Server URL**: The target BodhiApp instance (e.g., `http://localhost:1135`)
- **Auth Server URL**: The bodhi-auth-server URL (typically `https://id.getbodhi.app`)
- **Realm**: Authentication realm (usually `bodhi`)

### 2. App Client Registration
Your application must be registered as an app client in the bodhi-auth-server system. This is typically done through the developer console at `console.getbodhi.app`.

### 3. User Account
Users must have accounts in the bodhi-auth-server realm and appropriate permissions on the target BodhiApp instance.

## Integration Flow Overview

The complete integration involves these steps:

```
1. Request Access  → App files a Draft access request (expires in 10 min), gets a review_url
2. Owner Review    → Owner opens review_url, sees the requested role, models, and MCP servers
3. Approve / Deny  → Owner approves (mints scope_access_request:<id>) or denies the request
4. OAuth Flow      → App appends the minted scope to its own Keycloak authorize URL; user signs in
5. API Calls       → App calls BodhiApp APIs with the Bearer token (internal RFC 8693 exchange)
6. Manage / Revoke → Owner lists connected apps and can revoke access at any time
```

The app builds its own Keycloak authorize URL (and an error URL) and passes them to the review page as **client-side query params** — BodhiApp never stores redirect URIs. There is **no BodhiApp token-exchange endpoint**; the exchange happens internally.

## Step 1: Request Access

Your app files an access request describing the role and resources it wants. This creates a short-lived **Draft** that the owner must review.

### Endpoint
```http
POST /bodhi/v1/apps/request-access
Content-Type: application/json
```

### Request Format
```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/apps/request-access', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    app_client_id: 'app-your-client-id',
    requested_role: 'scope_user_user',
    requested: {
      version: '1',
      models_list: false,
      models_access: true,
      mcps_list: false,
      mcps_access: false,
      mcp_servers: [{ url: 'https://mcp.example.com/mcp' }]
    }
  })
});
```

`requested_role` is a `UserScope` — either `scope_user_user` or `scope_user_power_user`. The `requested` object is a versioned `RequestedResources` v1 envelope (see [Grant Envelopes](#grant-envelopes)); its four booleans are **UI drivers only** that tell the owner's consent screen which controls to render — they are **not** grants.

**Response JSON (201)**:
```json
{
  "id": "<ulid>",
  "status": "draft",
  "review_url": "http://localhost:1135/ui/apps/access-requests/review?id=<id>"
}
```

### Implementation Notes
- **Not cached, not idempotent**: Each call creates a fresh Draft that expires **10 minutes** after creation. Do not cache the `id` across sessions — file a new request when you need one.
- **No scope yet**: No OAuth scope exists at this point. The scope (`scope_access_request:<id>`) is minted **only at approval** (Step 3).
- **No Authentication**: This endpoint doesn't require authentication (tenant/user are NULL on the Draft until an owner approves it).
- **App-built redirect URLs**: Pre-build your Keycloak authorize URL and an error URL and pass them as client-side query params when you direct the owner to `review_url`. The backend never sees or stores them (the old `flow_type`/`redirect_uri` intake was removed).
- **400** on an invalid body; **404** if the app client is not found.

### (Optional) Poll Request Status
Your app can poll the Draft's status while the owner reviews:
```http
GET /bodhi/v1/apps/access-requests/{id}?app_client_id=app-your-client-id
```
Returns `{ id, status, requested_role, approved_role?, access_request_scope? }`. `approved_role` and `access_request_scope` are present only once the request is approved. A client-id mismatch is hidden as **404**.

## Step 2: Owner Review & Consent

The owner opens `review_url` in an authenticated BodhiApp session. The review page loads the request detail and validates the app-supplied authorize URL against the canonical one.

### Load the Review
```http
GET /bodhi/v1/access-requests/{id}/review
```
Session-authenticated. Returns an `AccessRequestReviewResponse`:
```json
{
  "id": "<ulid>",
  "app_client_id": "app-your-client-id",
  "app_name": "Your App",
  "app_description": "…",
  "status": "draft",
  "requested_role": "scope_user_user",
  "requested": { "version": "1", "models_list": false, "models_access": true, "mcps_list": false, "mcps_access": false, "mcp_servers": [ { "url": "https://mcp.example.com/mcp" } ] },
  "mcps_info": [ { "url": "https://mcp.example.com/mcp", "instances": [ /* owner's matching Mcp instances */ ] } ],
  "auth_endpoint": "https://id.getbodhi.app/realms/bodhi/protocol/openid-connect/auth"
}
```
- `mcps_info` lists the owner's own MCP instances, with exact-URL matches surfaced first, so the owner can bind each requested URL to one of their instances.
- `auth_endpoint` is the canonical Keycloak authorize URL; the page uses it to validate the authorize URL your app supplied.
- Returns **404** if not found, **410** if the Draft has expired.

## Step 3: Approve or Deny

### Approve
```http
PUT /bodhi/v1/access-requests/{id}/approve
Content-Type: application/json
```
Body `ApproveAccessRequest`:
```json
{
  "approved_role": "scope_user_user",
  "approved": {
    "version": "1",
    "models_list": false,
    "models_access": { "type": "all" },
    "mcps_list": false,
    "mcps": [
      { "url": "https://mcp.deepwiki.com/mcp", "status": "approved",
        "instance": { "id": "instance-uuid", "path": "/bodhi/v1/apps/mcps/instance-uuid/mcp" } }
    ],
    "mcps_access": { "type": "specific", "ids": [] }
  }
}
```
On success the request moves Draft → **Approved**: BodhiApp asks Keycloak to mint `scope_access_request:<id>`, and persists the `approved` JSON, `approved_role`, and scope.

**Response JSON (200)**:
```json
{
  "status": "approved",
  "access_request_scope": "scope_access_request:<id>"
}
```
The review page then appends `access_request_scope` to the app-supplied Keycloak authorize URL and redirects the user there.

**Guards** (403 on violation):
- `approved_role` is clamped to at most the `requested_role` **and** at most the approver's `max_grantable` (PowerUser if the approver is PowerUser+, otherwise User). Exceeding either → `PrivilegeEscalation`.
- The approver must be a Session with role ≥ User. ApiToken / ExternalApp / Anonymous callers → `InsufficientPrivileges`.
- Each approved by-url MCP must carry an `instance` referencing one of the owner's **owned + enabled** instances (missing → 400 `McpInstanceNotConfigured`, not owned → 403 `McpInstanceNotOwned`, disabled → 400).
- The `approved` envelope's `version` must equal the stored `requested` version, else `VersionMismatch`.
- **404** unknown id; **409** if the request was already processed (not still Draft).

### Deny
```http
POST /bodhi/v1/access-requests/{id}/deny
```
No body. Moves Draft → **Denied** and returns `{ "status": "denied" }`. The review page redirects the user back to the app-supplied error URL with `error=access_denied&error_source=bodhi`. Returns **404**/**409** if the request is missing or not in a deniable state.

## Step 4: OAuth Authorization Flow

Once the owner approves, the app already holds the minted `scope_access_request:<id>` (via the review-page redirect or by polling Step 1's status endpoint). The app runs a standard Keycloak authorization-code + PKCE flow against **its own** authorize URL, adding that scope.

### Build Authorization URL
```typescript
function buildAuthUrl(
  authServerUrl: string,
  clientId: string,
  redirectUri: string,
  accessRequestScope: string,   // scope_access_request:<id> from approval
  userAccessLevel: 'scope_user_user' | 'scope_user_power_user'
): string {
  const scopes = [
    'openid',
    'email',
    'profile',
    'roles',
    userAccessLevel,      // requested user role
    accessRequestScope    // dynamic scope minted at approval
  ];

  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: scopes.join(' '),
    state: generateRandomString(32),
    code_challenge: generatePKCEChallenge(),
    code_challenge_method: 'S256'
  });

  return `${authServerUrl}/realms/bodhi/protocol/openid-connect/auth?${params}`;
}
```

The user completes Keycloak auth against this app-owned URL. **BodhiApp has no authorize or callback endpoint in this flow** — it only mints the scope; the OAuth round-trip is entirely between your app and Keycloak.

### Exchange Authorization Code for Tokens
```http
POST /realms/bodhi/protocol/openid-connect/token
Content-Type: application/x-www-form-urlencoded
```
```typescript
const tokenResponse = await fetch(`${authServerUrl}/realms/bodhi/protocol/openid-connect/token`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  body: new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: clientId,
    code: authorizationCode,
    redirect_uri: redirectUri,
    code_verifier: pkceVerifier
  })
});
```
The resulting `access_token` carries `scope_access_request:<id>` and is what your app sends to BodhiApp as a Bearer token.

## Step 5: API Calls

With the user token, call BodhiApp APIs directly. The RFC 8693 exchange happens automatically inside BodhiApp's authentication middleware.

### API Request Format
```typescript
const apiResponse = await fetch('http://localhost:1135/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${userToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'llama3:instruct',
    messages: [
      { role: 'user', content: 'Hello from external app!' }
    ]
  })
});
```

### Available Endpoints
The granted role plus the approved grants determine what you can reach. External apps can call the inference surfaces (`/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`, `/anthropic/v1/messages`, Gemini `/v1beta/models/{model}:{action}`) and the app MCP data-plane (`GET /bodhi/v1/apps/mcps`, `GET /bodhi/v1/apps/mcps/{id}`, `ANY /bodhi/v1/apps/mcps/{id}/mcp`). See [Grant Enforcement](#grant-enforcement) for how ungranted resources behave. For endpoint details, see the [Authentication](authentication.md) and [BodhiApp API](bodhi-api.md) guides.

## Step 6: Token Exchange (Internal)

This happens automatically inside BodhiApp on every request. Understanding it helps with troubleshooting — but note **there is no public exchange, upgrade, or supersede endpoint** (that capability was descoped).

### Process Flow
1. **Token Validation**: BodhiApp validates the incoming user token (`iss`, `aud`, signature).
2. **Scope Resolution**: It collects the `scope_access_request:<id>` scope and loads the bound access-request row.
3. **Bound-Row Checks**: The row must be **Approved**, its `app_client_id` must match, and its `user_id` must match the token subject.
4. **RFC 8693 Exchange**: BodhiApp exchanges the app token against Keycloak's `/token` endpoint and cross-checks that the exchanged `access_request_id` claim equals the row id.
5. **Grant Resolution**: It parses the stored `approved` JSON into the grant envelope and builds an `AuthContext::ExternalApp` (fail-closed on a corrupt payload → invalid token).
6. **Caching**: The result is cached by token digest for **5 minutes**; a revoke evicts it immediately.

## Grant Envelopes

Two versioned envelopes drive the flow. Both serialize as a **flat object** with a mandatory `"version": "1"` tag; an unknown version is rejected (`Unsupported resources version '<x>'. Supported versions: [1]`).

### RequestedResources (what the app asks for)
```json
{
  "version": "1",
  "models_list": false,
  "models_access": true,
  "mcps_list": false,
  "mcps_access": false,
  "mcp_servers": [{ "url": "https://mcp.example.com/mcp" }]
}
```
The four booleans are **UI drivers only** — they tell the consent screen which controls to render, not what is granted. `models_access` defaults `true`; the other three default `false`. `mcp_servers` (items `{ url }`) is omitted when empty.

### ApprovedResources (what the owner grants)
```json
{
  "version": "1",
  "models_list": false,
  "models_access": { "type": "all" },
  "mcps_list": false,
  "mcps": [
    { "url": "https://mcp.deepwiki.com/mcp", "status": "approved",
      "instance": { "id": "instance-uuid", "path": "/bodhi/v1/apps/mcps/instance-uuid/mcp" } }
  ],
  "mcps_access": { "type": "specific", "ids": [] }
}
```
- `models_access` and `mcps_access` are grants of shape `{"type":"all"}` or `{"type":"specific","ids":[...]}`. Both **default to specific/empty ⇒ deny** — all-access must be explicit `{"type":"all"}`. There is no `none` variant.
- `mcps` is the list of by-url approvals. Each approved entry must carry an `instance` (owner's owned + enabled instance). On deserialize only `instance.id` is read; `path` is recomputed as `/bodhi/v1/apps/mcps/{id}/mcp`.
- `mcps_access` covers **owner-extra** MCP instances beyond the requested URLs; it defaults to none (asymmetric-on-purpose vs API tokens). Effective MCP access is the **union** of the by-url approvals and `mcps_access`.
- `models_list` / `mcps_list` toggle full-catalog listing (including future resources). Listable = the toggle **OR** any individually granted resource.

## Grant Enforcement

A single `AccessPolicy` derived from the approved grants gates every request. It is **fail-closed**: an app with no bound grant resolves to Deny.

| Surface | Ungranted behavior |
|---|---|
| Inference (`/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`, `/anthropic/v1/messages`, Gemini `:{action}`) | **403** `token_grant_error-model_forbidden` |
| MCP connect/invoke (`ANY /bodhi/v1/apps/mcps/{id}/mcp`) | **403** `token_grant_error-mcp_forbidden` |
| Direct MCP GET (`GET /bodhi/v1/apps/mcps/{id}`) | **404** `entity_error-not_found` (existence hidden) |
| List endpoints (`GET /v1/models`, `GET /bodhi/v1/apps/mcps`, …) | Non-listable resources silently omitted (no error) |

Revoking an app (Step 6 below) tears access down immediately, enforced at three layers: the token exchange requires an Approved row, the `/apps/mcps/*` middleware re-checks Approved on every call, and the cached exchange is evicted.

## Managing & Revoking App Access

The owner can review and revoke connected apps at any time from an authenticated session.

### List Connected Apps
```http
GET /bodhi/v1/access-requests/apps
```
Returns `{ "data": [AppAccessSummary] }`, listing the caller's Approved and Revoked apps. Each `AppAccessSummary` reflects the effective grants:
```json
{
  "id": "<ulid>",
  "app_client_id": "app-your-client-id",
  "app_name": "Your App",
  "app_description": "…",
  "status": "approved",
  "approved_role": "scope_user_user",
  "models": { "type": "all", "list": false },
  "mcps": { "type": "specific", "list": false, "ids": ["instance-uuid"] },
  "created_at": "…",
  "updated_at": "…"
}
```
`models`/`mcps` are `ResourceAccess` values: `{"type":"all","list":bool}` or `{"type":"specific","list":bool,"ids":[...]}` (no `none` variant).

### Revoke Access
```http
POST /bodhi/v1/access-requests/{id}/revoke
```
No body. Moves Approved → **Revoked**, returns the updated `AppAccessSummary`, and evicts the cached exchange so the app token stops working immediately. Returns **404**/**409** if the request is not in a revocable state.

## Complete Implementation Example

Here's a TypeScript implementation of the app side of the flow:

```typescript
interface RequestedResources {
  version: '1';
  models_list: boolean;
  models_access: boolean;
  mcps_list: boolean;
  mcps_access: boolean;
  mcp_servers?: { url: string }[];
}

interface RequestAccessResponse {
  id: string;
  status: 'draft';
  review_url: string;
}

class BodhiAppIntegration {
  constructor(
    private appClientId: string,
    private authServerUrl: string = 'https://id.getbodhi.app',
    private realm: string = 'bodhi'
  ) {}

  // Step 1: File an access request (creates a 10-min Draft; not idempotent)
  async requestAccess(
    bodhiAppUrl: string,
    requestedRole: 'scope_user_user' | 'scope_user_power_user',
    requested: RequestedResources
  ): Promise<RequestAccessResponse> {
    const response = await fetch(`${bodhiAppUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_client_id: this.appClientId,
        requested_role: requestedRole,
        requested
      })
    });

    if (!response.ok) {
      throw new Error(`Request access failed: ${response.status}`);
    }

    return await response.json(); // { id, status: 'draft', review_url }
  }

  // Step 1 (optional): Poll status until the owner approves/denies
  async pollStatus(bodhiAppUrl: string, id: string) {
    const url = `${bodhiAppUrl}/bodhi/v1/apps/access-requests/${id}?app_client_id=${this.appClientId}`;
    const response = await fetch(url);
    if (!response.ok) throw new Error(`Status check failed: ${response.status}`);
    return await response.json(); // { id, status, requested_role, approved_role?, access_request_scope? }
  }

  // Step 4: Build OAuth authorize URL with the minted access-request scope
  buildAuthUrl(
    redirectUri: string,
    accessRequestScope: string,   // scope_access_request:<id> from approval
    userAccessLevel: 'scope_user_user' | 'scope_user_power_user' = 'scope_user_user'
  ): string {
    const scopes = ['openid', 'email', 'profile', 'roles', userAccessLevel, accessRequestScope];

    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.appClientId,
      redirect_uri: redirectUri,
      scope: scopes.join(' '),
      state: this.generateState(),
      code_challenge: this.generatePKCEChallenge(),
      code_challenge_method: 'S256'
    });

    return `${this.authServerUrl}/realms/${this.realm}/protocol/openid-connect/auth?${params}`;
  }

  // Step 4: Exchange authorization code for tokens (against Keycloak)
  async exchangeCodeForTokens(
    code: string,
    redirectUri: string,
    codeVerifier: string
  ): Promise<TokenResponse> {
    const response = await fetch(`${this.authServerUrl}/realms/${this.realm}/protocol/openid-connect/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: this.appClientId,
        code: code,
        redirect_uri: redirectUri,
        code_verifier: codeVerifier
      })
    });

    if (!response.ok) {
      throw new Error(`Token exchange failed: ${response.status}`);
    }

    return await response.json();
  }

  // Step 5: Make API calls to BodhiApp (internal RFC 8693 exchange is automatic)
  async callBodhiAPI<T>(
    bodhiAppUrl: string,
    endpoint: string,
    userToken: string,
    options: RequestInit = {}
  ): Promise<T> {
    const response = await fetch(`${bodhiAppUrl}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${userToken}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      throw new Error(`API call failed: ${response.status}`);
    }

    return await response.json();
  }

  // Utility methods
  private generateState(): string {
    // ... implementation
  }

  private generatePKCEChallenge(): string {
    // ... implementation
  }
}

// Usage example
const integration = new BodhiAppIntegration('app-your-client-id');

// 1. File an access request and direct the owner to review it
const { id, review_url } = await integration.requestAccess('http://localhost:1135', 'scope_user_power_user', {
  version: '1',
  models_list: false,
  models_access: true,
  mcps_list: false,
  mcps_access: false,
  mcp_servers: [{ url: 'https://mcp.example.com/mcp' }]
});
// Open review_url (with your app's authorize + error URLs as query params) for the owner...

// 2. Once approved, obtain the minted scope (via redirect or polling)
const status = await integration.pollStatus('http://localhost:1135', id);
const accessRequestScope = status.access_request_scope; // scope_access_request:<id>

// 3. Start OAuth flow with that scope
const authUrl = integration.buildAuthUrl('https://yourapp.com/callback', accessRequestScope, 'scope_user_power_user');
// Redirect user to authUrl...

// 4. After callback, exchange code for tokens
const tokens = await integration.exchangeCodeForTokens(authCode, 'https://yourapp.com/callback', pkceVerifier);

// 5. Make API calls
const models = await integration.callBodhiAPI('http://localhost:1135', '/v1/models', tokens.access_token);
```

## Error Handling

### Common Error Scenarios

#### 1. Request / Approval Errors
- **400 Bad Request**: Invalid app client ID or malformed `requested`/`approved` envelope
- **404 Not Found**: App client not found, unknown request id, or `app_client_id` mismatch on status poll
- **409 Conflict**: Request already processed (approve/deny/revoke on a non-eligible state)
- **410 Gone**: Draft expired (older than 10 minutes) — file a new request
- **403 Forbidden**: Owner tried to grant a role above their ceiling, or a non-session caller attempted approval

#### 2. OAuth Flow Errors
- **invalid_request**: Missing or invalid OAuth parameters
- **access_denied**: Owner denied the request (app redirected with `error=access_denied&error_source=bodhi`)
- **invalid_client**: App client not found or misconfigured

#### 3. API Call Errors
- **401 Unauthorized**: Token invalid, expired, missing audience, or its access request is no longer Approved (e.g., revoked)
- **403 Forbidden**: `token_grant_error-model_forbidden` (inference on an ungranted model) or `token_grant_error-mcp_forbidden` (connect to an ungranted MCP)
- **404 Not Found**: Direct GET of a non-listable MCP (existence hidden)

### Error Handling Implementation

```typescript
try {
  const result = await integration.callBodhiAPI(url, endpoint, token);
} catch (error) {
  if (error.status === 401) {
    console.error('Token invalid or access revoked — the user may need to re-authorize');
    // Re-run the request-access + approval flow
  } else if (error.status === 403) {
    console.error('This resource was not granted by the owner');
    // Ask the owner to approve the model/MCP, or request a higher role
  } else {
    console.error('API error:', error.message);
  }
}
```

## Security Considerations

### Token Security
- **Secure Storage**: Store tokens securely, under browser sandbox security if storing on client side
- **Token Validation**: Always validate token expiration before API calls
- **HTTPS Only**: Use HTTPS for all OAuth flows and API calls in production

### OAuth Security
- **PKCE Implementation**: Always use PKCE for public clients
- **State Validation**: Validate the `state` parameter to prevent CSRF attacks
- **Authorize URL Integrity**: The review page validates your app-supplied authorize URL against `auth_endpoint`; keep it pointed at the canonical Keycloak endpoint

### Scope & Grant Management
- **Minimum Request**: Request only the role and resources you actually need — the owner sees exactly what you ask for
- **Fail-Closed Defaults**: Anything not explicitly granted is denied; don't assume access to models or MCPs you didn't request
- **Handle Revocation**: Access can be revoked at any time; surface a clear re-authorization path when calls start returning 401/403

## Testing and Validation

### Verify Integration Steps

1. **Request Access Validation**
   ```typescript
   const { status, review_url } = await integration.requestAccess(bodhiAppUrl, 'scope_user_user', requested);
   console.assert(status === 'draft', 'Expected a Draft request');
   console.assert(!!review_url, 'Missing review_url');
   ```

2. **Approval Scope Validation**
   ```typescript
   const s = await integration.pollStatus(bodhiAppUrl, id);
   console.assert(s.access_request_scope?.startsWith('scope_access_request:'), 'Unexpected scope format');
   ```

3. **API Access Validation**
   ```typescript
   // Confirm the token authenticates and inspect the effective grants
   const userInfo = await callBodhiAPI('/bodhi/v1/user', token);
   console.assert(userInfo.auth_status === 'api_token', 'Token validation failed');
   console.log('Granted models:', userInfo.access?.models);
   console.log('Granted MCPs:', userInfo.access?.mcps);
   ```

### Common Integration Issues

- **Expired Draft**: More than 10 minutes elapsed before approval — file a fresh request
- **Missing Audience**: Token doesn't include BodhiApp's client ID — verify the OAuth `scope` and client config
- **Access Revoked**: Calls suddenly 401 — the owner revoked the app; re-run the request/approval flow
- **Forbidden Resource**: 403 `model_forbidden`/`mcp_forbidden` — the owner didn't grant that model or MCP

## Best Practices

### Implementation
- **File Fresh Requests**: Don't cache request ids; Drafts expire in 10 minutes and are single-use
- **Poll or Redirect**: Obtain the minted scope via the review-page redirect or by polling the status endpoint
- **Token Refresh**: Implement automatic token refresh using refresh tokens
- **Error Recovery**: Provide clear error messages and recovery paths for users

### User Experience
- **Clear Consent Context**: Explain to the owner why your app needs each model/MCP so the review is easy to approve
- **Permission Levels**: Request the appropriate user role (`scope_user_user` vs `scope_user_power_user`)
- **Connection Management**: Point owners to the connected-apps list so they can review and revoke access
- **Offline Handling**: Gracefully handle cases when BodhiApp is unavailable

## Next Steps

After implementing the OAuth flow:

1. **[Explore BodhiApp APIs](bodhi-api.md)** - Learn about available endpoints and capabilities
2. **[Handle Errors](error-handling.md)** - Comprehensive error handling strategies
3. **[Model Management](model-management.md)** - Advanced model workflows for power users
4. **[Examples](examples.md)** - Complete integration examples and patterns

---

*This guide provides the complete access-request and OAuth flow for secure app-to-BodhiApp integration. Owner-driven consent plus fail-closed, grant-based enforcement ensures apps reach exactly the resources the user approved — no more.*
