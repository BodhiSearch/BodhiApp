# BodhiApp Native API

BodhiApp provides a comprehensive set of native API endpoints under the `/bodhi/v1/` path prefix. These endpoints offer advanced functionality beyond the OpenAI-compatible APIs, including system information, user management, API token management, and application settings.

## Overview

The BodhiApp native API provides:

- **System Information**: Application status, version, and health checks
- **User Management**: User information and authentication status
- **API Token Management**: Create, list, and manage API tokens
- **Settings Management**: System configuration and preferences
- **Setup and Authentication**: Initial setup and OAuth flows

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

**Response Format (Authenticated)**:
```json
{
  "logged_in": true,
  "email": "user@example.com",
  "role": "power_user",
  "token_type": "bearer",
  "role_source": "scope_token"
}
```

**Response Format (Not Authenticated)**:
```json
{
  "logged_in": false,
  "email": null,
  "role": null,
  "token_type": null,
  "role_source": null
}
```

**Token Types**:
- `session`: Session-based authentication (browser cookies)
- `bearer`: API token authentication

**Role Sources**:
- `role`: Direct user role assignment
- `scope_token`: Role derived from token scope
- `scope_user`: Role derived from user scope

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

**Response Format**:
```json
{
  "data": [
    {
      "id": "token-123",
      "name": "Development Token",
      "scope": "scope_token_power_user",
      "status": "active",
      "created_at": "2024-01-15T10:30:00Z",
      "last_used": "2024-01-15T14:20:00Z",
      "expires_at": null
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 30
}
```

### Create API Token

#### Endpoint: `POST /bodhi/v1/tokens`

Create a new API token for programmatic access.

**Authentication**: PowerUser (session-only)

```typescript
const tokenData = {
  name: 'My API Token',
  scope: 'scope_token_power_user',
  expires_at: null // null for non-expiring token
};

const response = await fetch('http://localhost:1135/bodhi/v1/tokens', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify(tokenData)
});

const newToken = await response.json();
```

**Request Format**:
```json
{
  "name": "My API Token",
  "scope": "scope_token_power_user",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response Format**:
```json
{
  "id": "token-456",
  "name": "My API Token",
  "scope": "scope_token_power_user",
  "status": "active",
  "token": "sk-bodhi-abc123...", // Only shown once!
  "created_at": "2024-01-15T15:00:00Z",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Available Scopes**:
- `scope_token_user`: Basic API access
- `scope_token_power_user`: Advanced API access including model management

**Important**: The actual token value is only returned once during creation. Store it securely!

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

**Updatable Fields**:
- `name`: Token display name
- `status`: `active` or `inactive`

**Note**: Token scope and expiration cannot be modified after creation.

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

1. **[Try Ollama APIs](ollama-api.md)** - Use Ollama-compatible endpoints
2. **[Handle Errors](error-handling.md)** - Implement robust error handling
3. **[See Examples](examples.md)** - Complete integration examples
4. **[API Reference](api-reference.md)** - Quick endpoint reference

---

*The BodhiApp native API provides comprehensive control over your local AI infrastructure, from basic health monitoring to advanced system configuration.* 