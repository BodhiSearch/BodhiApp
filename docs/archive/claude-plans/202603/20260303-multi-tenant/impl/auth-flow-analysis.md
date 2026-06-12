# Auth Flow Analysis for Multi-Tenancy

> **Purpose**: Documents current auth flow, proposed changes for multi-tenancy,
> and the deferred two-phase auth flow for multi-tenant login.
>
> **Created**: 2026-03-03
> **Update when**: Auth flow decisions change or implementation reveals new requirements.

---

## Current Auth Flow (Standalone)

### AuthContext Enum (4 variants)

```
AuthContext::Anonymous     { client_id: Option<String> }
AuthContext::Session       { client_id, user_id, username, role, token }
AuthContext::ApiToken      { client_id, user_id, role, token }
AuthContext::ExternalApp   { client_id, user_id, role, token, external_app_token, app_client_id, access_request_id }
```

- `client_id` = Bodhi's own registered OAuth client (from tenants/apps table)
- `app_client_id` = External app's client (from JWT `azp` claim, ExternalApp only)

### Standalone Auth Flow

```
1. User visits app → GET /bodhi/v1/info
2. If status=Setup → setup wizard → POST /bodhi/v1/setup
   - Registers OAuth client with Keycloak
   - Creates tenants row (client_id, encrypted_client_secret)
   - Returns AppStatus::ResourceAdmin
3. User clicks Login → POST /bodhi/v1/auth/initiate
   - Generates PKCE challenge, stores state in session
   - Returns Keycloak authorization URL
4. User authenticates at Keycloak
5. Callback → GET /ui/auth/callback → POST /bodhi/v1/auth/callback
   - Exchanges code for tokens
   - Stores tokens in session
6. Subsequent requests:
   - Session cookie → auth_middleware extracts tokens
   - Validates with Keycloak → constructs AuthContext::Session
   - AuthScope extractor wraps into AuthScopedAppService
```

### Middleware Functions

| Function | Purpose | Returns |
|----------|---------|---------|
| `auth_middleware` | Strict auth — requires valid credentials | Error on failure |
| `optional_auth_middleware` | Permissive — Anonymous on failure | AuthContext::Anonymous on failure |
| `api_auth_middleware` | Authorization — role checks | Error if role insufficient |
| `access_request_auth_middleware` | Entity-level auth for external apps | Error if access not approved |

### JWT Claims Used

- `azp` (authorized party) — identifies the OAuth client that requested the token
- `resource_access.<client-id>.roles` — client-scoped roles
- `sub` — user ID
- `preferred_username` — username

---

## Proposed Changes for Multi-Tenancy

### AuthContext with tenant_id

```
AuthContext::Anonymous     { client_id: Option<String>, tenant_id: Option<String> }
AuthContext::Session       { client_id, tenant_id, user_id, username, role, token }
AuthContext::ApiToken      { client_id, tenant_id, user_id, role, token }
AuthContext::ExternalApp   { client_id, tenant_id, user_id, role, token, ... }
```

New accessors:
- `tenant_id() -> Option<&str>`
- `require_tenant_id() -> Result<&str, AuthContextError>`

New error variant:
- `AuthContextError::MissingTenantId` (ErrorType::InternalServer)

### Middleware Tenant Resolution

```
1. JWT arrives with azp: "bodhi-acme-corp"
2. Middleware looks up: SELECT id FROM tenants WHERE client_id = 'bodhi-acme-corp'
   (cached in-memory — tenants table mapping is near-static)
3. tenant_id ULID injected into AuthContext
4. Auth-scoped services use tenant_id for all DB queries
```

For standalone mode:
- One tenant row exists
- client_id in JWT matches the single tenant's client_id
- Resolution always returns the same tenant_id
- Behavior identical to pre-multi-tenant

For multi-tenant mode:
- Multiple tenant rows exist
- JWT azp claim identifies which tenant the user authenticated against
- Resolution returns that tenant's ULID
- All queries scoped to that tenant

### Tenant Lookup Caching

The tenants table is near-static (tenants are rarely created/deleted). Cache the `client_id → tenant_id` mapping in MokaCacheService with a TTL of ~5 minutes. Cache invalidation on tenant create/update.

---

## Deferred: Multi-Tenant Login Flow

> **Status**: Not in current plan. Documented here for future implementation.

### Two-Phase Auth Flow

**Phase 1: Platform Authentication**
```
1. User visits app.getbodhi.ai → sees login page
2. Clicks "Login" → auth initiated against BODHI_MULTITENANT_CLIENT_ID (platform client)
3. Authenticates at Keycloak → returns to app with platform token
4. Platform token grants access to: GET /bodhi/v1/tenants (list user's tenants)
5. User sees tenant selector UI
```

**Phase 2: Tenant Authentication**
```
1. User selects a tenant (e.g., "Acme Corp")
2. Frontend initiates new auth flow against tenant's Keycloak client
3. Keycloak reuses SSO session → issues new token with azp=bodhi-acme-corp
   (no password re-entry needed)
4. Frontend stores tenant token, discards platform token
5. All subsequent API calls use tenant token
6. Active tenant stored in cookie for session persistence
```

### New Backend Endpoints Needed

| Endpoint | Auth | Purpose |
|----------|------|---------|
| `POST /bodhi/v1/auth/platform/initiate` | None | Start platform auth flow |
| `POST /bodhi/v1/auth/platform/callback` | None | Exchange platform auth code |
| `GET /bodhi/v1/tenants` | Platform token | List tenants user has access to |
| `POST /bodhi/v1/auth/tenant/initiate` | Platform token | Start tenant-specific auth |
| `POST /bodhi/v1/auth/tenant/callback` | None | Exchange tenant auth code |

### Keycloak Configuration

Each tenant in the `tenants` table maps to a Keycloak client:
- Platform client: BODHI_MULTITENANT_CLIENT_ID (e.g., "bodhi-platform")
- Tenant clients: tenants.client_id (e.g., "bodhi-acme-corp")

User's tenant membership determined by:
- Keycloak client roles (user has roles in tenant's client → has access)
- Or: Application-level membership table (future)

### Session-Based Tenant Routing (Decision D13)

Active tenant stored in cookie, not URL path. No slug column needed.

```
Cookie: bodhi_active_tenant=<tenant_id>
Authorization: Bearer <tenant-specific-jwt>
```

The JWT azp claim is the source of truth for tenant identity, not the cookie.
The cookie is for UI convenience (remembering which tenant was last active).

---

## Key Files

| File | Purpose |
|------|---------|
| `crates/services/src/auth/auth_context.rs` | AuthContext enum definition |
| `crates/auth_middleware/src/auth_middleware/middleware.rs` | 4 middleware functions |
| `crates/auth_middleware/src/token_service/service.rs` | JWT validation, token exchange |
| `crates/routes_app/src/shared/auth_scope_extractor.rs` | AuthScope Axum extractor |
| `crates/services/src/app_service/auth_scoped.rs` | AuthScopedAppService |
| `crates/routes_app/src/auth/` | Auth route handlers |
