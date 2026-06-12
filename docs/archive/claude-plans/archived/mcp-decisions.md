# MCP Integration Decisions Log

This document captures all decisions made during the MCP integration planning phase.

---

## OAuth 2.1 Client Registration

**Decision:** Dynamic registration (primary) + Pre-registered (fallback)

**Priority Order:**
1. **Dynamic Registration** - Each Bodhi App instance has its own client_id/client_secret from id.getbodhi.app
2. **Metadata Document** - Only for publicly hosted Bodhi App instances (using `bodhi_public_host` property)
3. **Pre-registered** - Admin enters client_id/client_secret manually as fallback
4. **Proxy Service** - DEFERRED to next phase (requires external proxy development similar to chromiumapp.org approach)

**Rationale:** Bodhi App instances receive their own client credentials from id.getbodhi.app, enabling resource isolation. Each instance manages its own authentication.

---

## Public MCP Servers

**Decision:** Instance required for all MCP types (including public)

**Rationale:** Consistent UX - all MCPs work the same way. User creates instance with custom slug/name regardless of auth type.

---

## MCP Authorization Scopes

**Decision:** DEFERRED - not part of current story

Current story focuses only on configure/create MCP server functionality.

---

## PKCE Storage

**Decision:** Database with state param key

**Implementation:**
- Create pending MCP auth record with state UUID as key
- Store code_verifier in this record
- Lookup record on OAuth callback using state param
- Complete the token exchange and store tokens

---

## MCP Session Management

**Decision:** Fresh session per chat

**Rationale:** New MCP session created for each chat conversation. Simpler implementation, no persistence of MCP-Session-Id across chats.

---

## OAuth Flow Timeout

**Decision:** No auto-cleanup

**Rationale:** User manually deletes incomplete MCP instances. Avoids complexity of background cleanup jobs. User has full control.

---

## Server Capabilities Caching

**Decision:** No caching

**Rationale:** Fetch capabilities (tools, resources, prompts) fresh on every tool invocation. Ensures always current data, accepts latency trade-off.

---

## Admin MCP Registry Preview

**Decision:** Fetch and display server metadata on add/view

**Implementation:** When admin adds MCP server or views registry entry, fetch and display:
- Available tools
- Available resources
- Available prompts
- Server capabilities

Helps admin verify server is working correctly.

---

## User MCP Page URL

**Decision:** `/ui/mcp`

---

## Chat UI - Tool Display

**Decision:** Separate icons for MCP tools and toolset tools

**Implementation:** In chat tools selector:
- MCP tools have distinct MCP icon
- Toolset tools have toolset icon
- Visually distinguishable at a glance

---

## Tool Naming Convention

**Decision:** `mcp_{instance_slug}__{tool_name}`

**Example:** If user creates MCP instance with slug "my-github" and server has tool "search_code":
- LLM sees tool as: `mcp_my-github__search_code`

**Rationale:** Consistent with toolset pattern (`toolset_{instance_name}__{method}`). Clear namespace prevents collision.

---

## OAuth Token Refresh Failure

**Decision:** Return auth error + mark instance as needs-reauth

**Implementation:**
1. If token refresh fails during tool execution, return authentication error to chat
2. Mark MCP instance status as "needs_reauth"
3. Instance shows "Reconnect" button in UI
4. User clicks to re-initiate OAuth flow
5. Clear remediation path without disrupting chat experience

---

## Summary Table

| Decision Area | Choice |
|---------------|--------|
| OAuth Registration | Dynamic + Pre-registered (MVP) |
| Public MCP | Instance required |
| Scopes | Deferred |
| PKCE Storage | Database with state key |
| MCP Sessions | Fresh per chat |
| OAuth Timeout | No auto-cleanup |
| Capabilities Cache | No caching |
| Admin Preview | Fetch metadata on add/view |
| URL Path | /ui/mcp |
| Chat UI | Separate icons |
| Tool Naming | mcp_{slug}__{tool} |
| Token Refresh Fail | Error + needs_reauth status |

---

## Additional Decisions (continued Q&A)

### OAuth Discovery

**Decision:** Discovery only (MCP spec compliant)

Follow MCP spec strictly - use `.well-known/oauth-protected-resource` discovery. If server doesn't support discovery, it's not MCP-compliant.

### MCP Features for MVP

**Decision:** Tools only

Focus on tool calling. Resources and prompts deferred to future phase.

### Redirect URI

**Decision:** `/ui/auth/mcp/callback`

Fixed path. State param sent as query parameter.

### Transport Type

**Decision:** `streamable_http` only

SSE is legacy. Only support Streamable HTTP transport for MVP.

### Custom Header

**Decision:** Configurable per-server

Admin specifies header name when adding server (e.g., `X-Custom-Auth`, `Authorization`).

### URL Validation

**Decision:** No validation on add

Admin can add any URL. Validation happens when user creates instance and connects.

### OAuth Window

**Decision:** Same window redirect

Navigate to OAuth provider in same window. User returns after completing flow.

### Default Slug

**Decision:** Default from domain, editable

Extract domain from MCP server URL. User can modify before saving.
Example: `https://api.github.com/mcp` -> default slug `github`

### Admin Page Location

**Decision:** `/ui/admin/mcp`

Dedicated MCP admin page. Consistent with admin namespace pattern.

### OAuth Return Page

**Decision:** MCP list page (`/ui/mcp`)

After OAuth callback completes, redirect to user's MCP instances list. Instance shows Ready status.

### Multiple Instances per Server

**Decision:** Yes - multiple instances allowed

User can create multiple instances of same MCP server with different slugs.
Example: `github-personal`, `github-work` - different OAuth accounts.

### Error Display

**Decision:** Full error always

Show complete MCP error response during tool execution. Helpful for debugging
