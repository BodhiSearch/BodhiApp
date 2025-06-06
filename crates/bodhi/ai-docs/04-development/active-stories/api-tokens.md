# API Token Management Feature

## Requirements

### User Story
As a User of Bodhi App  
I want to generate and manage API tokens  
So that I can access the application programmatically from external applications

### Core Requirements

#### Token Characteristics
- Tokens are generated using OAuth2 token exchange with offline_access and scope_token_user scopes
- Offline tokens do not expire but have an idle timeout of 30 days
- Tokens are stateless and user session-independent
- Tokens must be used at least once every 30 days to prevent idle timeout
- All tokens are created with user scope by default (scope_token_user)
- Tokens remain valid even when user is logged out
- App must be in authenticated mode to generate tokens (authz: true)
- Tokens can be exchanged for new access tokens repeatedly

#### Security Requirements
- Tokens are generated through OAuth2 token exchange only
- Token invalidation only marks tokens as invalid in database
- No limit on number of active tokens per user
- Token permissions are cached against jti for performance
- Token names can be updated after creation
- Token validation includes signature and claims verification
- Authorized party (azp) must match client_id
- Token hash verification required for additional security

#### User Interface Requirements
1. Navigation & Access
   - API Tokens menu item in navigation
   - Show message in non-authenticated mode: "Non-authenticated setup don't need API Tokens. Either ignore the Auth header or pass an empty/random Bearer token. They are not validated." [ADDED EXACT MESSAGE]
   - Only show API tokens page when user is authenticated

2. Token Generation
   - Form with input field for token name
   - Create button to generate new token
   - Dialog to display newly generated token
   - Copy functionality for new tokens
   - Clear warning that token will not be shown again
   - Warning about 30-day usage requirement
   - Success/error feedback for token creation

3. Token Management
   - Table display with columns: Name, Status, Created Date, Updated Date
   - Visual warning for tokens inactive for 25+ days
   - Sorting functionality
   - Confirmation dialog for token invalidation
   - Responsive design for all screen sizes

#### Database Schema
```sql
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT DEFAULT '',
    token_id TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX idx_api_tokens_token_id ON api_tokens(token_id);
```

#### Cache Management [ADDED SECTION]
- Cache key format: `token:<jti>:<token_hash_prefix>`
- Token hash prefix uses first 12 chars of SHA256 hash
- Cache value contains:
  - access_token: The exchanged access token
  - exp: Token expiration timestamp
- Automatic cache invalidation on token status changes

### Not In Scope
- Custom token scopes
- Role-based token access control
- Custom claims in token
- Rate limiting for token generation
- Token usage tracking
- Token audit logging
- Password confirmation for token generation
- Token search/filtering

## Implementation Tasks

### Backend Tasks

#### Database Layer
- [x] Create api_tokens table with schema
- [x] Add token status enum (active/inactive)
- [x] Add token hash for verification
- [x] Add index for faster token lookups
- [ ] ~Add last used timestamp~
- [ ] ~Add token usage counter~

#### Service Layer
- [x] Implement create_api_token method
- [x] Implement list_api_tokens with pagination
- [x] Add update_token method
- [x] Add get_token_by_id method
- [x] Add total count to list response

#### API Endpoints
- [x] POST /api/tokens
- [x] GET /api/tokens?page=1&per_page=10
- [x] PUT /api/tokens/:id

#### Token Validation & Middleware
- [x] Check token exists in database
- [x] Verify token hash
- [x] Check token status is active
- [x] Validate token signature and claims
- [x] Check access token expiry
- [x] Auto refresh expired access tokens
- [ ] ~Track token usage~
- [x] Add comprehensive error logging

#### Caching Implementation
- [x] Cache access tokens by token_id
- [x] Validate cached token expiry
- [x] Invalidate cache on token deactivation
- [x] Clean up expired cache entries
- [ ] ~Implement cache monitoring metrics~

### Frontend Tasks

#### Token List Component
- [x] Table with columns: Name, Status, Created Date, Updated Date
- [x] Pagination controls
- [x] Loading states and error handling
- [x] Empty state design
- [x] Status badges (Active/Inactive)
- [x] Created/Updated date formatting
- [ ] ~Visual indicators for token age~
- [x] Responsive design for mobile
- [x] Add skeleton loading state

#### Token Actions
- [ ] ~Edit name with inline editing~
- [x] Status toggle
- [x] Copy token ID functionality
- [x] Action tooltips and feedback
- [x] Toast notifications for actions

#### React Query Integration
- [x] Combined useCreateToken and useListTokens into useApiTokens
- [x] Added cache invalidation after token creation
- [x] Proper error handling and loading states
- [x] Type-safe API responses

### Testing Tasks
- [x] Unit tests for token service
- [x] API endpoint tests
- [x] UI component tests
- [x] Integration tests for token flow
- [x] Test token invalidation
- [x] Test non-authenticated mode message
- [ ] ~Test inactivity warnings~
- [ ] ~Performance tests for token validation~
- [ ] ~Cache behavior tests~

## File Overview

### Frontend (React/TypeScript)
- `crates/bodhi/src/app/ui/tokens/page.tsx`: Main token management page component with token list and form
- `crates/bodhi/src/app/ui/tokens/TokenForm.tsx`: Form component for creating new API tokens
- `crates/bodhi/src/app/ui/tokens/TokenDialog.tsx`: Dialog component for displaying newly created tokens
- `crates/bodhi/src/hooks/useApiTokens.ts`: React Query hooks for token creation and listing
- `crates/bodhi/src/components/navigation/AppNavigation.tsx`: Navigation component with API tokens menu item

### Backend (Rust)
- `crates/routes_app/src/routes_api_token.rs`: API endpoints for token management (create, list)
- `crates/services/src/db/service.rs`: Database service implementation for token operations
- `crates/services/src/auth_service.rs`: Token exchange and validation service
- `crates/auth_middleware/src/token_service.rs`: Token validation and caching middleware
- `crates/routes_all/src/routes.rs`: Route registration for token endpoints

### Database
- `crates/services/migrations/0004_create_api_tokens.up.sql`: Migration for creating api_tokens table
- `crates/services/src/db/objs.rs`: Database models and types for API tokens

### Tests
- `crates/bodhi/src/app/ui/tokens/*.test.tsx`: Frontend component tests
- `crates/bodhi/src/hooks/useApiTokens.test.ts`: Hook tests with MSW
- `crates/auth_middleware/tests/test_live_auth_middleware.rs`: Token middleware tests
- `crates/services/src/test_utils/db.rs`: Test utilities for database operations
- `crates/services/src/test_utils/auth.rs`: Test utilities for auth operations

### Documentation
- `crates/bodhi/ai-docs/story-20250112-api-tokens.md`: Main story and implementation details
- `crates/auth_middleware/src/resources/en-US/messages.ftl`: Error messages for token validation
- `crates/routes_app/src/resources/en-US/messages.ftl`: API endpoint error messages
- `crates/services/src/resources/en-US/messages.ftl`: Service layer error messages

## Recommendations for Implementation
1. Start with implementing core token validation before UI features
2. Implement caching early to ensure good performance
3. Consider adding monitoring for:
   - Token creation rate
   - Token usage patterns
   - Cache hit/miss ratio
   - Token invalidation events
4. Plan for future scaling:
   - Consider token bucket implementation for rate limiting
   - Plan database partitioning strategy
   - Design audit logging structure

## Migration Plan
1. Deploy database changes first
2. Add new endpoints with feature flag
3. Deploy UI changes
4. Enable feature for beta testing
5. Roll out to all users
6. Monitor for issues

## Progress Update (2025-01-13)

### Completed Tasks

#### Backend Tasks
- [x] Create api_tokens table with schema
- [x] Add token status enum and token hash
- [x] Add index for faster token lookups
- [x] Implement token storage and retrieval
- [x] Add token validation and expiry checks

#### Frontend Tasks
1. Token List Component:
   - [x] Table with columns: Name, Status, Created Date, Updated Date
   - [x] Pagination controls
   - [x] Loading states and error handling
   - [x] Status badges (Active/Inactive)
   - [x] Created/Updated date formatting
   - [x] Responsive design for mobile

2. React Query Integration:
   - [x] Combined useCreateToken and useListTokens into useApiTokens
   - [x] Added cache invalidation after token creation
   - [x] Proper error handling and loading states
   - [x] Type-safe API responses

### Next Steps
1. Complete token management UI features
2. Implement token invalidation flow
3. Add sorting functionality
4. Add user documentation

## Progress Update (2025-01-15)

### Completed Tasks

#### Backend Tasks
1. Database Service:
   - [x] Extended DbService trait with token operations
   - [x] Implemented create_api_token and list_api_tokens
   - [x] Added comprehensive test coverage
   - [x] Added test notification system

2. Token Validation:
   - [x] Implemented token hash verification
   - [x] Added token status validation
   - [x] Set up token expiry checks

#### Frontend Tasks
1. Token List Enhancements:
   - [x] Implemented DataTable with pagination
   - [x] Added status badge component
   - [x] Improved date formatting
   - [x] Enhanced responsive layout

2. Testing Infrastructure:
   - [x] Updated API endpoint constants
   - [x] Added MSW handlers for token listing
   - [x] Added pagination test cases
   - [x] Verified cache invalidation

### Next Steps
1. Implement token name updates
2. Add token invalidation endpoint
3. Enhance error handling
4. Complete cache monitoring

## Progress Update (2025-01-16)

### Documentation Improvements
1. Requirements Clarification:
   - [x] Added token exchange capability details
   - [x] Expanded security requirements with signature and claims verification
   - [x] Added exact non-authenticated mode message
   - [x] Added success/error feedback requirement for token creation
   - [x] Specified responsive design requirement

2. Technical Documentation:
   - [x] Added detailed Cache Management section
   - [x] Added Implementation Recommendations section
   - [x] Added Migration Plan section

### New Tasks Added
1. Backend:
   - Add comprehensive error logging
   - Implement cache monitoring metrics

2. Frontend:
   - Add skeleton loading state
   - Add toast notifications for actions

3. Testing:
   - Performance tests for token validation
   - Cache behavior tests

### Next Focus Areas
1. Implement core token validation
2. Set up caching infrastructure
3. Add monitoring metrics
4. Begin UI development with skeleton loading
