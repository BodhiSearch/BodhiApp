---
inclusion: manual
---
# Authentication and Authorization Guidelines

## Authentication Architecture

**Flow**: SPA-managed OAuth 2.0 Authorization Code flow with PKCE
**Frontend Role**: "Dumb" frontend - send all query params to backend without validation
**Backend Responsibility**: Handle all OAuth logic, errors, and redirects
**Anonymous Access**: Eliminated - authentication always required

## Required Documentation References

**MUST READ before auth changes:**
- `ai-docs/01-architecture/authentication.md` - OAuth2 and JWT patterns, security implementation
- `ai-docs/03-crates/auth_middleware.md` - Middleware implementation details
- `ai-docs/06-knowledge-transfer/llm-resource-server.md` - OAuth2 resource server patterns and vision

## Critical Authentication Rules

### OAuth Security Implementation
- **State Parameter**: Cryptographic validation using SHA-256 digest
- **State Composition**: `scope + 8char_random_id stored in session`
- **Scope Handling**: Extract from JWT claims, sort array, join with `%20`
- **Token Scopes at Client Level**: token_user, with `offline_access` at token level
- **Client Scopes**: resource_user, resource_power_user, resource_manager, resource_admin

### Authentication Flow Patterns
- **`/auth/initiate` returns**:
  - 401 with `auth_url` when login required
  - 303 with UI home location when already authenticated
- **Pages handle redirects**, not hooks
- **Reuse existing `useMutation` patterns from useQuery**

### Integration Testing
- **Test Scopes**: `'openid email profile roles'`
- Create encrypted secrets files
- Set AppStatus to Ready
- Obtain auth tokens and insert session data
- Set up cookies properly

## Follow Documentation Patterns

All authentication flow patterns, OAuth2 integration details, application states, frontend authentication patterns, backend implementation guidelines, and security best practices are comprehensively documented in the referenced ai-docs files above. Refer to those documents for the authoritative guidance rather than duplicating conventions here.
