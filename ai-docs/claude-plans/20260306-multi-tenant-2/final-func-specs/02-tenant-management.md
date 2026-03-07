# Multi-Tenant Functional Specification — Tenant Management

> **Scope**: Tenant CRUD, lifecycle, membership, D65 creation limit, switching
> **Related specs**: [Index](00-index.md) · [Auth Flows](01-auth-flows.md) · [Info Endpoint](03-info-endpoint.md)
> **Decisions**: D52, D55, D60, D65, D66, D69, D78, D79, D82, D97

---

## Tenant Data Model

### Database Schema (BodhiApp)

```
tenants
├── id: String (ULID, primary key)
├── client_id: String (unique — Keycloak OAuth2 client ID)
├── encrypted_client_secret: Option<String>
├── salt_client_secret: Option<String>
├── nonce_client_secret: Option<String>
├── app_status: AppStatus (Setup | ResourceAdmin | Ready | TenantSelection)
├── created_by: Option<String> (Keycloak user ID / JWT sub claim) (D66)
├── created_at: DateTime<Utc>
└── updated_at: DateTime<Utc>
```

### Rust Types

```rust
// crates/services/src/tenants/tenant_objs.rs
pub struct Tenant {
  pub id: String,
  pub client_id: String,
  pub client_secret: String,     // Decrypted at read time
  pub status: AppStatus,
  pub created_by: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

pub enum AppStatus {
  Setup,            // No tenant exists yet
  ResourceAdmin,    // Standalone: tenant created, awaiting first login
  Ready,            // Fully operational
  TenantSelection,  // Multi-tenant: user must select/create a tenant
}
```

---

## Client ID Naming Convention (D82)

| Mode | Format | Example |
|------|--------|---------|
| Standalone | `bodhi-resource-<UUID>` | `bodhi-resource-a1b2c3d4-...` |
| Multi-tenant | `bodhi-tenant-<UUID>` | `bodhi-tenant-e5f6g7h8-...` |
| Test/dev | `test-resource-<UUID>` | `test-resource-12345678-...` |

Existing deployed clients keep their old IDs (no migration). Only new registrations use the new format.

---

## Standalone Tenant Creation {#standalone-creation}

### `POST /bodhi/v1/setup`

Creates the initial (and only) tenant for standalone mode.

**Precondition:** `AppStatus == Setup` (0 tenants in DB)

**Request:**
```json
{
  "name": "My Bodhi Server",
  "description": "Optional description"
}
```
- `name`: required, minimum 10 characters
- `description`: optional

**Flow:**
1. Validate app is in `Setup` status
2. Build redirect URIs from host header or `public_server_url()`
3. Call SPI: `POST /resources` (anonymous — no Authorization header)
4. SPI creates Keycloak client → returns `{ client_id, client_secret }`
5. SPI also creates `bodhi_clients` row with null `multi_tenant_client_id` (D97)
6. BodhiApp creates tenant row: `{ client_id, encrypted_client_secret, status: ResourceAdmin }`
7. Return setup response with new status

**Response (200):**
```json
{
  "status": "resource_admin"
}
```

**Lifecycle after setup:**
```
POST /setup → status: ResourceAdmin
POST /auth/initiate { client_id } → OAuth login
POST /auth/callback → make_resource_admin + set_client_ready(client_id, user_id)
→ status: Ready, created_by: user_id
```

---

## Multi-Tenant Tenant Creation {#multi-tenant-creation}

### `POST /bodhi/v1/tenants`

Creates a new tenant via SPI proxy. **Multi-tenant only** (D101).

**Precondition:** Valid dashboard token in session

**Request:**
```json
{
  "name": "My Team Workspace",
  "description": "Optional team description"
}
```
- `name`: required, 1–255 characters
- `description`: optional, max 1000 characters

> **TECHDEBT** [Description Optionality]: Backend accepts `description` as optional (`Option<String>`). Frontend registration form (`/ui/setup/tenants/`) requires it (min 1 char validation). This mismatch should be resolved — frontend should make it optional to match the API contract. See [TECHDEBT.md](../TECHDEBT.md).

**Flow:**
1. Validate `is_multi_tenant()` (D101)
2. `ensure_valid_dashboard_token()` — refresh if expired
3. Build redirect URIs: `["{public_server_url()}/ui/auth/callback"]` (D57, D60)
4. Call SPI: `POST /realms/{realm}/bodhi/tenants` with dashboard token
   - Body: `{ name, description, redirect_uris }`
5. SPI creates Keycloak client + `bodhi_clients` row → returns `{ client_id, client_secret }`
6. Extract `user_id` from dashboard JWT `sub` claim (D66)
7. BodhiApp creates tenant row: `{ client_id, encrypted_client_secret, status: Ready, created_by: user_id }` (D69, D79)
8. If local row creation fails: log error, continue (D52 — accept orphan)
9. Return `{ client_id }`

**Response (201):**
```json
{
  "client_id": "bodhi-tenant-a1b2c3d4-..."
}
```

**After creation:**
- Tenant status is `Ready` immediately — no setup wizard (D79)
- Frontend auto-initiates resource OAuth: `POST /auth/initiate { client_id }`
- After OAuth: user lands in `/ui/chat`

---

## D65: Creation Limit {#creation-limit}

**One tenant creation per user** (D65):
- A user can CREATE at most 1 tenant (tracked by `created_by` column matching JWT `sub`)
- A user can be a MEMBER of many tenants (via admin invite or access request approval)
- The limit is enforced at the SPI level
- Hard limit for initial SaaS launch, expansion deferred

**Membership via other paths:**
- Admin invites user via `/resources/assign-role` SPI endpoint → user appears in `GET /tenants` list
- Access request flow → user approved for tenant membership

---

## Tenant Listing {#tenant-listing}

### `GET /bodhi/v1/tenants`

Lists tenants the current user can access. **Multi-tenant only**.

**Precondition:** Valid dashboard token in session

**Flow:**
1. `ensure_valid_dashboard_token()` — refresh if expired (D53)
2. Call SPI: `GET /realms/{realm}/bodhi/tenants` with dashboard token (D78 — SPI is source of truth)
3. For each tenant in SPI response, enrich with session state:
   - `is_active`: `client_id == session["active_client_id"]`
   - `logged_in`: `session["{client_id}:access_token"]` exists and is not expired

**Response (200):**
```json
{
  "tenants": [
    {
      "client_id": "bodhi-tenant-a1b2c3d4-...",
      "name": "My Workspace",
      "description": "Team workspace",
      "is_active": true,
      "logged_in": true
    },
    {
      "client_id": "bodhi-tenant-e5f6g7h8-...",
      "name": "Other Workspace",
      "description": null,
      "is_active": false,
      "logged_in": false
    }
  ]
}
```

**TypeScript types:**
```typescript
interface TenantListItem {
  client_id: string;
  name: string;
  description?: string | null;
  is_active: boolean;
  logged_in: boolean;
}

interface TenantListResponse {
  tenants: TenantListItem[];
}
```

---

## Tenant Switching {#tenant-switching}

### Initial Selection (after dashboard login)

The `/ui/login` page handles initial tenant selection:

| Tenants | Behavior |
|---------|----------|
| 0 | Redirect to `/ui/setup/tenants/` (registration form) |
| 1 | Auto-initiate resource OAuth (seamless, no user interaction) |
| N | Show tenant selector dropdown |

### Switching to Another Tenant

From `/ui/login` or tenant dropdown:

**Case A: Target tenant has cached token (`logged_in: true`)**

### `POST /bodhi/v1/tenants/{client_id}/activate` {#tenant-activate}

Instant switch — no OAuth required.

**Request:** empty body (client_id in URL path)

**Flow:**
1. Validate `is_multi_tenant()` (D101)
2. Check session for `{client_id}:access_token` — must exist
3. If missing → return `TenantNotLoggedIn` error
4. Set `session["active_client_id"] = client_id`
5. Return 200

**Response (200):** empty

**Errors:**
- `400` — `tenant_not_logged_in`: no cached token for this tenant

**Case B: Target tenant has no cached token (`logged_in: false`)**

Requires OAuth flow:
1. `POST /auth/initiate { client_id: "target-tenant-id" }`
2. Keycloak SSO session reuse → instant redirect (no password)
3. `POST /auth/callback { code, state }` → sets new `active_client_id`

---

## User Management

### Role Hierarchy

Roles come from Keycloak group membership, available in JWT `resource_access` claims after login (D95):

| Role | Level | Can access |
|------|-------|-----------|
| `resource_admin` | Highest | All operations |
| `resource_manager` | High | User management, all below |
| `resource_power_user` | Medium | Model CRUD, tokens, all below |
| `resource_user` | Base | Read models, use chat, MCPs, toolsets |

### Adding Users to a Tenant

Users gain tenant membership through:
1. **Tenant creation**: Creator automatically becomes admin
2. **Admin invite**: Via SPI `/resources/assign-role` → adds to Keycloak groups + `bodhi_clients_users`
3. **Access request**: External app requests access → user approves → membership added

The `GET /tenants` SPI endpoint returns all tenants where the user has membership (D78).

---

## Frontend Pages

### Tenant Registration — `/ui/setup/tenants/`

- Shown when user has dashboard session but 0 tenants
- Form fields: `name` (required, min 1, max 255), `description` (required on frontend, min 1, max 1000)
- Uses `useCreateTenant()` hook → `POST /bodhi/v1/tenants`
- On success: auto-initiates resource OAuth with returned `client_id`

### Login / Tenant Selector — `/ui/login`

- `AppInitializer allowedStatus={['ready', 'tenant_selection']}`

**Standalone mode:**
- Not logged in: "Login" button → `POST /auth/initiate { client_id }` (client_id from `/info`)
- Logged in: user info + home/logout buttons

**Multi-tenant mode (`MultiTenantLoginContent`):**
- No dashboard session: "Login to Bodhi Platform" button → dashboard OAuth
- Dashboard session, 0 tenants: auto-redirect to `/ui/setup/tenants/`
- Dashboard session, 1 tenant: auto-initiate resource OAuth
- Dashboard session, N tenants: tenant dropdown with `is_active`/`logged_in` status, Connect button per tenant
- Fully authenticated: current tenant info + switch dropdown + logout

### Frontend Hooks

```typescript
// useTenants.ts
useTenants(options?: { enabled?: boolean })     // GET /bodhi/v1/tenants
useCreateTenant()                                // POST /bodhi/v1/tenants
useTenantActivate()                              // POST /bodhi/v1/tenants/{client_id}/activate

// useAuth.ts
useDashboardOAuthInitiate()                      // POST /bodhi/v1/auth/dashboard/initiate
useDashboardOAuthCallback()                      // POST /bodhi/v1/auth/dashboard/callback
useOAuthInitiate(vars: { client_id: string })    // POST /bodhi/v1/auth/initiate
```

---

## SPI (Keycloak Server Provider Interface)

The SPI is treated as an opaque dependency. BodhiApp proxies requests to it via `AuthService`. Key behaviors:

- **Tenant creation**: SPI creates Keycloak client + `bodhi_clients` row + group membership
- **Tenant listing**: SPI returns clients where user has membership in `bodhi_clients_users`
- **Role assignment**: SPI manages Keycloak group membership (source of truth for roles)
- **Errors**: Proxied as OpenAI-compatible error format (D64)

SPI tables (`bodhi_clients`, `bodhi_clients_users`) are managed by the SPI via JPA + Liquibase (D59, D71). BodhiApp does not query them directly.

---

## TECHDEBT

> **TECHDEBT** [Description Optionality]: `POST /bodhi/v1/tenants` accepts `description` as `Option<String>`, but the frontend registration form requires it (min 1 char). Frontend should be updated to make description optional, matching the API contract. See [TECHDEBT.md](../TECHDEBT.md).
