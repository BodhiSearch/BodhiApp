# Authentication & Authorization

BodhiApp implements a comprehensive authentication and authorization system designed for both individual users and team environments. This guide covers the role hierarchy, API token management, and authorization patterns needed for successful API integration.

## Authentication Overview

BodhiApp supports two primary authentication methods:

### 1. Session-Based Authentication
**Use Case**: Web UI interactions, browser-based access
- **Mechanism**: OAuth2 session cookies
- **Best For**: Interactive web interface usage
- **Automatic Features**: Token refresh, CSRF protection, browser integration

### 2. API Token Authentication  
**Use Case**: Programmatic access, integrations, automation
- **Mechanism**: Bearer token in Authorization header
- **Best For**: API clients, scripts, and automated systems
- **Features**: Long-lived tokens, scope-based permissions, revocable access

## Role Hierarchy System

BodhiApp implements a hierarchical role system where higher roles inherit all permissions of lower roles:

### Role Levels (Hierarchical)

#### Admin (`admin`)
**Highest privilege level** - Complete system control
- **User Management**: Create, modify, and delete user accounts
- **System Configuration**: Modify all application settings
- **Token Management**: Create and revoke API tokens for all users
- **Model Management**: Download, configure, and manage all models
- **API Access**: Full access to all API endpoints
- **Use Cases**: System administrators, deployment managers

#### Manager (`manager`)
**Team management level** - Organizational control
- **Team Management**: Manage users within their organization
- **Resource Allocation**: Control model downloads and system resources
- **Monitoring**: Access to usage analytics and system metrics
- **API Access**: All PowerUser capabilities plus management endpoints
- **Use Cases**: Team leads, project managers, resource coordinators

#### PowerUser (`power_user`)
**Advanced user level** - Extended capabilities
- **Model Configuration**: Create and modify model aliases
- **Advanced Settings**: Access to performance and inference parameters
- **Bulk Operations**: Download multiple models, batch operations
- **API Access**: Full model and chat APIs, advanced configuration endpoints
- **Use Cases**: AI developers, researchers, power users

#### User (`user`)
**Basic user level** - Standard access
- **Chat Access**: Use existing models for chat interactions
- **Basic Settings**: Modify personal preferences and chat parameters  
- **Model Usage**: Use pre-configured models and aliases
- **API Access**: Chat completions, basic model information
- **Use Cases**: End users, basic integrations, simple applications

### Permission Inheritance

```
Admin
├── Complete system control
├── All Manager permissions
├── All PowerUser permissions
└── All User permissions

Manager  
├── Team and resource management
├── All PowerUser permissions
└── All User permissions

PowerUser
├── Advanced model and system configuration
└── All User permissions

User
└── Basic chat and model usage
```

## Token Scope System

API tokens carry a **scope** that governs their role/privilege, and a separate **grant envelope** that governs which models and MCPs they may actually reach. The scope parallels the role hierarchy; it no longer decides resource access on its own.

> **Scope-only access is superseded.** A token's `scope` sets its role (what *kinds* of operations it may perform), while its per-resource `grants` decide *which* models and MCPs it can list and use. Grants are **fail-closed**: a token created without explicit `grants` (or with a corrupt stored envelope) denies everything. See [Token Grants](#token-grants).

### Available Token Scopes

#### `scope_token_power_user`
**Advanced API access** - Model management and advanced features
- Model configuration and alias management
- Model download and pull operations
- Advanced chat parameters and settings

#### `scope_token_user`
**Basic API access** - Chat and basic model operations
- Chat completions and model information
- Read-only access to models and model files
- Standard API usage

### Scope Hierarchy and Access

Token scopes follow a hierarchical pattern where higher scopes include lower scope permissions:

```typescript
// Higher scopes automatically include lower scope permissions
const scopeHierarchy = {
  'scope_token_power_user': ['scope_token_user'],
  'scope_token_user': []
};
```

**Note**: There are exactly two token scopes — `scope_token_user` and `scope_token_power_user`. A `User`-role caller may only mint `scope_token_user`; `PowerUser` and above may mint either. Token scope governs role/privilege; which specific models and MCPs a token can reach is controlled independently by its [grants](#token-grants).

## API Token Management

### Creating API Tokens

#### Via Web Interface
1. **Navigate to Settings**: Go to **Settings** → **API Tokens**
2. **Create New Token**: Click "Create Token" button
3. **Configure Token**:
   - **Name**: Descriptive identifier (e.g., "Production API", "Development Scripts")
   - **Scope**: Select appropriate permission level
   - **Grants**: Choose which models and MCPs the token may access (omit for the deny-everything default)
4. **Save Token**: Copy the generated token immediately (shown only once)

Token management endpoints are **session-only** — API tokens cannot create, list, update, or delete tokens (this prevents privilege escalation). A `scope_token_user` caller may only mint `scope_token_user` tokens; PowerUser+ may mint `user` or `power_user`.

#### Create Request (`POST /bodhi/v1/tokens`)

The request body carries `name` (optional), `scope`, and an optional `grants` envelope:

```json
{
  "name": "My API Token",
  "scope": "scope_token_power_user",
  "grants": {
    "version": "1",
    "models_list": false,
    "models": { "type": "specific", "ids": ["llama3:instruct"] },
    "mcps_list": false,
    "mcps": { "type": "all" }
  }
}
```

Omitting `grants` yields the **deny-everything** default (`models` and `mcps` both `{"type":"specific","ids":[]}`). See [Token Grants](#token-grants) for the envelope shape.

The response returns the raw secret **once** (with `Cache-Control: no-store`):

```json
{
  "token": "bodhiapp_<random>.<client_id>"
}
```

#### Token Detail (list response)

`GET /bodhi/v1/tokens` returns token records — the raw secret is **never** returned again, only `token_prefix`:

```json
{
  "id": "…",
  "user_id": "…",
  "name": "My API Token",
  "token_prefix": "bodhiapp_ab…",
  "scopes": "scope_token_power_user",
  "status": "active",
  "grants": {
    "version": "1",
    "models_list": false,
    "models": { "type": "specific", "ids": ["llama3:instruct"] },
    "mcps_list": false,
    "mcps": { "type": "all" }
  },
  "last_used_at": "2024-01-15T15:45:00Z",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### Token Grants

A token's `grants` are a versioned `TokenGrants` v1 envelope — a **flat object** with a mandatory `"version":"1"` tag:

```json
{
  "version": "1",
  "models_list": false,
  "models": { "type": "specific", "ids": [] },
  "mcps_list": false,
  "mcps": { "type": "specific", "ids": [] }
}
```

- **`models` / `mcps`** are grants of shape `{"type":"all"}` (wildcard, includes future resources) or `{"type":"specific","ids":[...]}` (explicit allowlist). Both **default to `specific`/empty ⇒ deny**; there is no `none` variant. All-access must be explicit `{"type":"all"}`.
- **`models_list` / `mcps_list`** toggle full-catalog listing. A resource is listable if the toggle is on **or** it is individually granted; an individually granted resource is usable for inference even when the full catalog isn't listable.
- The example above is the canonical **deny-everything** default applied when `grants` is omitted or the stored envelope is corrupt (fail-closed).

**Grants are immutable after creation.** They are set only at create time — `PUT /bodhi/v1/tokens/{id}` changes only `name` and `status`. To change what a token can access, **delete it and mint a new one**.

#### Updating a Token (`PUT /bodhi/v1/tokens/{id}`)

The update body is limited to `name` and `status` — it has **no `grants` field**:

```json
{
  "name": "Renamed Token",
  "status": "inactive"
}
```

Returns the updated `TokenDetail`; **404** (`entity_error-not_found`) if the id is missing or not owned.

#### Deleting a Token (`DELETE /bodhi/v1/tokens/{id}`)

This is a **hard delete** — the row is physically removed inside the tenant transaction and the token is revoked immediately (not a soft status flip). Returns **204 No Content**; **404** (`entity_error-not_found`) for a missing or unowned id.

### Using API Tokens

#### HTTP Header Format
```http
Authorization: Bearer your-api-token-here
Content-Type: application/json
```

#### Example API Call
```typescript
const API_TOKEN = 'your-api-token-here';
const BASE_URL = 'http://localhost:1135';

const response = await fetch(`${BASE_URL}/bodhi/v1/info`, {
  headers: {
    'Authorization': `Bearer ${API_TOKEN}`,
    'Content-Type': 'application/json'
  }
});
```

#### Using OpenAI SDK
```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'your-api-token-here',
  baseURL: 'http://localhost:1135/v1'
});

const completion = await client.chat.completions.create({
  model: 'llama3:instruct',
  messages: [{ role: 'user', content: 'Hello!' }]
});
```

### Token Management Best Practices

#### Security Guidelines
- **Token Storage**: Store tokens securely (environment variables, secure vaults)
- **Scope Minimization**: Use the lowest scope that meets your requirements
- **Regular Rotation**: Periodically regenerate tokens for long-running applications
- **Access Monitoring**: Review token usage in the web interface

#### Token Lifecycle Management
```typescript
// Example token validation
class TokenManager {
  private token: string;
  private scope: string;

  async validateToken(): Promise<boolean> {
    try {
      const response = await fetch('/bodhi/v1/user', {
        headers: { 'Authorization': `Bearer ${this.token}` }
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  async getTokenInfo() {
    const response = await fetch('/bodhi/v1/user', {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return response.json();
  }
}
```

### Inspecting the Current Principal (`/bodhi/v1/user`)

`GET /bodhi/v1/user` reflects the calling principal. For token-bearing callers (API tokens and external apps) it includes an `access` envelope describing the effective per-resource grants; the envelope is **omitted** for sessions and anonymous callers.

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

Each of `access.models` / `access.mcps` is a `ResourceAccess` value with exactly two shapes (there is **no `none` variant** — deny is `{"type":"specific","list":false,"ids":[]}`):

- `{ "type": "all", "list": bool }`
- `{ "type": "specific", "list": bool, "ids": [...] }`

`list` mirrors the token's listing toggle (whether the full catalog is discoverable). Validate a token by checking `auth_status` rather than a `logged_in` flag, and read `access` to learn what the token may reach.

## API Endpoint Authorization

BodhiApp uses a multi-layered authorization system with role-based access control and token scopes. Below is the complete authorization matrix for all API endpoints:

### Public Endpoints (No Authentication Required)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/ping` | Health check |
| `GET` | `/bodhi/v1/info` | App information and status |
| `POST` | `/bodhi/v1/setup` | Initial app setup |
| `POST` | `/bodhi/v1/logout` | User logout |
| `POST` | `/bodhi/v1/apps/request-access` | File an app access request (external-app OAuth flow) |
| `GET` | `/bodhi/v1/apps/access-requests/{id}` | Poll app access-request status (`?app_client_id=` match) |
| `GET` | `/dev/secrets` | Development secrets (dev mode only) |
| `GET` | `/dev/envs` | Environment variables (dev mode only) |

### Optional Authentication Endpoints

These endpoints work with or without authentication, providing different information based on auth status:

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/bodhi/v1/user` | User information (returns an `access` envelope for token-bearing callers) | Optional |
| `POST` | `/bodhi/v1/auth/initiate` | Start OAuth flow | No |
| `POST` | `/bodhi/v1/auth/callback` | OAuth callback | No |

### User Level Access

**Required**: `user` role OR `scope_token_user` OR `scope_user_user`

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/v1/models` | List OpenAI-compatible models |
| `GET` | `/v1/models/{id}` | Get specific model info |
| `POST` | `/v1/chat/completions` | OpenAI chat completions |
| `GET` | `/bodhi/v1/models` | List model aliases |
| `GET` | `/bodhi/v1/models/{id}` | Get model alias details |
| `GET` | `/bodhi/v1/modelfiles` | List local model files |

### PowerUser Level Access

**Required**: `power_user` role OR `scope_token_power_user` OR `scope_user_power_user`

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/bodhi/v1/models` | Create model alias |
| `PUT` | `/bodhi/v1/models/{id}` | Update model alias |
| `GET` | `/bodhi/v1/modelfiles/pull` | List model downloads |
| `POST` | `/bodhi/v1/modelfiles/pull` | Start model download |
| `POST` | `/bodhi/v1/modelfiles/pull/{id}` | Pull model by alias |
| `GET` | `/bodhi/v1/modelfiles/pull/{id}` | Get download status |

### PowerUser Session-Only Access

**Required**: `power_user` role (session authentication only, no token scopes)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/bodhi/v1/tokens` | Create API token (with `grants`) |
| `GET` | `/bodhi/v1/tokens` | List user's API tokens |
| `PUT` | `/bodhi/v1/tokens/{token_id}` | Update API token (name + status only; grants immutable) |
| `DELETE` | `/bodhi/v1/tokens/{token_id}` | Delete API token (hard delete, immediate revoke, 204) |

### Admin Session-Only Access

**Required**: `admin` role (session authentication only, no token scopes)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/bodhi/v1/settings` | List system settings |
| `PUT` | `/bodhi/v1/settings/{key}` | Update system setting |
| `DELETE` | `/bodhi/v1/settings/{key}` | Delete system setting |

### Authorization Logic

Authorization has **two independent layers**:

1. **Role / scope gate** — establishes the caller's role (hierarchical: admin > manager > power_user > user), from the session role, the API-token `scope`, or a third-party app's user scope (`scope_user_user` / `scope_user_power_user`). This decides which *kinds* of operations the caller may perform.
2. **Grant gate** — for token-bearing callers (API tokens and external apps), a per-resource grant envelope decides *which* models and MCPs are reachable. This layer is **fail-closed**: anything not explicitly granted is denied. Sessions and anonymous callers are unrestricted at this layer.

The user scopes `scope_user_user` / `scope_user_power_user` are minted for external apps through the owner-consent flow in [App-to-BodhiApp OAuth Integration](app-to-bodhi-oauth.md), where the same grant envelopes gate model and MCP access.

#### Grant Enforcement

For API tokens and external apps, a resource not covered by the token's grants behaves as follows:

| Surface | Ungranted behavior |
|---|---|
| Inference (`POST /v1/chat/completions`, `/v1/embeddings`, `/v1/responses`, `/anthropic/v1/messages`, Gemini `:{action}`) | **403** `token_grant_error-model_forbidden` |
| MCP connect/invoke (`ANY /bodhi/v1/apps/mcps/{id}/mcp`) | **403** `token_grant_error-mcp_forbidden` |
| Model GET (`GET /v1/models/{id}`, Anthropic/Gemini equivalents) | **404 hidden** (`model_not_found`) |
| MCP GET (`GET /bodhi/v1/apps/mcps/{id}`) | **404 hidden** (`entity_error-not_found`) |
| List endpoints (`GET /v1/models`, `GET /bodhi/v1/models`, `GET /bodhi/v1/apps/mcps`, …) | Ungranted resources silently omitted (no error) |

**Important Notes**:
- Session-only endpoints (token management, settings) cannot be accessed via API tokens
- Higher roles automatically include lower role permissions
- Access is **not** determined by scope alone — grants gate the actual model/MCP surface, fail-closed by default
- Token scopes are limited to `scope_token_user` and `scope_token_power_user`; user (app) scopes to `scope_user_user` and `scope_user_power_user` — there are no `manager`/`admin` variants of either
- Role hierarchy (`Admin`/`Manager`/`PowerUser`/`User`) governs session-authenticated access; token/user *scopes* cap at power-user

## BodhiApp Settings System

BodhiApp includes a comprehensive settings management system that allows configuration of various application parameters through multiple sources with a clear precedence hierarchy.

### Settings Architecture

The settings system supports multiple configuration sources with the following precedence (highest to lowest):

1. **System Settings**: Built-in application settings (highest priority)
2. **Command Line**: Settings passed via command line arguments
3. **Environment Variables**: Settings from environment variables
4. **Settings File**: User-configured settings in `~/.cache/bodhi/settings.yaml`
5. **Default Values**: Built-in default values (lowest priority)

### Key Settings Categories

#### System Configuration Settings

| Setting | Description | Default | Type |
|---------|-------------|---------|------|
| `BODHI_HOME` | BodhiApp data directory | `~/.cache/bodhi` | Path |
| `BODHI_ENV_TYPE` | Environment type (production/development) | System-defined | Enum |
| `BODHI_APP_TYPE` | Application type (native/server) | System-defined | Enum |
| `BODHI_VERSION` | Application version | System-defined | String |
| `BODHI_AUTH_URL` | Authentication server URL | `https://id.getbodhi.app` | URL |
| `BODHI_AUTH_REALM` | Authentication realm | System-defined | String |

#### Server Configuration Settings

| Setting | Description | Default | Type |
|---------|-------------|---------|------|
| `BODHI_SCHEME` | Server protocol scheme | `http` | String |
| `BODHI_HOST` | Server host address | `localhost` | String |
| `BODHI_PORT` | Server port number | `1135` | Number (1-65535) |

#### Logging Configuration Settings

| Setting | Description | Default | Type |
|---------|-------------|---------|------|
| `BODHI_LOGS` | Log files directory | `~/.cache/bodhi/logs` | Path |
| `BODHI_LOG_LEVEL` | Logging level | `warn` | Enum (error, warn, info, debug, trace) |
| `BODHI_LOG_STDOUT` | Enable console logging | `false` | Boolean |

#### Model and Storage Settings

| Setting | Description | Default | Type |
|---------|-------------|---------|------|
| `HF_HOME` | HuggingFace cache directory | `~/.cache/huggingface` | Path |
| `BODHI_EXEC_LOOKUP_PATH` | Executable lookup path | System-defined | Path |
| `BODHI_EXEC_VARIANT` | llama.cpp build variant | System-defined | Enum |
| `BODHI_KEEP_ALIVE_SECS` | Server keep-alive timeout | `300` | Number (300-86400) |

### Settings Management via API

Settings can be managed programmatically through the admin-only settings API:

#### List All Settings
```typescript
// GET /bodhi/v1/settings
const response = await fetch('/bodhi/v1/settings', {
  headers: { 'Authorization': `Bearer ${adminToken}` }
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
  }
]
```

#### Update Setting
```typescript
// PUT /bodhi/v1/settings/{key}
await fetch('/bodhi/v1/settings/BODHI_PORT', {
  method: 'PUT',
  headers: {
    'Authorization': `Bearer ${adminToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ value: 8080 })
});
```

#### Delete Setting (Reset to Default)
```typescript
// DELETE /bodhi/v1/settings/{key}
await fetch('/bodhi/v1/settings/BODHI_PORT', {
  method: 'DELETE',
  headers: { 'Authorization': `Bearer ${adminToken}` }
});
```

### Settings File Format

Settings are stored in YAML format at `~/.cache/bodhi/settings.yaml`:

```yaml
# Server Configuration
BODHI_HOST: localhost
BODHI_PORT: 1135
BODHI_SCHEME: http

# Logging Configuration
BODHI_LOG_LEVEL: info
BODHI_LOG_STDOUT: true

# Model Configuration
BODHI_EXEC_VARIANT: metal
BODHI_KEEP_ALIVE_SECS: 600
```

### Environment Variable Override

Any setting can be overridden using environment variables:

```bash
# Override port via environment variable
export BODHI_PORT=8080

# Override log level
export BODHI_LOG_LEVEL=debug

# Start BodhiApp with custom settings
./bodhi-app
```

### Settings Validation

The settings system includes built-in validation:

- **Type Validation**: Ensures values match expected types
- **Range Validation**: Numeric settings have min/max constraints
- **Enum Validation**: String settings with predefined options
- **Path Validation**: Directory and file path settings

### Settings Change Notifications

The settings system supports change notifications for dynamic configuration updates:

- Settings changes trigger internal notifications
- Some settings require application restart
- UI reflects setting changes in real-time where applicable

### Important Directories

Based on settings configuration, BodhiApp uses these key directories:

| Directory | Purpose | Setting |
|-----------|---------|---------|
| `~/.cache/bodhi/` | Main data directory | `BODHI_HOME` |
| `~/.cache/bodhi/logs/` | Log files | `BODHI_LOGS` |
| `~/.cache/bodhi/aliases/` | Model aliases | Derived from `BODHI_HOME` |
| `~/.cache/huggingface/hub/` | Model files | `HF_HOME` |
| `~/.cache/bodhi/bodhi.sqlite` | Main database | Derived from `BODHI_HOME` |
| `~/.cache/bodhi/session.sqlite` | Session database | Derived from `BODHI_HOME` |

## Authentication Flows

### Initial Setup Authentication

During first-time setup, BodhiApp establishes authentication:

1. **App Registration**: BodhiApp registers itself with bodhi-auth-server
2. **Admin Assignment**: First user automatically becomes admin
3. **OAuth Flow**: Standard OAuth2 authorization code flow
4. **Session Establishment**: Browser session created for web UI access

### Programmatic Authentication

For API access, follow this pattern:

```typescript
class BodhiAPIClient {
  private token: string;
  private baseURL: string;

  constructor(token: string, baseURL = 'http://localhost:1135') {
    this.token = token;
    this.baseURL = baseURL;
  }

  private async makeRequest(endpoint: string, options: RequestInit = {}) {
    const response = await fetch(`${this.baseURL}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.token}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`API Error: ${error.message}`);
    }

    return response.json();
  }

  async getUserInfo() {
    return this.makeRequest('/bodhi/v1/user');
  }

  async chatCompletion(messages: any[]) {
    return this.makeRequest('/v1/chat/completions', {
      method: 'POST',
      body: JSON.stringify({
        model: 'llama3:instruct',
        messages
      })
    });
  }
}

// Usage
const client = new BodhiAPIClient('your-api-token');
const userInfo = await client.getUserInfo();
console.log('User role:', userInfo.role);
```

## Security Considerations

### Token Security
- **Storage**: Never store tokens in client-side code or version control
- **Transmission**: Always use HTTPS for token transmission (in production)
- **Scope Limitation**: Use the minimum required scope for each token
- **Monitoring**: Regularly review token usage and access patterns

### Authentication Security
- **OAuth2 Compliance**: Full OAuth2/OpenID Connect implementation
- **Token Validation**: Cryptographic hash verification prevents tampering
- **Session Management**: Secure session handling with CSRF protection
- **Audit Trail**: Authentication and authorization events are logged

### Common Security Patterns

```typescript
// Environment-based token management
const getAPIToken = (): string => {
  const token = process.env.BODHI_API_TOKEN;
  if (!token) {
    throw new Error('BODHI_API_TOKEN environment variable not set');
  }
  return token;
};

// Request retry with token validation
async function makeAuthenticatedRequest(endpoint: string, options: RequestInit) {
  let response = await fetch(endpoint, {
    ...options,
    headers: {
      'Authorization': `Bearer ${getAPIToken()}`,
      ...options.headers
    }
  });

  // Handle token expiration
  if (response.status === 401) {
    // Token may be expired, handle refresh or regeneration
    throw new Error('Authentication failed - token may be expired');
  }

  return response;
}
```

## Troubleshooting Authentication

### Common Issues

#### 401 Unauthorized Errors
**Symptoms**: API calls return 401 status
**Solutions**:
- Verify token is correctly formatted and copied
- Check token hasn't been revoked in the web interface
- Ensure token has appropriate scope for the endpoint
- Confirm `Authorization: Bearer <token>` header format

#### 403 Forbidden Errors  
**Symptoms**: API calls return 403 status
**Solutions**:
- Verify token scope matches endpoint requirements
- Check user role has necessary permissions
- Review endpoint documentation for required permission level

#### Token Creation Issues
**Symptoms**: Cannot create API tokens
**Solutions**:
- Ensure you have appropriate role (PowerUser or higher)
- Check if you're logged in to the web interface
- Verify BodhiApp is properly connected to bodhi-auth-server

#### Authentication Flow Problems
**Symptoms**: OAuth login fails or redirects incorrectly
**Solutions**:
- Verify internet connection to `https://id.getbodhi.app`
- Clear browser cache and cookies
- Check for popup blockers preventing OAuth redirect
- Try incognito/private browsing mode

### Debugging Authentication

```typescript
// Debug authentication status
async function debugAuth(token: string) {
  try {
    // Test basic connectivity
    const pingResponse = await fetch('/ping');
    console.log('Server connectivity:', pingResponse.ok);

    // Test authentication
    const authResponse = await fetch('/bodhi/v1/user', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (authResponse.ok) {
      const userInfo = await authResponse.json();
      console.log('Authentication successful:', userInfo);
      console.log('User role:', userInfo.role);
      console.log('Token scope:', userInfo.role_source);
    } else {
      console.error('Authentication failed:', authResponse.status);
      const error = await authResponse.text();
      console.error('Error details:', error);
    }
  } catch (error) {
    console.error('Request failed:', error);
  }
}
```

## Next Steps

Now that you understand authentication and authorization:

1. **[Explore OpenAI APIs](openai-api.md)** - Use familiar OpenAI-compatible endpoints
2. **[Master Model Management](model-management.md)** - Advanced model workflows  
3. **[Learn BodhiApp APIs](bodhi-api.md)** - Native BodhiApp functionality
4. **[Handle Errors](error-handling.md)** - Troubleshoot authentication issues

---

*Authentication is the foundation for secure API access. The next sections focus on using the APIs effectively with proper authorization.* 