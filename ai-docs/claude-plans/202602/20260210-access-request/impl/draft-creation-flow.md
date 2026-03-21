# Draft Creation Flow

> **Purpose**: Full-stack trace of how an external app creates an access request draft and polls its status.
> Covers: POST /apps/request-access, service.create_draft(), status polling endpoint.

---

## Endpoints

| Endpoint | Method | Auth | Handler |
|----------|--------|------|---------|
| `/bodhi/v1/apps/request-access` | POST | None (public) | `apps_create_access_request` |
| `/bodhi/v1/apps/access-requests/{id}` | GET | None (public) | `apps_get_access_request_status` |

Both registered in the **public APIs** router in `crates/routes_app/src/routes.rs` — no auth middleware applied.

---

## Draft Creation: POST /apps/request-access

### Route Handler

**File**: `crates/routes_app/src/apps/routes_apps.rs`

**Signature**: `apps_create_access_request(auth_scope: AuthScope, ValidatedJson<CreateAccessRequest>)`

**Request Body** (from `apps_api_schemas.rs`):
```json
{
  "app_client_id": "my-third-party-app",
  "flow_type": "popup",
  "redirect_url": "https://myapp.com/callback",
  "requested_role": "scope_user_user",
  "requested": {
    "toolset_types": [{"toolset_type": "builtin-exa-search"}],
    "mcp_servers": [{"url": "https://mcp.example.com/mcp"}]
  }
}
```

### Handler Logic

1. **Validate redirect_url**: Required if `flow_type == Redirect` -> `AppsRouteError::MissingRedirectUrl`

2. **Extract requested resources**: Parse optional `requested` field; default to empty vectors

3. **Validate tool types**: For each requested toolset type, calls `auth_scope.tools().validate_type(&toolset_type)` to verify the type exists in the system. Returns `AppsRouteError::InvalidToolType` if unknown.

4. **Call service**: `access_request_service.create_draft(...)` with all parameters

5. **Build review URL**: `access_request_service.build_review_url(&created.id)` -> `"{frontend_url}/ui/apps/access-requests/review?id={id}"`

6. **Return 201**: `CreateAccessRequestResponse { id, status: Draft, review_url }`

### Service Layer: create_draft()

**File**: `crates/services/src/app_access_requests/access_request_service.rs`

1. **Re-validate redirect_uri** for redirect flow (defense in depth)
2. **Generate ULID**: `new_ulid()` for the access request ID
3. **Calculate expiry**: `time_service.utc_now() + Duration::minutes(10)`
4. **Serialize requested resources**: JSON string of `{"toolset_types": [...], "mcp_servers": [...]}`
5. **Modify redirect_uri**: Appends `?id=<request_id>` (or `&id=<request_id>` if URI already has query params) — enables the external app to extract the ID after redirect
6. **Build row**: `AppAccessRequest` with:
   - `tenant_id: None` — not bound to any tenant
   - `user_id: None` — no user involved yet
   - `status: Draft`
   - `approved: None`, `approved_role: None`, `access_request_scope: None`
7. **Insert via repository**: `db_service.create(&row)`
8. **Return** created row

### Database Insert

Repository `create()` uses `with_tenant_txn()` with empty string for tenant_id (maps to NULL in the row). On PostgreSQL, RLS allows INSERT with `tenant_id IS NULL`.

---

## Status Polling: GET /apps/access-requests/{id}

### Route Handler

**File**: `crates/routes_app/src/apps/routes_apps.rs`

**Signature**: `apps_get_access_request_status(auth_scope, Path(id), Query(AccessRequestStatusQuery))`

**Query Parameter**: `app_client_id` — required for security

### Handler Logic

1. **Fetch request**: `access_request_service.get_request(&id)`
2. **Verify app_client_id**: If `request.app_client_id != query.app_client_id` -> return 404 (no info leak)
3. **Parse roles**: Convert `requested_role` and `approved_role` strings to `UserScope` enums
4. **Return**: `AccessRequestStatusResponse { id, status, requested_role, approved_role, access_request_scope }`

### Auto-Expiry on Read

The repository `get()` method performs lazy expiry:
- If the record is Draft and `expires_at < now`, it updates the status to `Expired` in the database before returning
- This means expired drafts are auto-transitioned without a background job

### Security Model

- The endpoint is unauthenticated — anyone with the ID and matching app_client_id can poll
- The ULID is effectively a capability token (unguessable)
- `app_client_id` verification prevents one app from polling another app's requests
- Mismatch returns 404 (not 403) to prevent ID enumeration

---

## Response Shapes

### Draft (just created)
```json
{
  "id": "01HXYZ...",
  "status": "Draft",
  "requested_role": "scope_user_user",
  "approved_role": null,
  "access_request_scope": null
}
```

### Approved (after user approval)
```json
{
  "id": "01HXYZ...",
  "status": "Approved",
  "requested_role": "scope_user_user",
  "approved_role": "scope_user_user",
  "access_request_scope": "scope_access_request:01HXYZ..."
}
```

The external app uses `access_request_scope` to include in its OAuth authorization request to Keycloak.

---

## Error Paths

| Condition | Error | HTTP |
|-----------|-------|------|
| `flow_type == Redirect` without `redirect_url` | `MissingRedirectUrl` | 400 |
| Unknown toolset type in `requested` | `InvalidToolType` | 400 |
| Service-level validation failure | `AccessRequestError::*` | varies |
| Poll with wrong `app_client_id` | 404 (treated as not found) | 404 |
| Poll for expired draft | Returns with `status: Expired` | 200 |

---

## Key Files

| File | Role |
|------|------|
| `crates/routes_app/src/apps/routes_apps.rs` | Handler implementations |
| `crates/routes_app/src/apps/apps_api_schemas.rs` | Request/response DTOs |
| `crates/routes_app/src/apps/error.rs` | Route error enum |
| `crates/services/src/app_access_requests/access_request_service.rs` | Business logic |
| `crates/services/src/app_access_requests/access_request_repository.rs` | DB operations |
| `crates/routes_app/src/routes.rs` | Route registration (public_apis router) |
