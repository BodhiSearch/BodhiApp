# Migration Notes: Database-Backed API Tokens

## Overview
This migration transitions from Keycloak offline token exchange to database-backed API tokens for BodhiApp.

## Breaking Changes

### Database Schema
- **Migration File**: `crates/services/migrations/0003_create_api_tokens.up.sql`
- `api_tokens.token_id` renamed to `token_prefix`
- `api_tokens.scopes` field added (required TEXT field)
- Index updated from `idx_api_tokens_token_id` to `idx_api_tokens_token_prefix`

### Token Format
- **Old format**: JWT offline tokens from Keycloak with JTI field
- **New format**: `bodhiapp_<random_string>` where random_string is 32-byte cryptographically secure random encoded in URL-safe base64

### Token Storage
- **Token prefix**: First 8 characters after `bodhiapp_` stored for fast database lookup
- **Token hash**: SHA-256 hash of full token stored for validation
- **Scopes**: String field mapping to `TokenScope` enum (e.g., "scope_token_user")

### API Changes
- Token creation no longer accepts external JWT tokens
- Tokens are database-backed with status tracking (active/inactive)
- **SECURITY**: Tokens shown only once at creation - cannot be retrieved later

## Migration Strategy

Since no production data exists:
- Migration file modified directly (no data migration needed)
- Fresh start with new token format
- Old offline token code removed from codebase

## For Developers

### Generating Tokens
Use `POST /bodhi/v1/tokens` endpoint with session authentication.

Request:
```json
{
  "name": "My Integration Token"
}
```

Response:
```json
{
  "offline_token": "bodhiapp_<random_43_character_string>"
}
```

**IMPORTANT**: Save this token immediately - it cannot be retrieved later!

### Using Tokens
Include in Authorization header:
```bash
curl -H "Authorization: Bearer bodhiapp_..." https://api.example.com/v1/chat/completions
```

### Token Scopes
- `scope_token_user` - Standard user access
- `scope_token_power_user` - Power user access
- `scope_token_manager` - Manager access
- `scope_token_admin` - Admin access

Users can only create tokens at their own role level.

### Revoking Tokens
Update token status to 'inactive' via `PUT /bodhi/v1/tokens/{id}`:
```json
{
  "status": "inactive"
}
```

### Security Features
- **Constant-time hash comparison**: Prevents timing attacks
- **SHA-256 hashing**: Full token hash stored, never plaintext
- **Prefix-based lookup**: Fast O(log n) database queries with indexed token_prefix
- **Status tracking**: Tokens can be revoked without deletion (audit trail preserved)

## Technical Details

### Token Generation
1. Generate 32 cryptographically secure random bytes
2. Encode with base64 URL-safe (no padding)
3. Prefix with "bodhiapp_"
4. Extract first 8 chars after prefix for database lookup
5. Hash full token with SHA-256 for storage

### Token Validation
1. Check for "bodhiapp_" prefix
2. Extract prefix (first 17 chars: "bodhiapp_" + 8)
3. Database lookup by prefix
4. Check status is 'active'
5. Hash provided token with SHA-256
6. Constant-time comparison with stored hash
7. Parse scopes string to TokenScope enum

## References
- Implementation plan: `ai-docs/specs/20251001-api-tokens/final-plan.md`
- Agent logs: `ai-docs/specs/20251001-api-tokens/agent-token-log.md`
- Context patterns: `ai-docs/specs/20251001-api-tokens/agent-token-ctx.md`
