# SPA OAuth 2.0 Authorization Code Flow with PKCE (Pending Implementation)

## Overview
Transform the current OAuth authentication flow from backend-managed redirects to a React SPA-managed OAuth 2.0 Authorization Code flow with PKCE (Proof Key for Code Exchange). This change enables the frontend to handle the complete OAuth flow while maintaining secure session-based authentication for SPA clients and supporting token-based authentication for third-party clients.

## Current vs. Desired Flow

### Current Flow (Backend-Managed)
```
1. Frontend → GET /app/login
2. Backend generates PKCE, stores in session → redirects to Keycloak
3. User authenticates with Keycloak
4. Keycloak → GET /app/login/callback?code=...&state=...
5. Backend exchanges code for tokens → stores in session → redirects to frontend
```

### Desired Flow (SPA-Managed)
```
1. Frontend → POST /bodhi/v1/auth/initiate
2. Backend generates PKCE, stores in session → returns auth URL
3. Frontend redirects user to auth URL
4. User authenticates with Keycloak
5. Keycloak → Frontend callback (React Router)
6. Frontend → POST /bodhi/v1/auth/callback with code
7. Backend exchanges code for tokens → stores in session → returns success
```

## Core Features

### 1. SPA OAuth Flow Management

#### Frontend Capabilities
- Initiate OAuth flow via API call
- Handle OAuth callback in React Router
- Manage authentication state transitions
- Handle OAuth errors and edge cases
- Maintain PKCE security throughout flow

#### Backend API Endpoints
```
POST /bodhi/v1/auth/initiate
- Generate PKCE parameters
- Store code_verifier in session
- Return authorization URL and state

POST /bodhi/v1/auth/callback
- Validate state parameter
- Exchange authorization code for tokens
- Store tokens in session
- Return authentication status

GET /bodhi/v1/auth/status
- Return current authentication state
- Include user information if authenticated
```

### 2. PKCE Security Implementation

#### Code Verifier Generation
- Generate cryptographically secure 43-character code verifier
- Create SHA256 hash for code challenge
- Use S256 challenge method for maximum security
- Store verifier securely in backend session

#### State Parameter Validation
- Generate cryptographically secure state parameter
- Store state in backend session during initiation
- Validate state match during callback processing
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

## User Stories

### Story 1: SPA User Authentication Flow
**As a** web application user  
**I want to** authenticate through a seamless OAuth flow  
**So that** I can securely access the application without complex redirects

**Acceptance Criteria:**
- User clicks login button in React application
- Application initiates OAuth flow via backend API
- User is redirected to Keycloak for authentication
- After authentication, user returns to application
- Application completes OAuth flow via backend API
- User is authenticated and can access protected features
- Authentication state persists across browser sessions

### Story 2: OAuth Flow Error Handling
**As a** web application user  
**I want to** receive clear feedback when authentication fails  
**So that** I understand what went wrong and how to proceed

**Acceptance Criteria:**
- User receives clear error messages for authentication failures
- Application handles OAuth errors gracefully
- User can retry authentication after errors
- Application logs security events for monitoring
- Incomplete flows are cleaned up automatically

### Story 3: Secure Session Management
**As a** system administrator  
**I want to** ensure OAuth flows are secure against common attacks  
**So that** user authentication cannot be compromised

**Acceptance Criteria:**
- PKCE implementation prevents code interception attacks
- State parameter prevents CSRF attacks
- Session security prevents fixation attacks
- Sensitive parameters are cleared after use
- All security events are logged for audit

## API Specifications

### POST /bodhi/v1/auth/initiate
**Purpose:** Start OAuth flow and return authorization URL

**Request:**
```json
{
  "redirect_uri": "http://localhost:3000/auth/callback"
}
```

**Response:**
```json
{
  "authorization_url": "https://keycloak.example.com/auth/realms/bodhi/protocol/openid-connect/auth?...",
  "state": "generated-state-parameter"
}
```

**Security:**
- Validates redirect_uri against allowed patterns
- Generates and stores PKCE code_verifier in session
- Generates and stores state parameter in session
- Returns authorization URL with code_challenge

### POST /bodhi/v1/auth/callback
**Purpose:** Complete OAuth flow with authorization code

**Request:**
```json
{
  "code": "authorization-code-from-keycloak",
  "state": "state-parameter-from-initiate"
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "email": "user@example.com",
    "roles": ["user"]
  }
}
```

**Security:**
- Validates state parameter against session
- Retrieves code_verifier from session
- Exchanges code for tokens using PKCE
- Stores tokens in session
- Clears PKCE parameters from session

### GET /bodhi/v1/auth/status
**Purpose:** Check current authentication status

**Response (Authenticated):**
```json
{
  "authenticated": true,
  "user": {
    "email": "user@example.com",
    "roles": ["user"]
  }
}
```

**Response (Not Authenticated):**
```json
{
  "authenticated": false
}
```

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
- **Allowlist Validation**: Validate redirect URIs against configured allowlist
- **Exact Match**: Require exact URI matching, no wildcard patterns
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

### User Documentation
- Update login process documentation
- Create troubleshooting guide for OAuth issues
- Document browser compatibility requirements
- Update administrator setup guide
- Create security best practices guide

## Migration Considerations

### Backward Compatibility
- Maintain existing `/app/login` endpoint during transition
- Support both old and new flows during migration period
- Provide configuration flag to enable new flow
- Ensure existing sessions remain valid during migration

### Deployment Strategy
- Feature flag for new OAuth flow
- Gradual rollout to user segments
- Monitoring and rollback capabilities
- Database migration for session schema changes
- Configuration updates for OAuth settings

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

