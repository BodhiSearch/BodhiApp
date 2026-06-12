# Pending Items - Toolsets Backend

> Status: Tracking | Updated: 2026-01-18

## Overview

Pending security enhancements and feature requirements for the toolsets backend.

---

## Completed Security Enhancements

### 1. Restrict Toolset Access for First-Party API Tokens

**Priority:** 🔴 High  
**Status:** ✅ Completed

#### Solution Implemented

API tokens (`bodhiapp_*`) are now **completely blocked** from ALL toolset endpoints at the route level. This is simpler and more secure than adding granular permissions.

#### Implementation Details

**Route-level blocking** in `crates/routes_all/src/routes.rs`:
- Toolset config endpoints (`GET/PUT/DELETE /toolsets/{id}/config`) → `user_session_apis` (session-only)
- Toolset list and execute endpoints (`GET /toolsets`, `POST /execute`) → `user_oauth_apis` (session + OAuth only)

**API Token behavior:**
- ALL toolset endpoints return 401 Unauthorized for API tokens
- No migration needed - existing tokens simply cannot access toolsets

**OAuth Token behavior:**
- List endpoint (`GET /toolsets`) → Returns toolsets filtered by `scope_toolset-*` scopes in token
- Execute endpoint (`POST /execute`) → Requires matching `scope_toolset-*` scope
- Config endpoints → Blocked (401) - session-only for security

#### Access Matrix

| Auth Type | List | Get Config | Put/Delete Config | Execute |
|-----------|------|------------|-------------------|---------|
| Session | All toolsets | Allowed | Allowed | Allowed |
| API Token | 401 | 401 | 401 | 401 |
| OAuth | Filtered by scope | 401 | 401 | With scope check |

#### E2E Tests

See `crates/lib_bodhiserver_napi/tests-js/specs/toolsets/toolsets-auth-restrictions.spec.mjs`

---

## Design Decisions

### Session Auth - Unrestricted Toolset Access (Accepted)

Session authentication will continue to have unrestricted toolset access. This is acceptable because:

1. Session auth is only used by BodhiApp's own frontend
2. The user is directly interacting with their own BodhiApp instance
3. User has explicitly configured their API key (implicit consent)
4. Toolset availability is already admin-gated at app-level

### API Tokens - Full Block vs Granular Permissions

**Decision:** Full block (all toolset endpoints return 401 for API tokens)

**Rationale:**
- Simpler implementation - no token schema changes needed
- More secure - token leakage cannot compromise toolset access
- API tokens are primarily for chat completions, not tool execution
- Users who need programmatic toolset access can use OAuth flow

---

## Future Enhancements (Lower Priority)

### Toolset Audit Logging

- Log toolset executions for security audit
- Include: user_id, toolset_id, tool_name, auth_type, timestamp, success/failure

### Toolset Rate Limiting

- Per-user rate limits for toolset API calls
- Prevent abuse of external APIs (e.g., Exa)

### Role-Based Toolset Access

- Allow admin to restrict certain toolsets to specific roles
- Example: Only `PowerUser` role can use expensive toolsets

### Additional Toolsets

- Web scraping toolset
- Image search toolset
- Code execution toolset

---

## Related Documents

- [00-overview.md](./00-overview.md) - Security Considerations section
- [05-auth-scopes.md](./05-auth-scopes.md) - Token types summary
- [05.6-external-app-toolset-access.md](./05.6-external-app-toolset-access.md) - External OAuth flow
