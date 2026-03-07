# Multi-Tenant Keycloak SPI Extensions — ✅ COMPLETED

> **Created**: 2026-03-06
> **Completed**: 2026-03-08
> **Target repo**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`
> **Integration doc**: `keycloak-bodhi-ext/ai-docs/claude-plans/20260307-multi-tenant/20260307-tenants-integration-doc.md`

---

## Summary

The Keycloak SPI was extended to support multi-tenant SaaS. Users authenticate against a dashboard client (`bodhi.client_type=multi-tenant`), then the SPI provides their tenant list and handles tenant creation.

### New SPI Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| POST | `/realms/{realm}/bodhi/tenants` | User token (azp = dashboard client) | Create a tenant resource client |
| GET | `/realms/{realm}/bodhi/tenants` | User token (azp = dashboard client) | List tenants visible to current user |

### New Client Type: `multi-tenant`

Dashboard clients have attribute `bodhi.client_type=multi-tenant`. They are NOT created via SPI — must be provisioned by Keycloak admin (realm import or admin console).

### Client ID Prefix Changes

| Context | Old Prefix | New Prefix |
|---------|-----------|------------|
| Standalone resource (production) | `resource-` | `bodhi-resource-` |
| Standalone resource (test) | `test-resource-` | `test-resource-` (unchanged) |
| Multi-tenant resource (production) | n/a | `bodhi-tenant-` |
| Multi-tenant resource (test) | n/a | `test-tenant-` |
| App client (production) | `app-` | `bodhi-app-` |

Both standalone and multi-tenant resources share `bodhi.client_type=resource`. The prefix distinguishes their origin.

---

## Database Tables

### `bodhi_clients` — Resource client tracking

One row per resource client created through `POST /resources` or `POST /tenants`.

```
id                      VARCHAR(36)   PK, app-generated UUID
realm_id                VARCHAR(255)  NOT NULL
client_id               VARCHAR(255)  NOT NULL, UNIQUE — OAuth2 string client_id
multi_tenant_client_id  VARCHAR(255)  nullable — dashboard's string client_id (NULL for standalone)
created_by_user_id      VARCHAR(36)   nullable — Keycloak user UUID
created_at              TIMESTAMP     NOT NULL
updated_at              TIMESTAMP     NOT NULL
```

Index: `(realm_id, multi_tenant_client_id)` — used by GET /tenants queries.

### `bodhi_clients_users` — Membership proxy

One row per (client_id, user_id) pair. Presence = user has membership; absence = no membership. The user's actual role comes from Keycloak group membership (source of truth), not from this table.

```
id          VARCHAR(36)   PK, app-generated UUID
realm_id    VARCHAR(255)  NOT NULL
client_id   VARCHAR(255)  NOT NULL — OAuth2 string client_id
user_id     VARCHAR(36)   NOT NULL — Keycloak user UUID
created_at  TIMESTAMP     NOT NULL
updated_at  TIMESTAMP     NOT NULL

UNIQUE(client_id, user_id)
```

Index: `(realm_id, user_id)` — used for user-centric lookups.

**Why string client_ids (not Keycloak internal UUIDs)**: Token's `azp` claim already contains the string client_id — zero resolution overhead on create and lookup.

### Role Management

Roles are managed via Keycloak groups (source of truth):
- Top-level group `users-{clientId}` with subgroups: `users`, `power-users`, `managers`, `admins`
- Each subgroup has Keycloak client roles assigned
- Users join groups to get roles
- `bodhi_clients_users` only tracks membership (no role column) for fast `GET /tenants` queries

4-level hierarchy: `resource_admin > resource_manager > resource_power_user > resource_user`

---

## Dual-Write Behavior

All role mutation endpoints write to both Keycloak groups AND `bodhi_clients_users`:

| Endpoint | Keycloak Group Change | bodhi_clients / bodhi_clients_users Change |
|----------|----------------------|--------------------------------------------|
| POST /resources | — | INSERT `bodhi_clients` (null multi_tenant, null created_by) |
| POST /resources/make-resource-admin | User -> admins group | INSERT `bodhi_clients_users` membership + SET `bodhi_clients.created_by_user_id` |
| POST /resources/assign-role | User -> target group | INSERT `bodhi_clients_users` membership (if not exists) |
| POST /resources/remove-user | User removed from all groups | DELETE `bodhi_clients_users` membership |
| POST /tenants | Creator -> admins group | INSERT `bodhi_clients` + INSERT `bodhi_clients_users` membership |

---

## Endpoint Details

### POST /tenants — Create Tenant Resource Client

**Auth**: User token where `azp` = dashboard client with `bodhi.client_type=multi-tenant`

**Request**:
```json
{
  "name": "string (required)",
  "description": "string (optional)",
  "redirect_uris": ["string (required)"]
}
```

**Response (201 Created)**:
```json
{
  "client_id": "bodhi-tenant-{uuid}",
  "client_secret": "generated-secret"
}
```

**Side effects**:
- Creates full resource client (roles, groups, service account) with `bodhi.client_type=resource`
- Inserts into `bodhi_clients` with `multi_tenant_client_id` = dashboard's client_id, `created_by_user_id` = token's `sub`
- Makes creating user `resource_admin` (joins admins group + inserts `bodhi_clients_users` membership)

**One-per-user constraint**: Each user can create at most one tenant per dashboard client. Application-level check using `bodhi_clients` table.

**Errors**:

| Status | Message | Cause |
|--------|---------|-------|
| 400 | `"name is required"` | Missing name |
| 401 | `"invalid session"` | No or invalid bearer token |
| 401 | `"service account tokens not allowed"` | Service account token used |
| 401 | `"dashboard client not found"` | Token's azp client doesn't exist |
| 401 | `"token is not from a valid dashboard client"` | azp doesn't have `bodhi.client_type=multi-tenant` |
| 409 | `"user already has a tenant for this dashboard"` | One-per-user constraint |

### GET /tenants — List Tenant Resource Clients

**Auth**: User token where `azp` = dashboard client with `bodhi.client_type=multi-tenant`

**Response (200)**:
```json
{
  "tenants": [
    {
      "client_id": "bodhi-tenant-abc123",
      "name": "My Workspace",
      "description": "A team workspace"
    }
  ]
}
```

- No role field — roles live in Keycloak groups; visible after tenant login via JWT claims
- `description` may be null

**Visibility rule**: Tenant appears only if `multi_tenant_client_id` matches dashboard AND `bodhi_clients_users` row exists for (client_id, user_id).

**Errors**: Same auth errors as POST /tenants.

### Authentication Pattern: `checkForDashboardToken`

1. Auth result must be valid -> 401 `"invalid session"`
2. No `client_id` in `otherClaims` (not a service account) -> 401 `"service account tokens not allowed"`
3. Token's `azp` must resolve to an existing client -> 401 `"dashboard client not found"`
4. That client must have `bodhi.client_type=multi-tenant` -> 401 `"token is not from a valid dashboard client"`

---

## Codebase Notes

- **JPA entities**: Registered via existing `BodhiJpaEntityProvider`
- **Tables**: Created via Liquibase changelog in `META-INF/bodhi-changelog.xml`
- **Services**: `ResourceService`, `RoleAssignmentService`, `AppClientService` are separate classes instantiated in `ResourceManagementService`
- **Client type validation**: Via `bodhi.client_type` attribute — `multi-tenant` added alongside `resource` and `app`
- **Tests**: Follow existing patterns (unit + Testcontainers integration)
- **Deployed to**: `main-id.getbodhi.app` dev Keycloak

---

## Full Contract Summary

| Operation | Method + Path | Auth | Response |
|-----------|--------------|------|----------|
| Create tenant | `POST /bodhi/tenants` | User token (azp=dashboard) | 201: `{client_id, client_secret}` |
| List tenants | `GET /bodhi/tenants` | User token (azp=dashboard) | 200: `{tenants: [{client_id, name, description}]}` |
| Create resource | `POST /bodhi/resources` | None | `{client_id, client_secret}` |
| Check admin | `GET /bodhi/resources/has-resource-admin` | Service account | `{has_admin}` |
| Make admin | `POST /bodhi/resources/make-resource-admin` | Service account | 201 |
| Assign role | `POST /bodhi/resources/assign-role` | SA or user (admin/manager) | 200 |
| Remove user | `POST /bodhi/resources/remove-user` | SA or user (admin/manager) | 200 |

All errors return: `{ "error": "descriptive message" }`.
