# Pending Items - Tools Backend

> Status: Tracking | Updated: 2026-01-16

## Overview

This document tracks pending security enhancements and feature requirements for the tools backend that should be addressed before full production release.

---

## Security Enhancements

### 1. Restrict Tool Access for First-Party API Tokens

**Priority:** 🔴 High  
**Status:** Pending  
**Phase:** Post-7.6

#### Problem Statement

Currently, first-party API tokens (`bodhiapp_*` tokens with `TokenScope`) have the same unrestricted tool access as session authentication. Any user who creates an API token can use it to execute tools if they have the tool configured.

This is a security concern because:
- API tokens are designed for programmatic access with limited scope
- Users may create tokens for specific purposes (e.g., chat completion only)
- Token leakage should not grant full tool execution capabilities
- Tools execute external API calls with user's personal API keys (e.g., Exa)

#### Current Behavior

```
First-party token auth (bodhiapp_*):
├── Check app_tool_configs (admin enabled)
├── Check user_tool_configs (user enabled + API key)
└── ✅ Tool execution allowed (NO scope check)
```

#### Required Behavior

```
First-party token auth (bodhiapp_*):
├── Check app_tool_configs (admin enabled)
├── Check token has tool execution permission ← NEW
├── Check user_tool_configs (user enabled + API key)
└── Tool execution allowed only if all checks pass
```

#### Acceptance Criteria

- [ ] By default, newly created API tokens do NOT have tool execution permission
- [ ] Tool execution with first-party token returns 403 Forbidden unless explicitly granted
- [ ] Admin/user can grant tool execution permission when creating tokens
- [ ] Existing tokens should be migrated to NOT have tool permission (breaking change documented)
- [ ] Error message clearly indicates token lacks tool permission

#### Implementation Options

**Option A: New TokenScope variant**
```rust
pub enum TokenScope {
  User,       // Existing - no tool access
  Admin,      // Existing - no tool access  
  UserTools,  // New - user access + tools
  AdminTools, // New - admin access + tools
}
```

**Option B: Separate tool permission flag**
- Add `tool_access: bool` to API token storage
- Check flag in `tool_auth_middleware`

**Option C: Token claims extension**
- Store tool permissions in token claims
- More flexible but requires token re-issuance

#### Related Files

- `crates/auth_middleware/src/tool_auth_middleware.rs` - Add token permission check
- `crates/services/src/db/objs.rs` - Extend ApiToken if needed
- `crates/routes_app/src/routes_api_token.rs` - Token creation endpoint

---

## Design Decisions

### Session Auth - Unrestricted Tool Access (Accepted)

Session authentication will continue to have unrestricted tool access (only app-level + user config checks). This is acceptable because:

1. Session auth is only used by BodhiApp's own frontend
2. The user is directly interacting with their own BodhiApp instance
3. User has explicitly configured their API key (implicit consent)
4. Tool availability is already admin-gated at app-level

---

## Future Enhancements (Lower Priority)

### Tool Audit Logging

- Log tool executions for security audit
- Include: user_id, tool_id, auth_type, timestamp, success/failure

### Tool Rate Limiting

- Per-user rate limits for tool API calls
- Prevent abuse of external APIs (e.g., Exa)

### Role-Based Tool Access

- Allow admin to restrict certain tools to specific roles
- Example: Only `PowerUser` role can use expensive tools

---

## Related Documents

- [00-overview.md](./00-overview.md) - Security Considerations section
- [05-auth-scopes.md](./05-auth-scopes.md) - Token types summary
- [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md) - External OAuth flow (properly secured)
