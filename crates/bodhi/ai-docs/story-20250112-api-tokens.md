# API Token Management Feature

## User Story
As a User of Bodhi App  
I want to generate and manage API tokens  
So that I can access the application programmatically from external applications

## Background
- Users need programmatic access to Bodhi App via API tokens
- Tokens are micro-JWT tokens signed by the auth server
- Tokens have a default validity of 30 days
- App must be in authenticated mode to generate tokens
- All tokens are created with user scope by default

## Acceptance Criteria

### Navigation & Access
- [ ] Add "API Tokens" menu item in the navigation
- [ ] Show message "Cannot create Auth Tokens if app setup in non-authenticated mode" when app is in non-authenticated mode
- [ ] Only show API tokens page when user is authenticated

### Token Generation Form
- [ ] Form with input field for token name
- [ ] Create button to generate new token
- [ ] Dialog to display newly generated token with copy functionality
- [ ] Clear warning that token will not be shown again

### Token Management Table
- [ ] Display table of user's API tokens with columns:
  - Token ID
  - Name (editable)
  - Status (active/invalid)
  - Created Date
  - Expiry Date
  - Actions (invalidate)
- [ ] Visual warning for tokens expiring in next 3 days
- [ ] Confirmation dialog before invalidating tokens

### Backend Implementation
- [ ] Create database table `api_tokens` with fields:
  - id (primary key)
  - user_id (foreign key)
  - token_id (unique identifier)
  - name
  - status
  - create_date
  - expires_at
  - created_at
  - updated_at
- [ ] API endpoint to create new token:
  - Generate token_id
  - Request auth server to create JWT with claims:
    - sub: user_id
    - aud: resource server
    - iss: id server
    - exp: 30 days from now
    - scope: "user"
  - Store token metadata in database
- [ ] API endpoint to list user's tokens
- [ ] API endpoint to update token name
- [ ] API endpoint to invalidate token

## Technical Implementation Steps

### Database Changes
1. Create new migration for `api_tokens` table
2. Add token entity and repository in `crates/db`
3. Add token service in `crates/services`

### Backend API Changes
1. Add token routes in `crates/routes_app`:
   - POST /api/tokens/create
   - GET /api/tokens
   - PUT /api/tokens/:id/name
   - POST /api/tokens/:id/invalidate
2. Add auth server client method to sign tokens
3. Implement token service methods

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
- Custom token expiry
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
2. Allow custom token expiry
3. Add token usage tracking
4. Implement rate limiting
5. Add audit logging for token operations

## Technical Considerations
- JWT tokens will be signed by auth server using client credentials
- Token invalidation only marks tokens as invalid in database
- Token names can be updated after creation
- Visual warning for tokens expiring in 3 days
- No limit on number of active tokens per user
- All tokens are created with user scope, allowing access to all non-admin APIs

## Testing Requirements
- [ ] Unit tests for token service
- [ ] API endpoint tests
- [ ] UI component tests
- [ ] Integration tests for token flow
- [ ] Test token invalidation
- [ ] Test non-authenticated mode message
- [ ] Test expiry warnings
