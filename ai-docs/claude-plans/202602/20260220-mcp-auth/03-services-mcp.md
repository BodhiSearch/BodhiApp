# MCP OAuth - Service Layer (`services` crate)

## Task

✅ **COMPLETED** - Implement McpService methods for OAuth config management, authorization flow, token exchange/refresh, RFC 8414/9728 discovery, and RFC 7591 dynamic registration.

**Refactor Impact**: Updated auth type matching to use simplified 3-variant `McpAuthType` enum.

## Files

| File | Purpose |
|------|---------|
| `crates/services/src/mcp_service/service.rs` | McpService trait + DefaultMcpService implementation |
| `crates/services/src/mcp_service/error.rs` | OAuth error variants |

## DefaultMcpService Struct

Fields:
- `db_service: Arc<dyn DbService>` - database access
- `mcp_client: Arc<dyn McpClient>` - MCP tool execution
- `time_service: Arc<dyn TimeService>` - testable time (never `Utc::now()`)
- `http_client: reqwest::Client` - shared HTTP client for all OAuth operations
- `refresh_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>` - per-config concurrency control for token refresh

## McpService Trait Methods (OAuth-related)

### OAuth Config CRUD

✅ **NO CHANGES** - Unified config creation method already existed:

- `create_oauth_config(user_id, name, mcp_server_id, client_id, client_secret?, authorization_endpoint, token_endpoint, scopes?, registration_type?, registration_endpoint?, token_endpoint_auth_method?, client_id_issued_at?, registration_access_token?) -> Result<McpOAuthConfig>` - Encrypts client_secret and registration_access_token if present, creates row via repository
- `get_oauth_config(id) -> Result<Option<McpOAuthConfig>>`
- `list_oauth_configs_by_server(mcp_server_id) -> Result<Vec<McpOAuthConfig>>`
- `delete_oauth_config(id) -> Result<()>`

**Note**: The discriminated union in `CreateMcpAuthConfigRequest` dispatches to this single unified method, passing `registration_type` from the request.

### OAuth Token Operations

- `store_oauth_token(user_id, config_id, access_token, refresh_token?, scopes_granted?, expires_in?) -> Result<McpOAuthToken>` - Encrypts access+refresh tokens, calculates `expires_at = now + expires_in`
- `get_oauth_token(user_id, token_id) -> Result<Option<McpOAuthToken>>` - user-scoped

### OAuth Discovery

- `discover_oauth_metadata(url) -> Result<serde_json::Value>` - GETs `{url}/.well-known/oauth-authorization-server` (RFC 8414)
- `discover_mcp_oauth_metadata(mcp_server_url) -> Result<serde_json::Value>` - Multi-step: fetches `{origin}/.well-known/oauth-protected-resource` (RFC 9728) to get AS URL, then fetches `{as_url}/.well-known/oauth-authorization-server` (RFC 8414), augments response with `resource` and `authorization_server_url` fields
- `dynamic_register_client(registration_endpoint, redirect_uri, scopes?) -> Result<serde_json::Value>` - POSTs RFC 7591 registration with client_name="BodhiApp", grant_types=["authorization_code","refresh_token"], token_endpoint_auth_method="none"

## Auth Resolution

✅ **UPDATED** - `resolve_auth_header_for_mcp(mcp_row) -> Result<Option<(String, String)>>` - Central dispatch:

1. Parses `mcp_row.auth_type` to `McpAuthType`
2. **Public**: returns `None`
3. **Header**: calls `db_service.get_decrypted_auth_header(auth_uuid)` -> `Option<(key, value)>`
4. **Oauth**: calls `resolve_oauth_token(user_id, auth_uuid)` for token with automatic refresh

**Git diff from crates/services/src/mcp_service/service.rs (line 575):**
```rust
-      McpAuthType::OauthPreRegistered | McpAuthType::OauthDynamic => {
+      McpAuthType::Oauth => {
         if let Some(ref auth_uuid) = mcp_row.auth_uuid {
           return self.resolve_oauth_token(user_id, auth_uuid).await;
         }
       }
```

## Token Resolution with Proactive Refresh

✅ **NO CHANGES** - Token refresh logic works identically with simplified enum:

`resolve_oauth_token(user_id, auth_uuid) -> Result<Option<(String, String)>>`:

1. **Acquires per-config Mutex** via `get_refresh_lock("oauth_refresh:{auth_uuid}")` - double-check locking pattern to minimize RwLock contention
2. **Fetches token**: `db_service.get_mcp_oauth_token(user_id, auth_uuid)` - raises `OAuthTokenNotFound` if missing
3. **Checks expiration**: `now >= (expires_at - 60)` - proactively refreshes 60 seconds before actual expiry
4. **If expired with refresh token**: decrypts refresh token, fetches OAuth config + client credentials, POSTs to `token_endpoint` with `grant_type=refresh_token` + `resource` parameter (MCP server URL), re-encrypts new tokens, updates row
5. **If expired without refresh token**: raises `OAuthTokenExpired`
6. **If not expired**: decrypts and returns stored access token as `("Authorization", "Bearer <token>")`

The token refresh process doesn't distinguish between pre-registered and dynamic clients - both use the same token endpoint flow.

## Error Variants (error.rs)

| Variant | ErrorType | Message |
|---------|-----------|---------|
| `OAuthTokenNotFound(String)` | `NotFound` | "OAuth token not found for config '{0}'." |
| `OAuthTokenExpired(String)` | `BadRequest` | "OAuth token expired and no refresh token available for config '{0}'." |
| `OAuthRefreshFailed(String)` | `InternalServer` | "OAuth token refresh failed: {0}." |
| `OAuthDiscoveryFailed(String)` | `InternalServer` | "OAuth discovery failed: {0}." |

## Design Decisions

- **Auth headers NOT deleted when MCP switches auth types**: Headers are admin-managed reusable resources, not tied to a single MCP instance lifecycle
- **reqwest::Client shared**: Single instance for all OAuth HTTP operations, connection pooling
- **Per-config Mutex, not global**: Allows concurrent refresh of different configs while serializing refreshes for the same config
- **Resource parameter in token exchange**: Includes MCP server URL as `resource` parameter per RFC 8707

## Cross-References

- Repository trait these methods delegate to: [02-services-db.md](./02-services-db.md)
- Route handlers that call these methods: [04-routes-app.md](./04-routes-app.md)
