# BodhiApp Native API

BodhiApp provides a comprehensive set of native API endpoints under the `/bodhi/v1/` path prefix. These endpoints offer advanced functionality beyond the OpenAI-compatible APIs, including system information, user management, API token management, and application settings.

## Overview

The BodhiApp native API provides:

- **System Information**: Application status, version, and health checks
- **User Management**: User information and authentication status
- **API Token Management**: Create, list, and manage API tokens (with per-resource grants)
- **Settings Management**: System configuration and preferences
- **Setup and Authentication**: Initial setup and OAuth flows
- **Third-Party App Access**: OAuth access-request flow for external apps (owner-consented, grant-scoped)

## Authentication Requirements

Different endpoints have varying authentication requirements:

- **Public**: No authentication required
- **Optional**: Works with or without authentication
- **User**: Requires user-level access
- **PowerUser**: Requires power user access (session-only for tokens)
- **Admin**: Requires admin access (session-only for settings)

## System Endpoints

### Health Check

#### Endpoint: `GET /ping`

Simple health check endpoint to verify server availability.

**Authentication**: None required

```typescript
const response = await fetch('http://localhost:1135/ping');
const result = await response.json();
```

**Response Format**:
```json
{
  "message": "pong"
}
```

**Use Cases**:
- Health monitoring
- Service discovery
- Load balancer checks
- Application startup verification

### Application Information

#### Endpoint: `GET /bodhi/v1/info`

Get application version and status information.

**Authentication**: None required

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/info');
const appInfo = await response.json();
```

**Response Format**:
```json
{
  "version": "0.1.0",
  "status": "ready"
}
```

**Status Values**:
- `setup`: Application needs initial configuration
- `resource-admin`: Application is registered, need to create admin
- `ready`: Application is fully configured and operational

**Use Cases**:
- Version checking for compatibility
- Deployment verification
- Status monitoring
- Feature availability detection

## User Information

### Current User Information

#### Endpoint: `GET /bodhi/v1/user`

Get information about the currently authenticated user.

**Authentication**: Optional (returns different information based on auth status)

```typescript
// Without authentication
const response = await fetch('http://localhost:1135/bodhi/v1/user');

// With authentication
const response = await fetch('http://localhost:1135/bodhi/v1/user', {
  headers: { 'Authorization': `Bearer ${apiToken}` }
});

const userInfo = await response.json();
```

The response is a discriminated union on `auth_status` (`logged_out` | `logged_in` | `api_token`), wrapped in an envelope that adds optional `dashboard` and `access` fields.

**Not authenticated**:
```json
{ "auth_status": "logged_out" }
```

**Session (logged in)**:
```json
{
  "auth_status": "logged_in",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "user@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "role": "resource_user",
  "id_token": "<oidc-id-token>"
}
```
`first_name`, `last_name`, `role`, and `id_token` are nullable/optional. `id_token` is present only for session auth (used by the frontend to call the external reference API).

**API token / external app**:
```json
{
  "auth_status": "api_token",
  "role": "scope_token_user",
  "access": {
    "models": { "type": "specific", "list": false, "ids": ["alias-1"] },
    "mcps":   { "type": "all",      "list": true }
  }
}
```
The `api_token` core body carries only `auth_status` + `role`. The effective per-resource grant is reflected in the separate envelope field **`access`** (`ResourceAccessInfo` = `{ models: ResourceAccess, mcps: ResourceAccess }`). `access` is present only for token-bearing principals (API token or bound external app) and omitted for sessions/anonymous.

**`ResourceAccess`** is discriminated on `type` with exactly two arms — there is no `none` variant:
- `{ "type": "all", "list": bool }` — every current and future resource.
- `{ "type": "specific", "list": bool, "ids": [...] }` — only the listed ids (empty ⇒ no access, the deny default).

`list` mirrors the token's `list_*` toggle (whether it may enumerate the full catalog).

**Use Cases**:
- User profile display
- Permission checking
- Authentication status verification
- Role-based UI rendering

## API Token Management

### List API Tokens

#### Endpoint: `GET /bodhi/v1/tokens`

List all API tokens for the current user.

**Authentication**: PowerUser (session-only)

```typescript
// Note: Only works with session authentication, not API tokens
const response = await fetch('http://localhost:1135/bodhi/v1/tokens', {
  credentials: 'include' // Include session cookies
});

const tokens = await response.json();
```

**Query Params**: `page`, `page_size` (cap 100). Returns the caller's own tokens only.

**Response Format** (`PaginatedTokenResponse` of `TokenDetail`):
```json
{
  "data": [
    {
      "id": "token-123",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Development Token",
      "token_prefix": "sk-bodhiapp_abc12",
      "scopes": "scope_token_power_user",
      "status": "active",
      "grants": {
        "version": "1",
        "models_list": false,
        "models": { "type": "specific", "ids": ["alias-1"] },
        "mcps_list": false,
        "mcps": { "type": "specific", "ids": [] }
      },
      "last_used_at": "2024-01-15T14:20:00Z",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 30
}
```

The raw token is **never** returned by list — only `token_prefix`. `grants` is the full grant envelope (see [Token Grants](#token-grants) below). There is no `expires_at` field; `last_used_at` is optional.

### Create API Token

#### Endpoint: `POST /bodhi/v1/tokens`

Create a new API token for programmatic access.

**Authentication**: PowerUser (session-only)

```typescript
const tokenData = {
  name: 'My API Token',
  scope: 'scope_token_power_user',
  grants: {
    version: '1',
    models_list: false,
    models: { type: 'specific', ids: ['alias-1'] },
    mcps_list: false,
    mcps: { type: 'specific', ids: [] }
  }
};

const response = await fetch('http://localhost:1135/bodhi/v1/tokens', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify(tokenData)
});

const newToken = await response.json();
```

**Request Format** (`CreateTokenRequest`):
```json
{
  "name": "My API Token",
  "scope": "scope_token_power_user",
  "grants": {
    "version": "1",
    "models_list": false,
    "models": { "type": "specific", "ids": ["alias-1"] },
    "mcps_list": false,
    "mcps": { "type": "specific", "ids": [] }
  }
}
```
- `name` (optional, 0–100 chars) and `scope` are the only required inputs; `grants` is optional. There is no `expires_at`.
- `grants` omitted ⇒ **deny-everything** default (fail-closed): the token can reach no models or MCPs until grants are set. See [Token Grants](#token-grants).

**Response Format** — **201 Created**, `Cache-Control: no-store`:
```json
{
  "token": "sk-bodhiapp_<base64url-random><checksum>.<client_id>"
}
```

**Available Scopes** (`TokenScope`):
- `scope_token_user`: Basic API access
- `scope_token_power_user`: Advanced API access including model management

**Privilege ceiling**: a caller with `ResourceRole::User` may mint only `scope_token_user`; requesting a higher scope returns **403** (`TokenRouteError::PrivilegeEscalation`). PowerUser+ callers may mint `scope_token_user` or `scope_token_power_user`.

**Important**: The raw token (format `sk-bodhiapp_<random><checksum>.<client_id>`) is returned **only once** at creation and never again. Store it securely!

### Update API Token

#### Endpoint: `PUT /bodhi/v1/tokens/{token_id}`

Update an existing API token's properties.

**Authentication**: PowerUser (session-only)

```typescript
const updates = {
  name: 'Updated Token Name',
  status: 'inactive'
};

const response = await fetch('http://localhost:1135/bodhi/v1/tokens/token-123', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify(updates)
});

const updatedToken = await response.json();
```

**Request Format** (`UpdateTokenRequest`) — **only** these two fields:
```json
{ "name": "Updated Token Name", "status": "inactive" }
```
- `name`: 3–100 chars
- `status`: `active` or `inactive`

Returns **200** with the updated `TokenDetail`, or **404** (`entity_error-not_found`) if the id is absent or not owned by the caller.

**Note**: Scope and **grants are immutable** after creation. To change grants, delete the token and mint a new one. There is no expiration.

### Delete API Token

#### Endpoint: `DELETE /bodhi/v1/tokens/{token_id}`

Permanently delete (hard-delete) an API token, immediately revoking it.

**Authentication**: PowerUser (session-only)

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/tokens/token-123', {
  method: 'DELETE',
  credentials: 'include'
});
```

This is a physical row delete (not a soft-delete or status flip) inside the tenant transaction; the token stops working immediately.

- **204** No Content on success.
- **404** (`entity_error-not_found`) if the id is missing or not owned by the caller.

## Token Grants

API tokens carry a per-resource **grant envelope** (`TokenGrants`) that gates which models and MCPs the token can reach — independent of its `scope` (which governs role/privilege only). Grants are set at create time and are immutable thereafter.

The envelope is versioned; the `version` tag is mandatory:
```json
{
  "version": "1",
  "models_list": false,
  "models": { "type": "specific", "ids": [] },
  "mcps_list": false,
  "mcps": { "type": "specific", "ids": [] }
}
```
This exact value is the canonical **deny-everything default** applied when `grants` is omitted at create.

- `models` / `mcps` (`ModelGrant` / `McpGrant`): `{ "type": "all" }` (wildcard, includes future resources) or `{ "type": "specific", "ids": [...] }` (explicit allowlist; empty ⇒ no access). Default is empty `specific` (deny). There is **no `none` variant**.
- `models_list` / `mcps_list`: listing toggles. OFF ⇒ discovery endpoints return only the individually granted resources; ON ⇒ full catalog is listable including future resources. An individually granted resource is usable for inference even when not in the full catalog listing.

**Enforcement** (denied behavior): inference on a non-granted model/MCP returns **403** (`token_grant_error-model_forbidden` / `token_grant_error-mcp_forbidden`); a direct GET of a non-listable resource returns **404** with existence hidden; list endpoints silently omit non-granted resources. Sessions are unrestricted.

## Settings Management

### List System Settings

#### Endpoint: `GET /bodhi/v1/settings`

List all system settings with their current values and metadata.

**Authentication**: Admin (session-only)

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/settings', {
  credentials: 'include'
});

const settings = await response.json();
```

**Response Format**:
```json
[
  {
    "key": "BODHI_PORT",
    "current_value": 1135,
    "default_value": 1135,
    "source": "Default",
    "metadata": {
      "type": "Number",
      "min": 1,
      "max": 65535
    }
  },
  {
    "key": "BODHI_LOG_LEVEL",
    "current_value": "info",
    "default_value": "warn",
    "source": "SettingsFile",
    "metadata": {
      "type": "Option",
      "options": ["error", "warn", "info", "debug", "trace"]
    }
  }
]
```

**Setting Sources**:
- `System`: Built-in system settings
- `CommandLine`: Command line arguments
- `Environment`: Environment variables
- `SettingsFile`: Configuration file
- `Default`: Default values

### Update System Setting

#### Endpoint: `PUT /bodhi/v1/settings/{key}`

Update a specific system setting.

**Authentication**: Admin (session-only)

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/settings/BODHI_LOG_LEVEL', {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify({ value: 'debug' })
});

const result = await response.json();
```

**Request Format**:
```json
{
  "value": "debug"
}
```

**Common Settings**:
- `BODHI_PORT`: Server port (1-65535)
- `BODHI_LOG_LEVEL`: Logging level (error, warn, info, debug, trace)
- `BODHI_LOG_STDOUT`: Console logging (true/false)
- `BODHI_KEEP_ALIVE_SECS`: Keep-alive timeout (300-86400)

### Delete System Setting

#### Endpoint: `DELETE /bodhi/v1/settings/{key}`

Reset a setting to its default value.

**Authentication**: Admin (session-only)

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/settings/BODHI_LOG_LEVEL', {
  method: 'DELETE',
  credentials: 'include'
});
```

This removes the setting from the configuration file, causing it to fall back to the default value.

## Authentication and Setup Endpoints

### OAuth Authentication Initiation

#### Endpoint: `POST /bodhi/v1/auth/initiate`

Start the OAuth authentication flow.

**Authentication**: None required

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/auth/initiate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    redirect_uri: 'http://localhost:1135/ui/auth/callback'
  })
});

const authInfo = await response.json();
// Redirect user to authInfo.authorization_url
```

### OAuth Callback

#### Endpoint: `POST /bodhi/v1/auth/callback`

Handle OAuth callback after user authentication.

**Authentication**: None required

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/auth/callback', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    code: 'auth_code_from_callback',
    state: 'state_from_initiate'
  })
});
```

### Initial Application Setup

#### Endpoint: `POST /bodhi/v1/setup`

Configure the application during initial setup.

**Authentication**: None required (only works when app status is "setup")

```typescript
const setupData = {
  name: 'My BodhiApp Instance',
  description: 'Local AI server for development'
};

const response = await fetch('http://localhost:1135/bodhi/v1/setup', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(setupData)
});

const result = await response.json();
```

### User Logout

#### Endpoint: `POST /bodhi/v1/logout`

Log out the current user and invalidate session.

**Authentication**: None required

```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/logout', {
  method: 'POST',
  credentials: 'include'
});
```

## Third-Party App Access (OAuth Access-Request Flow)

External apps obtain scoped, owner-consented access via a single-step access-request flow: the app creates a Draft request, the resource owner reviews and approves it (choosing model + MCP grants and a role), a dynamic scope `scope_access_request:<id>` is minted, and the app then authenticates with Keycloak. On each call the server internally performs an RFC-8693 token exchange (there is **no** BodhiApp token-exchange/upgrade endpoint). For the full end-to-end narrative, grant-envelope semantics, and app-side integration, see **[app-to-bodhi-oauth.md](app-to-bodhi-oauth.md)**.

Endpoint reference:

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| POST | `/bodhi/v1/apps/request-access` | None | App creates Draft access request → `201 {id, status:"draft", review_url}` |
| GET | `/bodhi/v1/apps/access-requests/{id}?app_client_id=` | None | App polls status (client mismatch hidden as 404) |
| GET | `/bodhi/v1/access-requests/{id}/review` | Session | Owner loads review detail (`requested`, matched `mcps_info`, `auth_endpoint`) |
| PUT | `/bodhi/v1/access-requests/{id}/approve` | Session (≥ User) | Owner approves with `{approved_role, approved}` → `200 {status:"approved", access_request_scope}` |
| POST | `/bodhi/v1/access-requests/{id}/deny` | Session | Owner denies → `200 {status:"denied"}` |
| GET | `/bodhi/v1/access-requests/apps` | Session | Owner lists their granted apps (with `ResourceAccess`) |
| POST | `/bodhi/v1/access-requests/{id}/revoke` | Session | Owner revokes (immediate; evicts cached exchange) |
| GET | `/bodhi/v1/apps/mcps` | OAuth app (User) | List MCP instances the app may reach (grant-pruned) |
| GET | `/bodhi/v1/apps/mcps/{id}` | OAuth app (User) | Get one MCP instance (404 if not grant-listable) |
| ANY | `/bodhi/v1/apps/mcps/{id}/mcp` | OAuth app (User) | Transparent MCP proxy (403 `token_grant_error-mcp_forbidden` if not granted) |

> These `/access-requests/{id}/...` paths belong to the **app** flow. A separate **user-role** flow (`POST /bodhi/v1/access-requests`, `/access-requests/pending`, `/access-requests/{id}/reject`) is unrelated and not part of app integration.

## Development Endpoints

### Development Secrets (Dev Mode Only)

#### Endpoint: `GET /dev/secrets`

Get development secrets and configuration (only available in development mode).

**Authentication**: None required
**Availability**: Development mode only

```typescript
const response = await fetch('http://localhost:1135/dev/secrets');
const secrets = await response.json();
```

### Environment Variables (Dev Mode Only)

#### Endpoint: `GET /dev/envs`

Get environment variables for debugging (only available in development mode).

**Authentication**: None required
**Availability**: Development mode only

```typescript
const response = await fetch('http://localhost:1135/dev/envs');
const envVars = await response.json();
```

## Advanced Usage Patterns
## Error Handling

### Common Error Scenarios

```typescript
async function handleBodhiAPICall(apiCall: () => Promise<Response>) {
  try {
    const response = await apiCall();
    
    if (!response.ok) {
      const error = await response.json();
      
      switch (response.status) {
        case 400:
          throw new Error(`Invalid request: ${error.error.message}`);
        case 401:
          throw new Error('Authentication required or token invalid');
        case 403:
          throw new Error('Insufficient permissions for this operation');
        case 404:
          throw new Error('Resource not found');
        case 409:
          throw new Error('Resource already exists or conflict');
        case 500:
          throw new Error(`Server error: ${error.error.message}`);
        default:
          throw new Error(`HTTP ${response.status}: ${error.error.message}`);
      }
    }
    
    return response.json();
  } catch (error) {
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new Error('Unable to connect to BodhiApp server');
    }
    throw error;
  }
}

// Usage
try {
  const userInfo = await handleBodhiAPICall(() => 
    fetch('http://localhost:1135/bodhi/v1/user', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
  );
  console.log('User info:', userInfo);
} catch (error) {
  console.error('API call failed:', error.message);
}
```

## Best Practices

### Security

1. **Token Storage**: Store API tokens securely, never in client-side code
2. **Scope Minimization**: Use the lowest privilege scope needed
3. **Token Rotation**: Regularly rotate long-lived tokens
4. **Session Management**: Properly handle session authentication for web apps

### Performance

1. **Health Checks**: Use `/ping` for lightweight health checks
2. **Caching**: Cache app info and settings that don't change frequently
3. **Connection Reuse**: Reuse HTTP connections for multiple requests
4. **Error Handling**: Implement proper retry logic with exponential backoff

### Monitoring

1. **Status Monitoring**: Regular checks of `/bodhi/v1/info` for status changes
2. **Token Usage**: Monitor token usage and activity
3. **Settings Changes**: Track configuration changes via settings API
4. **Error Tracking**: Log and monitor API errors for troubleshooting

## Next Steps

Now that you understand the BodhiApp native API:

1. **[Handle Errors](error-handling.md)** - Implement robust error handling
2. **[See Examples](examples.md)** - Complete integration examples
3. **[API Reference](api-reference.md)** - Quick endpoint reference

---

*The BodhiApp native API provides comprehensive control over your local AI infrastructure, from basic health monitoring to advanced system configuration.* 