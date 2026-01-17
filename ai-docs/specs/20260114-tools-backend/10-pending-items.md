# Pending Items - Toolsets Backend

> Status: Tracking | Updated: 2026-01-17

## Overview

Pending security enhancements and feature requirements for the toolsets backend.

---

## Security Enhancements

### 1. Restrict Toolset Access for First-Party API Tokens

**Priority:** 🔴 High  
**Status:** Pending

#### Problem Statement

Currently, first-party API tokens (`bodhiapp_*` tokens with `TokenScope`) have the same unrestricted toolset access as session authentication. Any user who creates an API token can use it to execute tools if they have the toolset configured.

This is a security concern because:
- API tokens are designed for programmatic access with limited scope
- Token leakage should not grant full toolset execution capabilities
- Toolsets execute external API calls with user's personal API keys (e.g., Exa)

#### Current Behavior

```
First-party token auth (bodhiapp_*):
├── Check app_toolset_configs (admin enabled)
├── Check user_toolset_configs (user enabled + API key)
└── ✅ Toolset execution allowed (NO scope check)
```

#### Required Behavior

```
First-party token auth (bodhiapp_*):
├── Check app_toolset_configs (admin enabled)
├── Check token has toolset execution permission ← NEW
├── Check user_toolset_configs (user enabled + API key)
└── Toolset execution allowed only if all checks pass
```

#### Acceptance Criteria

- [ ] By default, newly created API tokens do NOT have toolset execution permission
- [ ] Toolset execution with first-party token returns 403 Forbidden unless explicitly granted
- [ ] Admin/user can grant toolset execution permission when creating tokens
- [ ] Existing tokens should be migrated to NOT have toolset permission
- [ ] Error message clearly indicates token lacks toolset permission

#### Implementation Options

**Option A: New TokenScope variant**
```rust
pub enum TokenScope {
  User,        // No toolset access
  Admin,       // No toolset access  
  UserToolsets,  // User access + toolsets
  AdminToolsets, // Admin access + toolsets
}
```

**Option B: Separate toolset permission flag**
- Add `toolset_access: bool` to API token storage
- Check flag in `toolset_auth_middleware`

#### Related Files

- `crates/auth_middleware/src/toolset_auth_middleware.rs`
- `crates/services/src/db/objs.rs`
- `crates/routes_app/src/routes_api_token.rs`

---

## Design Decisions

### Session Auth - Unrestricted Toolset Access (Accepted)

Session authentication will continue to have unrestricted toolset access. This is acceptable because:

1. Session auth is only used by BodhiApp's own frontend
2. The user is directly interacting with their own BodhiApp instance
3. User has explicitly configured their API key (implicit consent)
4. Toolset availability is already admin-gated at app-level

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
