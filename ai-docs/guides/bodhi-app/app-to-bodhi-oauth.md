# App-to-BodhiApp OAuth Integration

This guide covers how external applications integrate with BodhiApp through OAuth2 token exchange, enabling secure API access with proper user consent and scope-based permissions.

## Overview

The app-to-BodhiApp OAuth flow enables external applications to securely access BodhiApp APIs on behalf of users through a standardized OAuth2 token exchange process. This integration pattern supports:

- **Dynamic Resource Access**: Apps can connect to BodhiApp instances without pre-configuration
- **User Consent**: Users explicitly approve which apps can access their BodhiApp resources
- **Scope-Based Permissions**: Fine-grained access control through user scopes (`scope_user_user`, `scope_user_power_user`)
- **Standard OAuth2 Compliance**: Uses RFC 8693 token exchange with Keycloak-based authentication

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

The complete integration involves four main steps:

```
1. Request Access    → App requests permission to access BodhiApp resources
2. OAuth Flow        → User authorizes app with resource-specific consent  
3. API Calls         → App calls BodhiApp APIs with user token
4. Token Exchange    → BodhiApp exchanges app token for resource token (internal)
```

## Step 1: Request Access

Before users can authorize your app, you must request access to the BodhiApp resource server. This is a one-time setup per BodhiApp instance.

### Endpoint
```http
POST /bodhi/v1/auth/request-access
Content-Type: application/json
```

### Request Format
```typescript
const response = await fetch('http://localhost:1135/bodhi/v1/auth/request-access', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    app_client_id: 'app-your-client-id'
  })
});
```
**Response JSON**:
```json
{
  "scope": "scope_resource-bodhi-server-abc123"
}
```

### Implementation Notes
- **Cache the Response**: Store the returned scope value for use in OAuth flows
- **One-Time Operation**: Only needs to be called once per BodhiApp instance
- **Idempotent**: Safe to call multiple times; returns same scope if already granted
- **No Authentication**: This endpoint doesn't require authentication

## Step 2: OAuth Authorization Flow

After obtaining the resource scope, initiate the OAuth flow to get user authorization with the appropriate permissions.

### Authorization Request

#### Endpoint
```http
GET /realms/bodhi/protocol/openid-connect/auth
```

#### Build Authorization URL
```typescript
function buildAuthUrl(
  authServerUrl: string,
  clientId: string, 
  redirectUri: string,
  resourceScope: string,
  userAccessLevel: 'scope_user_user' | 'scope_user_power_user'
): string {
  const scopes = [
    'openid',
    'profile', 
    'email',
    userAccessLevel,    // User permission level
    resourceScope       // Resource access from step 1
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

### User Consent Screen

Users will see a consent screen showing:
- Your app name and description
- Requested permissions (user access level)
- BodhiApp resource access request
- Option to approve or deny access

### Token Exchange

#### Endpoint
```http
POST /realms/bodhi/protocol/openid-connect/token
Content-Type: application/x-www-form-urlencoded
```

#### Request Format
```typescript
const tokenResponse = await fetch(`${authServerUrl}/realms/bodhi/protocol/openid-connect/token`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/x-www-form-urlencoded'
  },
  body: new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: clientId,
    code: authorizationCode,
    redirect_uri: redirectUri,
    code_verifier: pkceVerifier
  })
});
```

**Response JSON**:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "scope": "openid profile email scope_user_power_user scope_resource-bodhi-server-abc123"
}
```

### Scope Requirements

The user token must include:

- **User Scope**: Either `scope_user_user` or `scope_user_power_user`
  - `scope_user_user`: Basic read access to BodhiApp APIs
  - `scope_user_power_user`: Enhanced access including model management
- **Resource Scope**: The scope returned from the request-access call
- **Standard Scopes**: `openid`, `profile`, `email` for user information

## Step 3: API Calls

With the user token, you can now call BodhiApp APIs. The token exchange happens automatically within BodhiApp's authentication middleware.

### API Request Format
```typescript
// Make authenticated API calls
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

### Available API Endpoints

The user's access level determines which endpoints are available. For detailed endpoint documentation, see the [Authentication](authentication.md) and [BodhiApp API](bodhi-api.md) guides.

## Step 4: Token Exchange (Internal)

This step happens automatically within BodhiApp when it receives your API request. Understanding this process helps with troubleshooting.

### Process Flow
1. **Token Validation**: BodhiApp validates the incoming user token
2. **Audience Check**: Verifies BodhiApp's client ID is in the token audience
3. **Token Exchange**: Exchanges app token for BodhiApp-scoped token using RFC 8693
4. **Permission Check**: Validates user permissions against required access level
5. **API Processing**: Processes the API request with the exchanged token

## Complete Implementation Example

Here's a complete TypeScript implementation of the OAuth flow:

```typescript
class BodhiAppIntegration {
  constructor(
    private appClientId: string,
    private authServerUrl: string = 'https://id.getbodhi.app',
    private realm: string = 'bodhi'
  ) {}

  // Step 1: Request access to BodhiApp instance
  async requestAccess(bodhiAppUrl: string): Promise<string> {
    const response = await fetch(`${bodhiAppUrl}/bodhi/v1/auth/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ app_client_id: this.appClientId })
    });

    if (!response.ok) {
      throw new Error(`Request access failed: ${response.status}`);
    }

    const result = await response.json();
    return result.scope; // Cache this value
  }

  // Step 2: Build OAuth authorization URL
  buildAuthUrl(
    redirectUri: string,
    resourceScope: string,
    userAccessLevel: 'scope_user_user' | 'scope_user_power_user' = 'scope_user_user'
  ): string {
    const scopes = [
      'openid',
      'profile',
      'email', 
      userAccessLevel,
      resourceScope
    ];

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

  // Step 2: Exchange authorization code for tokens
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

  // Step 3: Make API calls to BodhiApp
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

// 1. Request access (one-time setup)
const resourceScope = await integration.requestAccess('http://localhost:1135');

// 2. Start OAuth flow
const authUrl = integration.buildAuthUrl(
  'https://yourapp.com/callback',
  resourceScope,
  'scope_user_power_user'
);

// Redirect user to authUrl...

// 3. After callback, exchange code for tokens
const tokens = await integration.exchangeCodeForTokens(
  authCode,
  'https://yourapp.com/callback',
  pkceVerifier
);

// 4. Make API calls
const models = await integration.callBodhiAPI(
  'http://localhost:1135',
  '/v1/models',
  tokens.access_token
);
```

## Error Handling

### Common Error Scenarios

#### 1. Request Access Errors
- **400 Bad Request**: Invalid app client ID or request format
- **500 Internal Server Error**: BodhiApp configuration issues

#### 2. OAuth Flow Errors
- **invalid_request**: Missing or invalid OAuth parameters
- **access_denied**: User denied authorization
- **invalid_client**: App client not found or misconfigured

#### 3. API Call Errors
- **401 Unauthorized**: Token invalid, expired, or missing audience
- **403 Forbidden**: Insufficient user permissions for endpoint
- **Token exchange failed**: Audience validation or token exchange issues

### Error Handling Implementation

```typescript
// Basic error handling pattern
try {
  const result = await integration.callBodhiAPI(url, endpoint, token);
} catch (error) {
  if (error.status === 401 && error.message.includes('audience')) {
    console.error('Token not valid for this BodhiApp instance');
    // Re-run request access flow
  } else if (error.status === 403) {
    console.error('Insufficient permissions');
    // Request higher access level
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
- **State Validation**: Validate state parameter to prevent CSRF attacks
- **Redirect URI Validation**: Ensure redirect URIs are properly configured

### Scope Management
- **Minimum Scope**: Request only the minimum required user access level
- **Scope Validation**: Verify granted scopes match requested scopes
- **Permission Checks**: Handle cases where users have insufficient permissions

## Testing and Validation

### Verify Integration Steps

1. **Request Access Validation**
   ```typescript
   const scope = await requestAccess(bodhiAppUrl);
   console.assert(scope.startsWith('scope_resource-'), 'Invalid scope format');
   ```

2. **API Access Validation**
   ```typescript
   // Test token validity by getting user info
   const userInfo = await callBodhiAPI('/bodhi/v1/user', token);
   console.assert(userInfo.logged_in === true, 'Token validation failed');
   ```

### Common Integration Issues

- **Missing Audience**: Token doesn't include BodhiApp client ID - ensure request-access was called
- **Scope Mismatch**: User scope not included in token - verify OAuth scope parameter
- **Permission Denied**: User lacks required role - check user's role in BodhiApp
- **Token Exchange Failed**: Invalid issuer or expired token - validate token before sending

## Best Practices

### Implementation
- **Cache Request-Access Results**: Store resource scopes per BodhiApp instance
- **Token Refresh**: Implement automatic token refresh using refresh tokens
- **Error Recovery**: Provide clear error messages and recovery paths for users
- **Rate Limiting**: Respect API rate limits and implement backoff strategies

### User Experience
- **Clear Consent**: Explain why your app needs access to BodhiApp resources
- **Permission Levels**: Let users choose their access level (user vs power user)
- **Connection Management**: Allow users to view and revoke app connections
- **Offline Handling**: Gracefully handle cases when BodhiApp is unavailable

## Next Steps

After implementing the OAuth flow:

1. **[Explore BodhiApp APIs](bodhi-api.md)** - Learn about available endpoints and capabilities
2. **[Handle Errors](error-handling.md)** - Comprehensive error handling strategies
3. **[Model Management](model-management.md)** - Advanced model workflows for power users
4. **[Examples](examples.md)** - Complete integration examples and patterns

---

*This guide provides the complete OAuth2 token exchange flow for secure app-to-BodhiApp integration. The token exchange mechanism ensures proper user consent and scope-based access control while maintaining standard OAuth2 compliance.* 