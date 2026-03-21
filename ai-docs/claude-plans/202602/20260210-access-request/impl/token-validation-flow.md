# Token Validation Flow

> **Purpose**: How external app bearer tokens are validated at runtime.
> Covers: auth middleware chain, pre/post KC token exchange, scope extraction, AuthContext::ExternalApp construction, caching.

---

## Middleware Chain Overview

When an external app calls a BodhiApp API with a bearer token, the request passes through:

```
Request with Authorization: Bearer <external_token>
  |
  v
1. auth_middleware          -- Extract token, classify, validate, build AuthContext
  |
  v
2. api_auth_middleware      -- Check UserScope role requirement
  |
  v
3. access_request_auth_middleware  -- Entity-level enforcement (see entity-enforcement-flow.md)
  |
  v
4. Route handler
```

---

## Step 1: auth_middleware

**File**: `crates/routes_app/src/middleware/auth/auth_middleware.rs`

### Header Sanitization

First action: strips all `X-BodhiApp-*` headers from the request to prevent injection attacks.

### Bearer Token Path

1. Extract `Authorization: Bearer <token>` header
2. Call `token_service.validate_bearer_token(token)`
3. Insert returned `AuthContext` into request extensions
4. Continue to next middleware

### Session Fallback (same-origin only)

If no bearer token but request is same-origin:
1. Read `SESSION_KEY_ACTIVE_CLIENT_ID` from session
2. Read `"{client_id}:access_token"` from session
3. Validate/refresh via KC
4. Build `AuthContext::Session` or `AuthContext::MultiTenantSession`

External app requests always use the bearer path, not session.

---

## Step 2: Token Classification (token_service)

**File**: `crates/routes_app/src/middleware/token_service/token_service.rs`

### validate_bearer_token()

Two paths based on token format:

**API Token** (prefix `bodhiapp_`):
- Format: `bodhiapp_<random_8+>.<client_id>`
- Look up by prefix in DB, SHA-256 verify, build `AuthContext::ApiToken`
- Not relevant to access request flow

**External/Session Token** (no `bodhiapp_` prefix):
1. Extract `exp` claim, check not expired
2. Check cache (SHA-256 digest of token as key)
3. If cache hit (valid + not stale): reconstruct `AuthContext::ExternalApp` from cached data
4. If cache miss: call `handle_external_client_token()` for full validation

---

## Step 3: Pre-Exchange Validation

**Function**: `handle_external_client_token()` in token_service.rs

### 3a. Extract JWT Claims

```
ScopeClaims {
  azp: "external-app-client",     // App's KC client ID
  aud: "bodhi-instance-client",   // BodhiApp's KC client ID
  sub: "user-123",                // User ID
  iss: "https://kc.example.com/realms/bodhi",
  scope: "openid profile email scope_access_request:ar-uuid",
  resource_access: { ... },
  access_request_id: Option<String>,
}
```

### 3b. Verify Issuer

`claims.iss` must match `setting_service.auth_issuer()`. Otherwise `TokenError::InvalidIssuer`.

### 3c. Tenant Resolution

Extract `aud` claim -> look up tenant via `tenant_service.get_tenant_by_client_id(aud)`. Returns the BodhiApp instance (tenant) that this token is scoped to.

### 3d. Access Request Scope Extraction

```
Filter scope string for tokens starting with "scope_access_request:"
Example: "openid profile email scope_access_request:ar-uuid"
         -> extracts "scope_access_request:ar-uuid"
```

### 3e. DB Lookup by Scope

If an access request scope was found:
```
db_service.get_by_access_request_scope(tenant_id, "scope_access_request:ar-uuid")
```

This uses the unique index on `(tenant_id, access_request_scope)`.

### 3f. Pre-Exchange Validations

| Check | Failure |
|-------|---------|
| Record exists | `AccessRequestValidation::ScopeNotFound` |
| `status == Approved` | `AccessRequestValidation::NotApproved` |
| `record.app_client_id == claims.azp` | `AccessRequestValidation::AppClientMismatch` |
| `record.user_id == claims.sub` | `AccessRequestValidation::UserMismatch` |

All failures result in 401/403 responses.

---

## Step 4: KC Token Exchange (RFC 8693)

### Build Exchange Scopes

Start with access request scopes, add standard OIDC scopes (`openid`, `email`, `profile`, `roles`) if present in original token.

### Exchange Call

```
auth_service.exchange_app_token(
  instance.client_id,      // BodhiApp's client ID
  instance.client_secret,  // BodhiApp's client secret
  external_token,          // Original token from external app
  exchange_scopes          // ["scope_access_request:ar-uuid", "openid", ...]
)
```

Keycloak performs RFC 8693 token exchange:
- Validates the external app's token
- Issues a new access token scoped to BodhiApp
- Injects `access_request_id` claim into the new token

---

## Step 5: Post-Exchange Validation

### 5a. Extract Claims from Exchanged Token

Parse the exchanged token's `ScopeClaims`, including the `access_request_id` claim.

### 5b. Verify access_request_id Claim

If a DB record was validated in pre-exchange:
- Exchanged token MUST have `access_request_id` claim
- Claim value MUST match `validated_record.id`
- Mismatch -> `AccessRequestValidation::AccessRequestIdMismatch`

### 5c. Role Derivation from DB

The external app's role comes from the **DB record's `approved_role`**, NOT from JWT claims:

```
role = validated_record.approved_role  // e.g., "scope_user_user"
  .parse::<UserScope>()               // -> UserScope::User
```

### 5d. Privilege Escalation Prevention

Even after KC exchange, verify the approved role doesn't exceed the user's actual capability:

```
user_resource_role = exchanged_token.resource_access[instance.client_id].roles
  -> parse to ResourceRole
max_scope = resource_role.max_user_scope()
if !max_scope.has_access_to(&approved_role) -> PrivilegeEscalation error
```

---

## Step 6: AuthContext Construction

```
AuthContext::ExternalApp {
  client_id: instance.client_id,        // BodhiApp's KC client ID
  tenant_id: instance.id,               // BodhiApp tenant ULID
  user_id: claims.sub,                  // User who approved
  role: Some(UserScope::User),          // From DB approved_role
  token: exchanged_access_token,        // New token from KC
  external_app_token: original_token,   // Original from external app
  app_client_id: claims.azp,            // External app's KC client ID
  access_request_id: Some("ar-uuid"),   // From exchanged token claim
}
```

This is inserted into request extensions and consumed by downstream middleware/handlers.

---

## Step 7: Caching

### Cache Key
```
"exchanged_token:{first_32_chars_of_sha256(token)}"
```

### Cached Data (CachedExchangeResult)

Stores: `token`, `client_id`, `tenant_id`, `app_client_id`, `role`, `access_request_id`, `cached_at`

### Cache TTL

5 minutes (`EXCHANGE_CACHE_TTL_SECS = 300`)

### Cache Hit Conditions

1. Entry exists
2. `cached_at` within TTL (not stale)
3. Inner token not expired
4. All valid -> reconstruct `AuthContext::ExternalApp` from cache, skip KC exchange

### Cache Miss

Full validation + KC exchange performed, result cached for next request.

---

## Step 8: api_auth_middleware

**File**: `crates/routes_app/src/middleware/apis/api_middleware.rs`

For `AuthContext::ExternalApp`:
- Checks `role.has_access_to(required_user_scope)`
- Most entity endpoints require `UserScope::User`
- `UserScope::PowerUser.has_access_to(UserScope::User)` = true (higher scope includes lower)
- Missing role -> `ApiAuthError::MissingAuth`

---

## Complete Request Example

```
External app sends:
  POST /bodhi/v1/toolsets/toolset-123/tools/search/execute
  Authorization: Bearer eyJ...{scope:"scope_access_request:ar-uuid", azp:"my-app", sub:"user-1"}

1. auth_middleware:
   - Strip X-BodhiApp-* headers
   - Extract bearer token
   - Call validate_bearer_token()

2. token_service.validate_bearer_token():
   - Not bodhiapp_ prefix -> external token path
   - Check expiry -> OK
   - Cache lookup -> MISS
   - Call handle_external_client_token()

3. handle_external_client_token() PRE-EXCHANGE:
   - Extract claims: azp="my-app", aud="bodhi-client", sub="user-1"
   - Verify issuer
   - Resolve tenant from aud
   - Extract scope: "scope_access_request:ar-uuid"
   - DB lookup: get_by_access_request_scope("tenant-id", "scope_access_request:ar-uuid")
   - Validate: status=Approved, app_client_id="my-app", user_id="user-1"

4. KC TOKEN EXCHANGE:
   - Exchange external token for scoped token
   - KC returns token with access_request_id claim

5. POST-EXCHANGE:
   - Verify access_request_id claim matches DB record
   - Read approved_role="scope_user_user" from DB -> UserScope::User
   - Verify UserScope::User <= user's max scope

6. Build AuthContext::ExternalApp, cache result

7. api_auth_middleware:
   - Required: UserScope::User
   - Have: UserScope::User -> OK

8. access_request_auth_middleware:
   - (see entity-enforcement-flow.md)
```

---

## Key Files

| File | Role |
|------|------|
| `crates/routes_app/src/middleware/auth/auth_middleware.rs` | Entry point, header extraction, session fallback |
| `crates/routes_app/src/middleware/token_service/token_service.rs` | Token classification, pre/post exchange, caching |
| `crates/routes_app/src/middleware/apis/api_middleware.rs` | Role-based authorization |
| `crates/services/src/auth/auth_context.rs` | AuthContext enum definition |
| `crates/services/src/auth/auth_service.rs` | exchange_app_token() KC call |
| `crates/services/src/auth/auth_objs.rs` | UserScope, ResourceRole |
| `crates/services/src/app_access_requests/access_request_repository.rs` | get_by_access_request_scope() |
