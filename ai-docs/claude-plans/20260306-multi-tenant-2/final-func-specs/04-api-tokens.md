# Multi-Tenant Functional Specification — API Tokens

> **Scope**: Token format, creation, validation, tenant scoping, CRUD operations
> **Related specs**: [Index](00-index.md) · [Auth Flows](01-auth-flows.md) · [Feature Gating](06-feature-gating.md)
> **Decisions**: D22 (token-based tenant resolution)

---

## Token Format

```
bodhiapp_<base64url_random_32_bytes>.<client_id>
```

**Example:** `bodhiapp_AAAA1234BBBB5678CCCC9012DDDD.bodhi-tenant-a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6`

**Components:**
- `bodhiapp_` — fixed prefix identifying BodhiApp tokens
- `<base64url_random_32_bytes>` — cryptographically random token body
- `.` — separator
- `<client_id>` — tenant's Keycloak client_id (tenant scoping)

**Storage:**
- **Prefix** (first 8 chars after `bodhiapp_`): stored in DB for lookup
- **Full hash**: SHA-256 of the entire token string (with client_id suffix)
- **Token shown once**: full token returned only at creation time, never again

The `client_id` suffix provides inherent tenant isolation — a token for tenant A cannot be used to access tenant B's data.

---

## Token Creation {#creation}

### `POST /bodhi/v1/tokens`

**Auth:** Session only (PowerUser+ role). API tokens CANNOT create tokens — this prevents privilege escalation.

**Request:**
```json
{
  "name": "My API Token",
  "scope": "scope_token_user"
}
```

- `name`: required, descriptive name for the token
- `scope`: required, one of `"scope_token_user"` or `"scope_token_power_user"`

**Scope validation by role:**

| User's Role | Can create `scope_token_user` | Can create `scope_token_power_user` |
|------------|------------------------------|-------------------------------------|
| `resource_user` | Yes | No |
| `resource_power_user` | Yes | Yes |
| `resource_manager` | Yes | Yes |
| `resource_admin` | Yes | Yes |

**Response (201):**
```json
{
  "token": "bodhiapp_AAAA1234BBBB5678CCCC9012DDDD.bodhi-tenant-a1b2c3d4-...",
  "id": "token-ulid-id",
  "name": "My API Token",
  "scope": "scope_token_user",
  "status": "active",
  "created_at": "2026-03-09T10:00:00Z"
}
```

**Token creation flow:**
1. Extract `tenant_id` and `user_id` from `AuthContext::Session`
2. Validate user's role permits requested scope
3. Generate 32 bytes of cryptographically random data → base64url encode
4. Append `.{client_id}` suffix (client_id from the tenant)
5. Compute SHA-256 hash of full token
6. Store: prefix (first 8 chars), hash, name, scope, user_id, tenant_id
7. Return full token to user (shown only once)

---

## Token Validation Flow {#validation}

When an API token arrives in the `Authorization: Bearer` header:

```
1. Check if token starts with "bodhiapp_" prefix
   → No: try JWT validation (session or external token)
   → Yes: continue API token flow

2. Extract prefix (first 8 chars after "bodhiapp_")

3. Lookup by prefix: get_api_token_by_prefix(prefix_hash)
   → Not found: 401 Unauthorized

4. SHA-256 verify: hash(full_token) == stored_hash
   → Mismatch: 401 Unauthorized

5. Parse client_id from suffix: split by "." → last segment
   → get_tenant_by_client_id(client_id)
   → Tenant not found: 401 Unauthorized

6. Check token status == "active"
   → Inactive/revoked: 401 Unauthorized

7. Build AuthContext::ApiToken {
     client_id,
     tenant_id: tenant.id,
     user_id: token.user_id,
     role: token.scope,      // scope_token_user or scope_token_power_user
     token: raw_token
   }
```

---

## CRUD Operations {#crud}

### `GET /bodhi/v1/tokens`

List tokens for the current user's active tenant.

**Auth:** Session only (PowerUser+)

**Query params:** `page` (default 1), `per_page` (default 20)

**Response (200):**
```json
{
  "data": [
    {
      "id": "token-ulid",
      "name": "My API Token",
      "token_prefix": "AAAA1234",
      "scope": "scope_token_user",
      "status": "active",
      "created_at": "2026-03-09T10:00:00Z",
      "updated_at": "2026-03-09T10:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 20
}
```

Note: `token_prefix` is shown instead of the full token. The full token is only available at creation.

### `PUT /bodhi/v1/tokens/{token_id}`

Update token name or status.

**Auth:** Session only (PowerUser+)

**Request:**
```json
{
  "name": "Renamed Token",
  "status": "inactive"
}
```

- `name`: optional, new display name
- `status`: optional, `"active"` or `"inactive"`

**Response (200):**
```json
{
  "id": "token-ulid",
  "name": "Renamed Token",
  "token_prefix": "AAAA1234",
  "scope": "scope_token_user",
  "status": "inactive",
  "created_at": "2026-03-09T10:00:00Z",
  "updated_at": "2026-03-09T11:00:00Z"
}
```

---

## Scope & Authorization

### Token Scopes

| Scope | API access level | Equivalent to |
|-------|-----------------|---------------|
| `scope_token_user` | Read models, chat completions | `resource_user` session |
| `scope_token_power_user` | Model CRUD, token listing, chat | `resource_power_user` session |

### Auth Middleware Scope Mapping

The `api_auth_middleware` checks incoming requests against required roles. Token scopes map to the role hierarchy:

```
scope_token_power_user → can access: PowerUser routes + User routes
scope_token_user       → can access: User routes only
```

Session-only routes (token management, settings, user management) are NOT accessible via API tokens.

---

## Multi-Tenant Behavior

### Inherent Tenant Isolation

- Token format includes `client_id` as suffix → tenant is resolved at validation time
- A token created for tenant A will always resolve to tenant A's data
- Cross-tenant access is impossible by construction (client_id mismatch → different tenant lookup)
- No additional multi-tenant logic needed beyond the standard validation flow

### Tenant-scoped Operations

All token CRUD operations use `AuthScopedTokenService`, which automatically injects `tenant_id` from `AuthContext`:

```rust
pub struct AuthScopedTokenService {
  app_service: Arc<dyn AppService>,
  auth_context: AuthContext,
}

// All methods automatically scope to auth_context.tenant_id
impl AuthScopedTokenService {
  pub async fn create_token(&self, request: CreateTokenRequest) -> Result<TokenCreated, TokenServiceError>;
  pub async fn list_tokens(&self, page: usize, per_page: usize) -> Result<PaginatedTokenResponse, TokenServiceError>;
  pub async fn get_token(&self, id: &str) -> Result<Option<TokenDetail>, TokenServiceError>;
  pub async fn update_token(&self, id: &str, request: UpdateTokenRequest) -> Result<TokenDetail, TokenServiceError>;
}
```
