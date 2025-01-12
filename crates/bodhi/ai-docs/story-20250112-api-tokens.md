# API Token Management Feature

## User Story
As a User of Bodhi App  
I want to generate and manage API tokens  
So that I can access the application programmatically from external applications

## Background
- Users need programmatic access to Bodhi App via API tokens
- Tokens are generated using OAuth2 token exchange with offline_access and scope_token_user scopes
- Offline tokens do not expire but have an idle timeout of 30 days
- Tokens are stateless and session-independent
- App must be in authenticated mode to generate tokens
- All tokens are created with user scope by default

## Token Characteristics
- Offline tokens can be used repeatedly to obtain new access tokens
- Tokens remain valid even when user is logged out
- Tokens must be used at least once every 30 days to prevent idle timeout
- Tokens are scoped to user-level access only via scope_token_user

## Acceptance Criteria

### Navigation & Access
- [ ] Add "API Tokens" menu item in the navigation
- [ ] Show message "Non-authenticated setup don't need API Tokens. Either ignore the Auth header or pass an empty/random Bearer token. They are not validated." when app is in non-authenticated mode
- [ ] Only show API tokens page when user is authenticated

### Token Generation Form
- [ ] Form with input field for token name
- [ ] Create button to generate new token
- [ ] Dialog to display newly generated token with copy functionality
- [ ] Clear warning that token will not be shown again
- [ ] Warning that tokens must be used at least once every 30 days

### Token Management Table
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
