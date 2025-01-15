# API Token Management Feature

## User Story
As a User of Bodhi App  
I want to generate and manage API tokens  
So that I can access the application programmatically from external applications

## Background
- Users need programmatic access to Bodhi App via API tokens
- Tokens are generated using OAuth2 token exchange with offline_access and scope_token_user scopes
- Offline tokens do not expire but have an idle timeout of 30 days
- Tokens are stateless and user session-independent
- App must be in authenticated mode to generate tokens, authz: true
- All tokens are created with user scope by default, scope: scope_token_user

## Token Characteristics
- Offline tokens can be used repeatedly to obtain new access tokens
- Tokens remain valid even when user is logged out
- Tokens must be used at least once every 30 days to prevent idle timeout
- Tokens are scoped to user-level access only via scope_token_user

## Acceptance Criteria

### Navigation & Access
- [x] Add "API Tokens" menu item in the navigation
- [x] Show message "Non-authenticated setup don't need API Tokens. Either ignore the Auth header or pass an empty/random Bearer token. They are not validated." when app is in non-authenticated mode
- [x] Only show API tokens page when user is authenticated and authz: true

### Token Generation Form
- [x] Form with input field for token name
- [x] Create button to generate new token
- [x] Dialog to display newly generated token with copy functionality
- [x] Clear warning that token will not be shown again
- [x] Warning that tokens must be used at least once every 30 days

### Token Management Table
- [ ] update the auth backend to latest version allowing for offline token delete
- [ ] Display table of user's API tokens with columns:
  - Token ID
  - Name (editable)
  - Status (active/invalid)
  - Created Date
  - Last Used Date
  - Actions (invalidate)
- [ ] Visual warning for tokens inactive for 25+ days
- [ ] Confirmation dialog before invalidating tokens

### Backend Implementation
- [ ] Create database table `api_tokens` with fields:
  - id (primary key)
  - user_id (foreign key)
  - token_id (jti)
  - name
  - status
  - create_date
  - last_used_date
  - created_at
  - updated_at
- [ ] API endpoint to create new token:
  - Exchange user's access token for offline token using token exchange
  - Store token jti and metadata in database
  - Cache token permissions against jti
- [ ] API endpoint to list user's tokens
- [ ] API endpoint to update token name
- [ ] API endpoint to invalidate token

## Progress Update (2025-01-13)

### Completed Features

#### Navigation & Access 
- [x] Add "API Tokens" menu item in the navigation
- [x] Show message "Non-authenticated setup don't need API Tokens..." in non-authenticated mode
- [x] Only show API tokens page when user is authenticated

#### Token Generation Form 
- [x] Form with input field for token name
- [x] Create button to generate new token
- [x] Dialog to display newly generated token with copy functionality
- [x] Clear warning that token will not be shown again
- [x] Warning that tokens must be used at least once every 30 days

#### Frontend Implementation Details
1. Created token management UI components:
   - `page.tsx`: Main token page with loading states and authentication checks
   - `TokenForm.tsx`: Form for token creation with validation
   - `TokenDialog.tsx`: Modal for displaying new tokens with copy/show/hide functionality
   - Tests for all components with MSW for API mocking

2. Added token-related hooks:
   - `useCreateToken`: Hook for token creation with error handling
   - Tests for hooks with proper error cases and network conditions

3. UI/UX Features:
   - Loading states with skeletons
   - Error handling with toast notifications
   - Security warnings and instructions
   - Copy to clipboard functionality
   - Show/hide token toggle
   - Responsive layout with cards

### Next Steps

#### Backend Implementation (Pending)
- [ ] Create database table `api_tokens`
- [ ] Implement token storage and retrieval
- [ ] Add token validation and expiry checks

#### Token Management Table (Pending)
- [ ] Implement token listing UI
- [ ] Add token status management
- [ ] Create token invalidation flow

#### Testing & Documentation
- [ ] Add integration tests for token flow
- [ ] Document API endpoints
- [ ] Add user documentation for token management

## Progress Update (2025-01-15)

### Database Implementation Progress

1. Created `api_tokens` table with schema:
   ```sql
   CREATE TABLE api_tokens (
       id TEXT PRIMARY KEY NOT NULL,
       user_id TEXT NOT NULL,
       name TEXT DEFAULT '',
       token_id TEXT NOT NULL UNIQUE,
       status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
       created_at INTEGER NOT NULL,
       updated_at INTEGER NOT NULL
   );
   ```

2. Added Data Models:
   - `TokenStatus` enum with Active/Inactive variants using serde for serialization
   - `ApiToken` struct with all required fields
   - Implemented PartialEq for comparison in tests
   - Added string conversion methods using kebab-case format

3. Database Service Implementation:
   - Extended DbService trait with token operations:
     - create_api_token: Creates new API tokens
     - list_api_tokens: Lists tokens with pagination
   - Added comprehensive tests for token creation and listing
   - Implemented test notification system for tracking DB operations

### Phase-wise Implementation Plan

#### Phase 1: Token Management Backend (Current)
1. Database Layer 
   - API tokens table with schema 
   - TokenStatus enum and ApiToken struct 
   - Basic CRUD operations 

2. Service Layer Extensions
   - Add update_token_status method
   - Add update_token_name method
   - Add total count to list_api_tokens response
   - Add get_token_by_id method for validation

3. API Endpoints
   - GET /api/tokens?page=1&per_page=10
   - PUT /api/tokens/:id/status
   - PUT /api/tokens/:id/name
   - Response DTOs with token metadata

#### Phase 2: Token Management UI
1. Token List Component
   - Table with columns: Name, Status, Created Date, Actions
   - Pagination controls
   - Loading states and error handling
   - Empty state design

2. Token Actions
   - Edit name with inline editing
   - Status toggle with confirmation
   - Copy token ID functionality
   - Action tooltips and feedback

3. Token Status Visualization
   - Status badges (Active/Inactive)
   - Created date formatting
   - Visual indicators for token age
   - Responsive design for mobile

#### Phase 3: Auth Middleware Enhancement
1. Token Validation Updates
   - Cache token status in Redis/memory
   - Check token status during validation
   - Add token status to validation response
   - Log token validation attempts

2. Token Deactivation Flow
   - Update cache on token status change
   - Invalidate existing sessions
   - Clear cached permissions
   - Audit logging for status changes

3. Error Handling
   - Specific error for inactive tokens
   - Clear error messages for UI
   - Logging for debugging
   - Metrics for monitoring

#### Phase 4: Testing & Documentation
1. Backend Testing
   - Integration tests for token flow
   - Cache behavior tests
   - Performance testing
   - Error scenario coverage

2. Frontend Testing
   - Component tests
   - User interaction tests
   - Error handling tests
   - Visual regression tests

3. Documentation
   - API documentation updates
   - UI component documentation
   - Token lifecycle documentation
   - Deployment notes

### Current Sprint Focus
- Implementing token status update endpoint
- Adding token name update functionality
- Preparing UI components for token listing
- Setting up token validation caching

### Next Implementation Tasks
1. Add token status update functionality
2. Implement token name update method
3. Add token invalidation endpoint
4. Integrate with token validation service

## Technical Implementation Steps

### Database Changes
1. Create new migration for `api_tokens` table
2. Add token entity and repository in `crates/db`
3. Add token service in `crates/services`

### Backend API Changes
1. Add token routes in `routes_token.rs`:
   - POST /api/tokens/create
   - GET /api/tokens
   - PUT /api/tokens/:id/name
   - POST /api/tokens/:id/invalidate
2. Implement token exchange in auth_service for offline token generation
3. Implement token service methods
4. Add cache for token permissions

### Frontend Changes
1. Add API token page in `crates/bodhi/src/app/ui/tokens`:
   - page.tsx for main layout
   - components/TokenForm.tsx for token creation
   - components/TokenTable.tsx for listing tokens
   - components/TokenDialog.tsx for displaying new token
2. Add token-related hooks in `crates/bodhi/src/hooks`:
   - useCreateToken
   - useTokens
   - useUpdateToken
   - useInvalidateToken
3. Add token types in `crates/bodhi/src/types`
4. Update navigation to include API tokens menu item

## Token Validation & Middleware Implementation

#### Auth Middleware Enhancement
- [ ] Modify `auth_middleware` to handle both session and token-based authentication:
  1. Check for Authorization header first
  2. If header exists, validate using Bearer token
  3. If no header, fallback to session validation
  4. Return invalid access if both validations fail

#### Token Validation Process
- [ ] Validate offline tokens with following checks:
  - Token type is "Offline"
  - Authorized party (azp) matches client_id
  - Scope includes "scope_token_user"
  - Signature verification using public key
- [ ] Exchange valid offline token for access token
- [ ] Inject new access token in request header
- [ ] Simplified error handling:
  - Return generic "auth token validation failed" message
  - Log detailed validation failures (kid mismatch, algorithm mismatch, etc.) at WARN level

#### Token Caching
- [ ] Implement token caching mechanism:
  - Cache validated tokens using JTI and token hash
  - Store corresponding access tokens with expiration
  - Return cached access token if valid
  - Refresh and cache new access token if expired/missing

#### Implementation Components
- [ ] Update `auth_middleware.rs`:
  - Add token validation logic
  - Implement caching mechanism
  - Update error handling
- [ ] Enhance `token_service.rs`:
  - Add token validation methods
  - Implement caching interface
  - Update service facade pattern
- [ ] Add comprehensive tests for:
  - Token validation scenarios
  - Caching behavior
  - Error cases
  - Integration with auth service

## Token Caching Story

## Overview
Implement token caching to optimize token validation and reduce unnecessary calls to the auth service.

## Implementation Details

### Token Caching Strategy
- Cache key format: `token:<jti>:<token_hash_prefix>`
  - `jti`: Unique token identifier from claims
  - `token_hash_prefix`: First 12 chars of SHA256 hash of the token
- Cache value: JSON serialized `CachedToken` containing:
  - `access_token`: The exchanged access token
  - `exp`: Token expiration timestamp

### Optimization Flow
1. Extract token from authorization header
2. Get `jti` and calculate token hash without full validation
3. Check cache using composite key
4. If cache hit and token not expired:
   - Return cached access token
5. If cache miss or token expired:
   - Validate token signature and claims
   - Exchange token with auth service
   - Cache new access token with expiry
   - Return new access token

### Security Considerations
- Token hash in cache key ensures cache miss if token is tampered
- Expiry check prevents use of expired tokens
- No sensitive data stored in cache besides access token
- Cache automatically invalidates on token changes

### Code Changes
- Added `TokenCache` struct to encapsulate caching logic
- Updated `DefaultTokenService` to use token cache
- Added tests for cache hit and miss scenarios
- Integrated with existing auth middleware

### Performance Impact
- Reduces auth service calls for valid tokens
- Minimal overhead for cache misses
- Efficient key lookup using token hash

## Status
- [x] Design caching strategy
- [x] Implement token caching
- [x] Add cache hit/miss tests
- [x] Update auth middleware
- [x] Code review changes
- [ ] Documentation

## Not In Scope
- Custom token scopes
- Role-based token access control
- Custom claims in token
- Rate limiting for token generation
- Token usage tracking
- Token audit logging
- Password confirmation for token generation
- Token search/filtering

## Follow-up Stories
1. Add custom claims to tokens for:
   - Role-based access control
   - Model-specific access restrictions
   - API endpoint restrictions
2. Add token usage tracking
3. Implement rate limiting
4. Add audit logging for token operations

## Technical Considerations
- Tokens are generated through OAuth2 token exchange
- Token invalidation only marks tokens as invalid in database
- Token names can be updated after creation
- Visual warning for tokens inactive for 25+ days
- No limit on number of active tokens per user
- All tokens are created with user scope via scope_token_user
- Token permissions are cached against jti for performance

## Testing Requirements
- [ ] Unit tests for token service
- [ ] API endpoint tests
- [ ] UI component tests
- [ ] Integration tests for token flow
- [ ] Test token invalidation
- [ ] Test non-authenticated mode message
- [ ] Test inactivity warnings
