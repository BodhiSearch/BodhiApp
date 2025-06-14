# SPA OAuth 2.0 Authorization Code Flow with PKCE

## Overview
Transform the current OAuth authentication flow from backend-managed redirects to a React SPA-managed OAuth 2.1 Authorization Code flow with PKCE (Proof Key for Code Exchange). This change enables the frontend to handle the complete OAuth flow while maintaining secure session-based authentication for SPA clients and supporting token-based authentication for third-party clients.

## Current vs. Desired Flow

### Current Flow (Backend-Managed) - DEPRECATED
```
1. Frontend → GET /app/login [DEPRECATED - now /bodhi/v1/auth/initiate]
2. Backend generates PKCE, stores in session → redirects to Keycloak
3. User authenticates with Keycloak
4. Keycloak → GET /app/login/callback?code=...&state=... [DEPRECATED - now /bodhi/v1/auth/callback]
5. Backend exchanges code for tokens → stores in session → redirects to frontend
```

### Desired Flow (SPA-Managed)
```
1. Frontend → POST /bodhi/v1/auth/initiate
2. Backend generates PKCE, state, stores in session → returns auth URL
3. Frontend redirects user to auth URL
4. User authenticates with Keycloak
5. Keycloak → Frontend callback (/ui/auth/callback)
6. Frontend → POST /bodhi/v1/auth/callback with all redirect query parameters
7. Backend validates parameters, exchanges code for tokens → stores in session → returns success
8. On error, backend forms i18n error and sends to frontend for display. Frontend provides mechanism to try login again.
```

## Core Features

### 1. SPA OAuth Flow Management

#### Frontend Capabilities
- Initiate OAuth flow via API call
- Handle OAuth callback in React Router
- Extract all query parameters from redirect URL
- Send all parameters to backend for validation
- Manage authentication state transitions
- Handle OAuth errors and edge cases
- Maintain PKCE security throughout flow

#### Backend API Endpoints
```
POST /bodhi/v1/auth/initiate
- Generate PKCE parameters and state
- Store code_verifier and state in session
- Return authorization URL

POST /bodhi/v1/auth/callback
- Accept all OAuth redirect parameters
- Validate known parameters (code, state, error, error_description)
- Handle additional OAuth 2.1 parameters dynamically
- Exchange authorization code for tokens
- Store tokens in session
- Return authentication status
```

### 2. PKCE Security Implementation

#### Code Verifier Generation
- Generate cryptographically secure 43-character code verifier
- Create SHA256 hash for code challenge
- Use S256 challenge method for maximum security
- Store verifier securely in backend session

#### State Parameter Security
- Generate cryptographically secure state parameter
- Store state in backend session during initiation
- Validate state parameter during callback processing
- Prevent CSRF attacks through state verification

#### Session Security
- Maintain PKCE parameters in secure session storage
- Clear sensitive parameters after successful exchange
- Implement session timeout for incomplete flows
- Protect against session fixation attacks

### 3. Client Type Differentiation

#### SPA Client Authentication (Session-Based)
- OAuth flow managed by React frontend
- Tokens stored in secure backend session
- Automatic token refresh via session middleware
- CSRF protection through session cookies
- Logout clears session and redirects appropriately

#### Third-Party Client Authentication (Token-Based)
- Direct API access with Bearer tokens
- Long-lived offline tokens for automation
- Scope-based permission management
- Token revocation capabilities
- API token management interface

## API Specifications

### POST /bodhi/v1/auth/initiate
**Purpose:** Start OAuth flow and return authorization URL

**Request:**
```json
{}
```

**Response:**
If requires login:
```
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer realm="OAuth"
Content-Type: application/json

{
  "auth_url": "https://id.getbodhi.app/realms/bodhi/protocol/openid-connect/auth?client_id=...",
}
```

If is already logged-in:
```
HTTP/1.1 303 See Other
Location: /ui/app/home
Content-Length: 0
```

**Security:**
- Generates and stores PKCE code_verifier in session
- Generates and stores state parameter in session
- Returns authorization URL

### POST /bodhi/v1/auth/callback
**Purpose:** Complete OAuth flow with all redirect parameters

**Request:**
Frontend sends all query parameters from OAuth redirect URL:
```json
{
  "code": "authorization-code-from-oauth-server",
  "state": "state-parameter-from-oauth-server", 
  "error": "optional-error-from-oauth-server",
  "error_description": "optional-error-description-from-oauth-server",
  // additional params received from the query are sent as is in body
  "session_state": "dynamic-session-state",
  "iss": "issuer-parameter",
  "custom_param": "any-other-oauth-parameter"
}
```

**Response:**
Success:
```
HTTP/1.1 303 See Other
Location: /ui/app/home
```
Error:
```
HTTP/1.1 422 Unprocessible Entity
Content-Type: application/json

{
  "error": {
    "message": "Error description",
    "type": "invalid_request_error", 
    "param": "parameter_name",
    "code": "error_code"
  }
}
```

**Security:**
- Validates state parameter against session
- Retrieves code_verifier from session
- Exchanges code for tokens using PKCE
- Stores tokens in session
- Clears PKCE parameters from session

## Security Considerations

### 1. PKCE Implementation
- **Code Verifier Security**: Generate cryptographically secure 43-character verifiers
- **Challenge Method**: Use S256 (SHA256) for code challenge generation
- **Storage Security**: Store verifiers in secure backend session, never expose to frontend
- **Cleanup**: Clear verifiers immediately after successful token exchange

### 2. State Parameter Protection
- **CSRF Prevention**: Generate cryptographically secure state parameters
- **Session Binding**: Store state in backend session tied to user session
- **Validation**: Strict state parameter validation during callback
- **Timeout**: Implement reasonable timeout for state parameter validity

### 3. Session Security
- **Secure Storage**: Store OAuth tokens in secure backend session
- **HttpOnly Cookies**: Use HttpOnly, Secure, SameSite cookie attributes
- **Session Rotation**: Rotate session IDs after successful authentication
- **Cleanup**: Clear incomplete OAuth flows after timeout

### 4. Redirect URI Validation
- **Protocol Enforcement**: Enforce HTTPS in production environments
- **Localhost Support**: Allow localhost for development environments

## Implementation Phases

### Phase 1: Backend API Development
1. Create OAuth initiation endpoint
2. Implement PKCE parameter generation
3. Create OAuth callback endpoint
4. Add state parameter validation
5. Update session management for OAuth flow

### Phase 2: Frontend Integration
1. Create OAuth service in React application
2. Implement OAuth flow initiation
3. Add React Router callback handling
4. Update authentication state management
5. Add error handling and user feedback

### Phase 3: Security Hardening
1. Implement comprehensive input validation
2. Add security event logging
3. Create OAuth flow monitoring
4. Add rate limiting for OAuth endpoints
5. Implement session cleanup mechanisms

## Testing Requirements

### Backend Testing
- Unit tests for PKCE generation and validation
- Integration tests for OAuth flow endpoints
- Security tests for state parameter validation
- Session management tests
- Error handling tests for invalid requests

### Frontend Testing
- Unit tests for OAuth service functions
- Integration tests for complete OAuth flow
- Error handling tests for OAuth failures
- Authentication state management tests
- React Router callback handling tests

### End-to-End Testing
- Complete OAuth flow from initiation to completion
- Error scenarios and recovery flows
- Session persistence across browser restarts
- Multiple concurrent OAuth flows
- Security attack simulation tests

## Documentation Updates Required

### Technical Documentation
- Update authentication architecture documentation
- Document new OAuth flow patterns
- Update API endpoint documentation
- Create security implementation guide
- Update deployment configuration guide

## Success Metrics

### Security Metrics
- Zero successful CSRF attacks on OAuth flow
- Zero code interception vulnerabilities
- 100% PKCE compliance for SPA clients
- Complete audit trail for all OAuth events

### User Experience Metrics
- Reduced authentication flow completion time
- Decreased user-reported authentication issues
- Improved error message clarity and actionability
- Maintained session persistence across browser sessions

### Technical Metrics
- Successful OAuth flow completion rate > 99%
- Average OAuth flow completion time < 10 seconds
- Zero session fixation vulnerabilities
- Complete test coverage for OAuth endpoints

# Pending

## Enhanced Parameter Handling Implementation

**Current Approach**: Frontend sends only known OAuth parameters (code, error, error_description) to backend.

**New Requirement**: 
- Frontend must extract and send **all** query parameters from OAuth redirect URL to backend
- Backend must handle both known OAuth 2.1 parameters and additional dynamic parameters
- State parameter validation will be implemented in backend (not exposed to frontend)

**Implementation Requirements**:
1. **Frontend**: Update `oauthUtils.extractOAuthParams` to capture all URL parameters
2. **Backend**: Update `AuthCallbackRequest` to accept known parameters and additional dynamic parameters
3. **Backend**: Implement state parameter generation in `auth_initiate_handler`
4. **Backend**: Implement state parameter validation in `auth_callback_handler`
