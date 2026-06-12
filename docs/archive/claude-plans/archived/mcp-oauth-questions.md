# MCP OAuth 2.1 Client Registration Methods Analysis

## Context

MCP specification 2025-11-25 requires OAuth 2.1 for HTTP transport authentication. The spec supports multiple client registration methods per RFC standards.

## Three Registration Methods

### Method 1: Pre-registered Credentials

**How it works:**
- Admin manually registers BodhiApp as a client with the MCP OAuth server
- Admin obtains `client_id` and `client_secret` from the OAuth server's admin console
- Admin enters these credentials when adding MCP server to BodhiApp registry
- All users share the same client credentials (but get individual access tokens)

**Pros:**
- Simplest implementation
- Works with ALL OAuth servers (100% compatibility)
- No dynamic discovery failures
- Predictable behavior
- Easier to debug

**Cons:**
- Requires admin to manually register with each OAuth provider
- Different MCP providers have different registration UIs
- May need to coordinate redirect_uri registration

**Implementation effort:** Low

**Schema addition:**
```sql
-- In mcp_server_registry
oauth_client_id TEXT,
oauth_client_secret_encrypted TEXT,  -- encrypted
oauth_client_secret_salt TEXT,
oauth_client_secret_nonce TEXT,
```

---

### Method 2: Client ID Metadata Documents (RFC 9449 style)

**How it works:**
- BodhiApp uses a URL as its `client_id` (e.g., `https://getbodhi.app/.well-known/oauth-client`)
- That URL serves JSON metadata describing the client
- OAuth server fetches this URL to validate client identity
- No client_secret needed (public client with PKCE)

**Example client metadata served at the URL:**
```json
{
  "client_id": "https://getbodhi.app/.well-known/oauth-client",
  "client_name": "BodhiApp",
  "redirect_uris": ["https://getbodhi.app/ui/auth/mcp"],
  "grant_types": ["authorization_code", "refresh_token"],
  "response_types": ["code"],
  "token_endpoint_auth_method": "none",
  "scope": "openid profile"
}
```

**Pros:**
- No manual registration per OAuth server
- OAuth server auto-discovers client info
- More "modern" approach

**Cons:**
- Requires BodhiApp to host a public metadata endpoint
- Not all OAuth servers support this
- Desktop app users need public endpoint (complicates self-hosted)
- MCP spec says this is one option but doesn't mandate support

**Implementation effort:** Medium-High (need public endpoint, CORS handling)

**Who supports it:**
- GitHub doesn't support it
- Google doesn't support it
- Auth0 partial support
- Keycloak partial support
- Most corporate OAuth servers don't support it

---

### Method 3: Dynamic Client Registration (RFC 7591)

**How it works:**
- OAuth server exposes a registration endpoint
- BodhiApp calls registration endpoint with client metadata
- Server returns `client_id` (and optionally `client_secret`)
- This happens automatically without admin intervention

**Flow:**
1. Discover OAuth server metadata
2. Find `registration_endpoint` in metadata
3. POST client metadata to registration endpoint
4. Receive client credentials
5. Store credentials for future use

**Example registration request:**
```http
POST /oauth/register HTTP/1.1
Content-Type: application/json

{
  "client_name": "BodhiApp",
  "redirect_uris": ["https://example.com/callback"],
  "grant_types": ["authorization_code", "refresh_token"],
  "response_types": ["code"],
  "token_endpoint_auth_method": "none"
}
```

**Pros:**
- Fully automated
- No admin intervention needed
- Good for large-scale MCP server adoption

**Cons:**
- Many OAuth servers disable dynamic registration (security reasons)
- Corporate OAuth servers almost never support it
- GitHub, Google, most major providers don't support it
- If server doesn't support, need fallback anyway

**Implementation effort:** Medium

**Who supports it:**
- Auth0 (can be enabled)
- Keycloak (can be enabled, disabled by default)
- Most enterprise OAuth servers: NO
- GitHub, Google, Microsoft: NO

---

## Recommendation Matrix

| Scenario | Recommended Method |
|----------|-------------------|
| Enterprise MCP servers | Pre-registered |
| Self-hosted MCP | Pre-registered |
| Public MCP ecosystem | Pre-registered (with future dynamic) |
| MVP | Pre-registered |

## MVP Recommendation: Pre-registered Only

**Rationale:**
1. **100% compatibility** - Works with any OAuth server
2. **Simplest implementation** - Clear responsibility (admin registers)
3. **Matches existing pattern** - Similar to how toolset API keys work
4. **Future extensible** - Can add dynamic registration later
5. **MCP server reality** - Most MCP servers will use standard OAuth (Google, GitHub, corporate) which require pre-registration

**Trade-off accepted:**
- Admin must manually register with each OAuth provider
- This is acceptable because adding an MCP server to registry is already an admin action

---

## Alternative: Pre-registered + Discovery-based fallback

If you want to support servers that use .well-known discovery:

1. Admin marks auth_type as "oauth2.1"
2. Admin optionally provides client_id/client_secret
3. If provided: use pre-registered credentials
4. If not provided: attempt discovery flow:
   a. Try Client ID Metadata Documents (if server supports)
   b. Try Dynamic Registration (if server supports)
   c. Fail with clear error if neither works

This gives flexibility but increases complexity.

---

## Schema Implications by Choice

### Pre-registered Only

```sql
-- mcp_server_registry (admin creates)
CREATE TABLE mcp_server_registry (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    url TEXT NOT NULL,
    auth_type TEXT NOT NULL,  -- 'public', 'oauth2.1', 'bearer', 'custom_header'
    enabled INTEGER NOT NULL DEFAULT 0,
    -- OAuth fields (only for oauth2.1)
    oauth_client_id TEXT,
    oauth_client_secret_encrypted TEXT,
    oauth_client_secret_salt TEXT,
    oauth_client_secret_nonce TEXT,
    oauth_scopes TEXT,  -- default scopes to request
    -- Custom header fields (only for custom_header)
    custom_header_name TEXT,
    -- Metadata
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### With Dynamic Registration

Would need additional fields:
```sql
    oauth_registration_method TEXT,  -- 'pre_registered', 'dynamic', 'metadata_document'
    oauth_dynamically_registered INTEGER DEFAULT 0,
```

---

## BodhiApp-Specific Architecture Decision

### Context: Bodhi App Instance Model

Each Bodhi App instance receives its own `client_id`/`client_secret` from `id.getbodhi.app`. This enables resource isolation where each instance manages its own authentication independently.

### Final Decision: Priority-Based Registration

**Priority Order (MVP):**

1. **Dynamic Registration (Primary)**
   - Each Bodhi App instance uses its credentials from id.getbodhi.app
   - Attempts RFC 7591 dynamic registration with MCP server's OAuth provider
   - If registration_endpoint available, auto-register

2. **Pre-registered (Fallback)**
   - Admin manually enters client_id/client_secret obtained from OAuth provider
   - Works when dynamic registration not supported

**Future Phase: Metadata Document + Proxy Service**

3. **Metadata Document**
   - Only for publicly hosted Bodhi App instances
   - Uses `bodhi_public_host` property to determine if instance is public
   - Hosts OAuth client metadata at `.well-known/oauth-client`

4. **Proxy Service (DEFERRED - requires external development)**
   - Similar to chromiumapp.org approach for Chrome extensions
   - Proxy service deployed by BodhiSearch team
   - Flow:
     a. Bodhi App instance authenticates to proxy using its client token
     b. Proxy receives state, PKCE params from user
     c. Proxy composes OAuth request with its redirect_url
     d. User completes OAuth flow, code returned to proxy
     e. Bodhi App sends code to proxy for token exchange
     f. Proxy forwards to MCP OAuth server, returns token to app
   - Restricts MCPs to those proxy supports
   - Provides option to raise GitHub ticket for new MCP support
   - Provides option for users to self-register and obtain credentials

### Schema for MVP (Dynamic + Pre-registered)

```sql
-- mcp_server_registry (admin creates)
CREATE TABLE mcp_server_registry (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    url TEXT NOT NULL,
    auth_type TEXT NOT NULL,  -- 'public', 'oauth2.1', 'bearer', 'custom_header'
    enabled INTEGER NOT NULL DEFAULT 0,

    -- OAuth fields (only for oauth2.1)
    oauth_registration_method TEXT,  -- 'dynamic', 'pre_registered'
    oauth_client_id TEXT,            -- NULL for dynamic, required for pre_registered
    oauth_client_secret_encrypted TEXT,
    oauth_client_secret_salt TEXT,
    oauth_client_secret_nonce TEXT,
    oauth_scopes TEXT,               -- default scopes to request

    -- Custom header fields (only for custom_header)
    custom_header_name TEXT,

    -- Metadata
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- mcp_instances (user creates)
CREATE TABLE mcp_instances (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    server_id TEXT NOT NULL,         -- FK to mcp_server_registry
    slug TEXT NOT NULL,              -- user-defined, URL-safe
    status TEXT NOT NULL,            -- 'not_ready', 'ready', 'needs_reauth'

    -- For dynamic registration - per-instance client credentials
    dynamically_registered INTEGER DEFAULT 0,
    oauth_client_id TEXT,            -- obtained during registration
    oauth_client_secret_encrypted TEXT,
    oauth_client_secret_salt TEXT,
    oauth_client_secret_nonce TEXT,

    -- OAuth tokens (per-user access)
    oauth_access_token_encrypted TEXT,
    oauth_access_token_salt TEXT,
    oauth_access_token_nonce TEXT,
    oauth_refresh_token_encrypted TEXT,
    oauth_refresh_token_salt TEXT,
    oauth_refresh_token_nonce TEXT,
    oauth_token_expires_at INTEGER,
    oauth_scopes_granted TEXT,

    -- PKCE state (temporary during OAuth flow)
    oauth_state TEXT,                -- state param for OAuth flow
    oauth_code_verifier_encrypted TEXT,
    oauth_code_verifier_salt TEXT,
    oauth_code_verifier_nonce TEXT,

    -- Bearer/Custom header auth
    auth_value_encrypted TEXT,       -- bearer token or custom header value
    auth_value_salt TEXT,
    auth_value_nonce TEXT,

    -- Metadata
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    UNIQUE(user_id, slug)
);
```

### Implementation Notes

1. **Dynamic Registration Flow:**
   - On instance creation with OAuth server:
     a. Fetch OAuth server metadata (`.well-known/oauth-authorization-server`)
     b. Check for `registration_endpoint`
     c. If available, POST client registration request
     d. Store obtained client_id/client_secret with instance
     e. Proceed to OAuth authorization flow

2. **Pre-registered Fallback:**
   - If dynamic registration fails or not available
   - Use client_id/client_secret from mcp_server_registry
   - All instances of this server share registry credentials
   - But each user gets individual access tokens

3. **id.getbodhi.app Integration:**
   - Bodhi App instance's own client credentials used for:
     a. Authenticating to dynamic registration endpoint (if required)
     b. Future proxy service authentication
   - Stored in app's existing secret management
