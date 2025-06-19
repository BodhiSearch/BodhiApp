# SPA OAuth 2.0 Authorization Code Flow with PKCE

## Overview
Transform the current OAuth authentication flow from backend-managed redirects to a React SPA-managed OAuth 2.1 Authorization Code flow with PKCE (Proof Key for Code Exchange). This change enables the frontend to handle the complete OAuth flow while maintaining secure session-based authentication for SPA clients and supporting token-based authentication for third-party clients.

## Current vs. Desired Flow

### Current Flow (Backend-Managed) - DEPRECATED
```
0. Frontend on /ui/setup/resource-admin or /ui/login
1. Frontend → GET /app/login
2. Backend generates PKCE, stores in session → redirects to Keycloak
3. User authenticates with Keycloak
4. Keycloak → GET /app/login/callback?code=...&state=...
5. Backend exchanges code for tokens → stores in session → redirects to frontend
```

### Desired Flow (SPA-Managed) - **CURRENT IMPLEMENTATION**
```
0. Frontend on /ui/setup/resource-admin/ or /ui/login/
1. Frontend on click of Login Button → POST /bodhi/v1/auth/initiate
2. Backend generates PKCE, state, stores in session → returns 200 with JSON body {"location": "..."}
3. Frontend performs window.location.href = location to navigate user to OAuth provider
4. User authenticates with Keycloak
5. Keycloak → Frontend callback (/ui/auth/callback?code=...)
6. Frontend → POST /bodhi/v1/auth/callback with all redirect query parameters
7. Backend validates parameters, exchanges code for tokens → stores in session → returns 200 with {"location": "..."}
8. Fronted does a router push (/ui/chat or /ui/setup/download-models)
9. On error, backend returns 422/500 with error details for frontend to display
10. Gives option to retry, where it does a POST /bodhi/v1/auth/initiate similar to /ui/login/ or /ui/setup/resource-admin/, and does the redirect
```

## Core Features

### 1. SPA OAuth Flow Management

#### Frontend Capabilities
- Initiate OAuth flow via API call, redirect to generated auth server url
- Handle OAuth callback
- Extract all query parameters from redirect URL
- Send all parameters to backend for validation and token exchange
- dumb frontend, with all validation and logic flows at backend

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
- Return authentication status and redirect to appropriate page based on app status
```

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
**Security:**
- Generates and stores PKCE code_verifier in session
- Generates and stores simple random state parameter (32 characters) in session
- Returns authorization URL in JSON response

**Request:**
```json
{}
```

**Response:**
```
HTTP/1.1 200 OK 
Content-Type: application/json

{
  "location": "https://id.getbodhi.app/realms/bodhi/protocol/openid-connect/auth?client_id=...",
}
```

If user is already logged-in:
```
HTTP/1.1 200 OK
Content-Type: application/json

{
  "location": "/ui/chat"
}
```
- Frontend on receiving full URL (https://) performs redirect using `window.location.href`
- On receiving relative path, performs router navigation using Next.js router

### POST /bodhi/v1/auth/callback
**Purpose:** Complete OAuth flow with all redirect parameters
**Security:**
- Validates state parameter against session (simple string comparison)
- Retrieves code_verifier from session
- Exchanges code for tokens using PKCE
- Stores tokens in session
- Clears state and PKCE parameters from session

**Request:**
Frontend sends all query parameters from OAuth redirect URL:
```json
{
  "code": "authorization-code-from-oauth-server",
  "state": "state-parameter-from-oauth-server", 
  "error": "optional-error-from-oauth-server",
  "error_description": "optional-error-description-from-oauth-server",
  "session_state": "dynamic-session-state",
  "iss": "issuer-parameter",
  "custom_param": "any-other-oauth-parameter"
  ...
}
```

**Response:**
Success:
```
HTTP/1.1 200 OK
Content-Type: application/json

{
  "location": "/ui/chat"
}
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

## Frontend Implementation

### OAuth Callback Page (`/ui/auth/callback/page.tsx`)
**Purpose:** Handle OAuth redirect callback and complete authentication flow
**Key Features:**
- Extract all query parameters from URL without Next.js `useSearchParams` (to avoid SSR issues)
- Send parameters to backend via `/bodhi/v1/auth/callback` endpoint
- Prevent duplicate requests using `useRef` pattern
- Handle success by redirecting to backend-provided location
- Display user-friendly error messages with retry option
- Use loading state during processing

**Implementation Requirements:**
- Use `window.location.search` and `URLSearchParams` for parameter extraction
- Implement duplicate request prevention with `hasProcessedRef`
- Handle both success and error states with appropriate UI
- Use `window.location.href` for external redirects, router for internal navigation

### State Management Hook Updates
**Updated `useOAuth.ts`:**
- Use standard `useMutationQuery` pattern from `useQuery.ts`
- Return JSON responses instead of handling redirects
- Include `skipCacheInvalidation: true` for auth endpoints
- Extract all URL parameters in callback function

## Security Considerations

### 1. PKCE Implementation
- **Code Verifier Security**: Generate cryptographically secure 43-character verifiers
- **Challenge Method**: Use S256 (SHA256) for code challenge generation
- **Storage Security**: Store verifiers in secure backend session, never expose to frontend
- **Cleanup**: Clear verifiers immediately after successful token exchange

### 2. State Parameter Protection
- **CSRF Prevention**: Generate cryptographically secure 32-character random state parameters
- **Session Binding**: Store state in backend session tied to user session
- **Validation**: Simple string comparison validation during callback (adequate for CSRF protection)
- **Timeout**: Implement reasonable timeout for state parameter validity

### 3. Session Security
- **Secure Storage**: Store OAuth tokens in secure backend session
- **HttpOnly Cookies**: Use HttpOnly, Secure, SameSite cookie attributes
- **Session Rotation**: Rotate session IDs after successful authentication
- **Cleanup**: Clear incomplete OAuth flows after timeout

### 4. Frontend Security
- **Duplicate Request Prevention**: Use React `useRef` to prevent double API calls
- **Parameter Extraction**: Extract all URL parameters to backend for validation
- **Error Handling**: Provide clear error messages without exposing sensitive information

### 5. Redirect URI Validation
- **Protocol Enforcement**: Enforce HTTPS in production environments
- **Localhost Support**: Allow localhost for development environments
- **JSON Response**: Use JSON responses instead of HTTP redirects to avoid CORS issues

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
