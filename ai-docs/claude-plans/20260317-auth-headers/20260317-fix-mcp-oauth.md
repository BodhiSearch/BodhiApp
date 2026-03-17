# Fix: OAuth Token Exchange — Nullable `mcp_id`, No Duplication

## Context

The redesign correctly links OAuth tokens to MCP instances (fixing old per-config design). But `mcp_id` being required broke "create new MCP with OAuth" — the instance doesn't exist at token exchange time.

## Schema Design (Final)

```
mcp_auth_params    — Header/query auth ONLY (instance-level values)
├── mcp_id (NOT NULL FK→mcps)
├── param_type (header|query), param_key, encrypted_value/salt/nonce

mcp_oauth_tokens        — OAuth auth ONLY (complete OAuth session record)
├── mcp_id (NULLABLE FK→mcps)  ← allows pre-creation token storage
├── auth_config_id (FK→mcp_auth_configs)
├── user_id
├── encrypted_access_token/salt/nonce   ← restored (was removed in redesign)
├── encrypted_refresh_token/salt/nonce
├── scopes_granted, expires_at
```

**No duplication**: OAuth access token lives ONLY in `mcp_oauth_tokens`. `mcp_auth_params` is ONLY for header/query auth.

**Runtime resolution** branches on `auth_type`:
- `public` → no auth
- `header` → read `mcp_auth_params` by `mcp_id`
- `oauth` → read `mcp_oauth_tokens` by `mcp_id`, decrypt access_token, build `Authorization: Bearer <token>`

## Token Lifecycle

| Phase | What happens | `mcp_id` |
|-------|-------------|----------|
| Token exchange (new MCP) | Store access+refresh in `mcp_oauth_tokens` | NULL |
| Fetch tools preview | Decrypt access_token from token row by ID | NULL |
| Create MCP | Update token row: set `mcp_id` | Set to new ID |
| Token exchange (existing MCP) | Store with `mcp_id` directly | Set |
| Runtime | Read token by `mcp_id`, decrypt access_token | Set |
| Token refresh | Update token row (new access+refresh) | Set |
| Update MCP (reconnect) | Delete old token rows, link new one | Set |
| Delete MCP | CASCADE deletes linked tokens | N/A |

Frontend receives only the **token row ID** — never raw tokens.

## File Changes

### Migration: `crates/services/src/db/sea_migrations/m20250101_000016_mcp_auth_redesign.rs`
- Rename table `mcp_auth_credentials` → `mcp_auth_params` (loosely coupled, no FK to config_params — match by param_type+param_key)
- `mcp_oauth_tokens.mcp_id`: `string()` → `string_null()` (nullable)
- Add `encrypted_access_token: string()`, `access_token_salt: string()`, `access_token_nonce: string()` to `mcp_oauth_tokens`
- Apply pending fixes: correct `McpOauthTokens` table name, `TRUNCATE TABLE IF EXISTS`

### Table/Entity Rename: `mcp_auth_credentials` → `mcp_auth_params`
- Entity: `mcp_auth_credential_entity.rs` → `mcp_auth_param_entity.rs` (`McpAuthCredentialEntity` → `McpAuthParamEntity`)
- Domain: `McpAuthCredentialInput` → `McpAuthParamInput`, `McpAuthCredential` → `McpAuthParam` (response type)
- Repository: all `*_credential*` methods → `*_auth_param*`
- Service/routes/frontend: rename throughout
- `default_service.rs` TRUNCATE: `mcp_auth_credentials` → `mcp_auth_params`

### Entity: `crates/services/src/mcps/mcp_oauth_token_entity.rs`
- `mcp_id`: `String` → `Option<String>`
- Add `encrypted_access_token: String`, `access_token_salt: String`, `access_token_nonce: String`
- Update `McpOAuthTokenView` and `From` impl

### Domain: `crates/services/src/mcps/mcp_objs.rs`
- `McpOAuthToken.mcp_id`: `String` → `Option<String>`
- `McpRequest`: add `oauth_token_id: Option<String>`

### Repository: `crates/services/src/mcps/mcp_auth_repository.rs`
- Handle nullable `mcp_id` in `create_mcp_oauth_token`
- New: `get_decrypted_oauth_access_token(tenant_id, token_id) → Option<String>`
- New: `link_oauth_token_to_mcp(tenant_id, token_id, user_id, mcp_id)` — sets `mcp_id` with user_id ownership check

### Service: `crates/services/src/mcps/mcp_service.rs`
- `store_oauth_token`: `mcp_id: &str` → `mcp_id: Option<&str>`. Store access_token in token table (not credentials). Skip `mcp_auth_params` entirely for OAuth.
- `exchange_oauth_token`: `mcp_id: &str` → `mcp_id: Option<&str>`
- `create()`: if `request.oauth_token_id` provided, link token to MCP (update `mcp_id`)
- `update()`: if `oauth_token_id` changed, delete old tokens for MCP, link new one
- `fetch_tools_for_server`: accept `oauth_token_id: Option<String>`, decrypt access_token from `mcp_oauth_tokens`
- `resolve_auth_params_for_mcp`: for OAuth, read from `mcp_oauth_tokens` (not `mcp_auth_params`)
- `resolve_oauth_token`: update access_token in `mcp_oauth_tokens` (not `mcp_auth_params`) on refresh

### Auth scoped: `crates/services/src/mcps/auth_scoped.rs`
- Update `exchange_oauth_token`, `fetch_tools_for_server` signatures

### Routes: `crates/routes_app/src/mcps/`
- `OAuthTokenExchangeRequest.mcp_id`: `String` → `Option<String>`
- `FetchMcpToolsRequest`: add `oauth_token_id: Option<String>`
- Route handlers: pass through

### Frontend: `crates/bodhi/src/`
- `callback/page.tsx`: `mcp_id: undefined` for new MCPs, store returned `token_id`
- `new/page.tsx`: pass `oauth_token_id` in fetch tools + create/update
- `mcpFormStore.ts`: add `oauthTokenId` field

### Regenerate: OpenAPI + TS client

### Tests
- **routes_app**: create MCP with `oauth_token_id` → verify token linked; update with new token → verify old deleted
- **E2E**: existing OAuth specs should pass

## Pending Uncommitted Changes (commit together)
1. Migration table name fix
2. TRUNCATE IF EXISTS

## Verification
1. `cargo check -p services -p routes_app -p server_app`
2. `cargo test -p services --lib -- mcps`
3. `cargo test -p routes_app --lib -- mcps`
4. `cd crates/bodhi && npm test`
5. `make build.ui-rebuild && make test.napi`
