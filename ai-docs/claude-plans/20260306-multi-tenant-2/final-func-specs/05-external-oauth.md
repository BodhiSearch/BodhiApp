# Multi-Tenant Functional Specification — External OAuth

> **Scope**: Access request flow, external app auth, token exchange, multi-tenant considerations
> **Related specs**: [Index](00-index.md) · [Auth Flows](01-auth-flows.md) · [API Tokens](04-api-tokens.md)
> **Decisions**: D24, D27

---

## Overview

External applications (IDE plugins, browser extensions, CLI tools) access BodhiApp resources through an access request + token exchange flow. The external app requests access, a BodhiApp user reviews and approves it, and the app receives a JWT that can be used to call BodhiApp APIs.

**Flow summary:**
```
External App                    BodhiApp                      User
     │                              │                           │
     │─── POST /apps/request-access ──▶│                        │
     │◀── { id, review_url } ───────│                           │
     │                              │                           │
     │    (app shows review_url     │                           │
     │     or opens popup)          │                           │
     │                              │◀── user visits review_url ─│
     │                              │─── shows review page ─────▶│
     │                              │◀── approve/deny ──────────│
     │                              │                           │
     │─── GET /apps/access-requests/{id} (poll) ──▶│            │
     │◀── { status: "approved" } ───│                           │
     │                              │                           │
     │─── API calls with JWT ───────▶│                          │
```

---

## Access Request Creation (Desired End-State) {#access-request-creation}

### `POST /bodhi/v1/apps/request-access`

Creates a new access request. **Public endpoint** — no authentication required.

**Desired behavior**: Access request is created tenant-agnostic (no `tenant_id`). The reviewing user selects which tenant to associate it with during review.

**Request:**
```json
{
  "app_client_id": "my-ide-plugin",
  "flow_type": "redirect",
  "redirect_url": "http://localhost:8080/callback",
  "requested_role": "resource_user",
  "requested": {
    "toolset_types": ["web_search", "code_interpreter"],
    "mcp_servers": ["server-uuid-1"]
  }
}
```

- `app_client_id`: required, unique identifier for the external application
- `flow_type`: required, `"redirect"` or `"popup"`
- `redirect_url`: required if `flow_type == "redirect"`, where to send the user after review
- `requested_role`: required, role the app is requesting
- `requested`: optional, specific resources the app wants access to

**Response (201):**
```json
{
  "id": "access-request-ulid",
  "status": "draft",
  "review_url": "https://bodhi.example.com/ui/access-requests/access-request-ulid/review"
}
```

**Status transitions:**
```
[draft] ──approve──▶ [approved]
[draft] ──deny─────▶ [denied]
[draft] ──error────▶ [failed]
```

> **TECHDEBT** [get_standalone_app]: `apps_create_access_request` at `routes_app/src/apps/routes_apps.rs:95` calls `get_standalone_app()` to get tenant_id. This breaks with >1 tenant in multi-tenant mode. Should use tenant-agnostic creation where user selects tenant during review. HIGH priority. See [TECHDEBT.md](../TECHDEBT.md).

---

## Polling {#polling}

### `GET /bodhi/v1/apps/access-requests/{id}`

External app polls for status changes. **Public endpoint**.

**Query params:**
- `app_client_id`: required, must match the `app_client_id` used at creation (security check)

**Response (200):**
```json
{
  "id": "access-request-ulid",
  "status": "approved",
  "requested_role": "resource_user",
  "approved_role": "resource_user",
  "access_request_scope": "scope_access_request:access-request-ulid"
}
```

- `status`: `"draft"` | `"approved"` | `"denied"` | `"failed"`
- `approved_role`: only present when `status == "approved"`, may differ from `requested_role`
- `access_request_scope`: present when approved with tool/MCP access, format `scope_access_request:{id}`

---

## Review Flow {#review-flow}

### `GET /bodhi/v1/access-requests/{id}/review`

User reviews the access request. **Session auth required** (User+ role).

**Response (200):**
```json
{
  "id": "access-request-ulid",
  "app_client_id": "my-ide-plugin",
  "flow_type": "redirect",
  "redirect_url": "http://localhost:8080/callback",
  "status": "draft",
  "requested_role": "resource_user",
  "requested": {
    "toolset_types": ["web_search"],
    "mcp_servers": ["server-uuid-1"]
  },
  "available_toolsets": [...],
  "available_mcps": [...]
}
```

The review page shows:
- What the app is requesting (role, resources)
- Available toolsets and MCPs the user can grant
- Approve/deny controls

---

## Approval {#approval}

### `PUT /bodhi/v1/access-requests/{id}/approve`

User approves the access request. **Session auth required** (User+ role).

**Request:**
```json
{
  "approved_role": "resource_user",
  "approved_resources": {
    "toolset_ids": ["toolset-ulid-1"],
    "mcp_ids": ["mcp-ulid-1"]
  }
}
```

- `approved_role`: may differ from `requested_role` (user can downgrade)
- `approved_resources`: specific instances the app gets access to

**Response (200):**
```json
{
  "id": "access-request-ulid",
  "status": "approved"
}
```

**Side effects:**
- Access request status → `approved`
- Generates `access_request_scope` for entity-level access control

---

## Denial {#denial}

### `POST /bodhi/v1/access-requests/{id}/deny`

User denies the access request. **Session auth required** (User+ role).

**Request:** empty body

**Response (200):**
```json
{
  "id": "access-request-ulid",
  "status": "denied"
}
```

---

## Token Exchange

After approval, the external app uses its credentials to call BodhiApp APIs.

### External App Token Format

The external app presents a JWT in the `Authorization: Bearer` header. This JWT:
- Has `aud` (audience) set to the tenant's `client_id`
- Has `access_request_id` in its claims
- Is issued by the same Keycloak realm as BodhiApp

### Middleware Resolution

When the auth middleware encounters a non-`bodhiapp_` Bearer token:

```
1. Verify JWT issuer matches BODHI_AUTH_ISSUER (D24)
2. Extract aud claim → get_tenant_by_client_id(aud) → tenant resolution
3. Use tenant's client_secret for token validation
4. Extract access_request_id from claims
5. Build AuthContext::ExternalApp {
     client_id: aud,
     tenant_id: tenant.id,
     user_id: from_claims,
     role: from_access_request_db,
     token: raw_jwt,
     external_app_token: original_token,
     app_client_id: from_claims,
     access_request_id: from_claims,
   }
```

### Access-Request-Gated Endpoints

Some endpoints (toolset execution, MCP tool execution) have an additional `access_request_middleware` layer that validates:
- The `access_request_id` in `AuthContext` is still `approved`
- The specific resource (toolset/MCP) was included in the approved resources

---

## Multi-Tenant Considerations

### Same Flow, Both Modes

The access request flow is identical in standalone and multi-tenant modes (D27):
- **Standalone**: Single tenant exists, `get_standalone_app()` provides tenant_id (current implementation)
- **Multi-tenant (desired)**: Access request created without tenant_id. User selects tenant during review.

### Tenant Resolution via JWT `aud`

External app JWTs include `aud = tenant_client_id`. The middleware resolves the tenant from this claim. This works identically in both modes — the JWT always targets a specific tenant.

### Current vs Desired Behavior

| Aspect | Current | Desired |
|--------|---------|---------|
| Tenant at creation | `get_standalone_app()` → fails with >1 tenant | Tenant-agnostic, user selects during review |
| Review flow | Tenant pre-selected | User selects tenant from dropdown |
| Token exchange | Works (aud-based) | Same |

---

## TECHDEBT

> **TECHDEBT** [get_standalone_app]: `apps_create_access_request` in `routes_app/src/apps/routes_apps.rs:95` calls `get_standalone_app()` to get `tenant_id`. This breaks in multi-tenant mode with >1 tenant (`DbError::MultipleTenant`). Should be refactored to create access requests tenant-agnostic, with user selecting tenant during review. HIGH priority. See [TECHDEBT.md](../TECHDEBT.md).
