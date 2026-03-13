# Entity Enforcement Flow

> **Purpose**: How the access_request_auth_middleware validates that an external app's token grants access to a specific toolset or MCP instance.
> Covers: middleware, validator trait, toolset/MCP validators, approved JSON parsing, route integration.

---

## Overview

After token validation builds `AuthContext::ExternalApp` (see token-validation-flow.md), entity-level enforcement checks whether the specific toolset or MCP being accessed is in the access request's approved list.

```
AuthContext::ExternalApp { access_request_id: Some("ar-uuid") }
  |
  v
access_request_auth_middleware
  |-- Extract entity ID from URL path
  |-- Load access request from DB
  |-- Validate: status, app_client_id, user_id
  |-- Parse approved JSON
  |-- Check entity ID exists in approved list
  |
  v
Route handler (entity access confirmed)
```

---

## Middleware

**File**: `crates/routes_app/src/middleware/access_requests/access_request_middleware.rs`

### Signature

```
access_request_auth_middleware(
  validator: Arc<dyn AccessRequestValidator>,
  State(app_service): State<Arc<dyn AppService>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, MiddlewareError>
```

### Auth Flow Classification

| AuthContext Variant | Behavior |
|-------------------|----------|
| `Session` / `MultiTenantSession` | **Pass through** — session users bypass entity validation entirely |
| `ExternalApp { access_request_id: Some(id) }` | **Validate** — full entity enforcement |
| `ExternalApp { access_request_id: None }` | **Reject** — `MissingAuth` |
| `ApiToken` / `Anonymous` | **Reject** — `MissingAuth` |

### Validation Steps (OAuth Flow Only)

1. **Extract entity ID**: `validator.extract_entity_id(request_path)` — parses the URL to get the entity UUID
2. **Validate access request record**: `validate_access_request(db_service, tenant_id, access_request_id, app_client_id, user_id)`
3. **Check entity in approved list**: `validator.validate_approved(&approved_json, &entity_id)`

### validate_access_request() Helper

Performs 5 checks against the DB record:

| # | Check | Error |
|---|-------|-------|
| 1 | Record exists | `AccessRequestNotFound` |
| 2 | `status == Approved` | `AccessRequestNotApproved { id, status }` |
| 3 | `app_client_id` matches | `AppClientMismatch { expected, found }` |
| 4 | `user_id` exists | `AccessRequestInvalid { id, reason }` |
| 5 | `user_id` matches | `UserMismatch { expected, found }` |

Returns the `approved` JSON field (Option<String>) on success.

---

## Validator Trait

**File**: `crates/routes_app/src/middleware/access_requests/access_request_validator.rs`

```
trait AccessRequestValidator: Send + Sync + 'static {
  fn extract_entity_id(path: &str) -> Result<String, AccessRequestAuthError>;
  fn validate_approved(approved_json: &Option<String>, entity_id: &str) -> Result<(), AccessRequestAuthError>;
}
```

### Path Extraction Helper

`extract_id_from_path(path, resource_prefix)`:
- Splits path by `/`
- Finds segment matching `resource_prefix` (e.g., "toolsets")
- Returns the next segment as entity ID

Examples:
- `/bodhi/v1/toolsets/01ABC.../tools/search/execute` + "toolsets" -> `"01ABC..."`
- `/bodhi/v1/mcps/01DEF.../tools/read/execute` + "mcps" -> `"01DEF..."`

---

## ToolsetAccessRequestValidator

Extracts entity ID from `"toolsets"` segment.

Approved JSON validation:
1. `approved_json` must be Some (None -> `EntityNotApproved`)
2. Deserialize to `ApprovedResources`
3. Search `approvals.toolsets` for entry where:
   - `approval.status == ApprovalStatus::Approved`
   - `approval.instance.id == entity_id`
4. No match -> `EntityNotApproved`

---

## McpAccessRequestValidator

Identical logic to toolset validator but:
- Extracts entity ID from `"mcps"` segment
- Searches `approvals.mcps` vector

---

## Route Integration

**File**: `crates/routes_app/src/routes.rs`

### Toolset Execution

```
POST /bodhi/v1/toolsets/{id}/tools/{tool_name}/execute
  Middleware (bottom-up execution):
    1. api_auth_middleware(ResourceRole::User, None, Some(UserScope::User))
    2. access_request_auth_middleware(ToolsetAccessRequestValidator)
  Handler: toolsets_execute()
```

### MCP Endpoints

```
GET  /bodhi/v1/mcps/{id}
POST /bodhi/v1/mcps/{id}/tools/refresh
POST /bodhi/v1/mcps/{id}/tools/{tool_name}/execute
  Middleware (bottom-up execution):
    1. api_auth_middleware(ResourceRole::User, None, Some(UserScope::User))
    2. access_request_auth_middleware(McpAccessRequestValidator)
  Handlers: mcps_show(), mcps_refresh_tools(), mcps_execute_tool()
```

**Execution order**: api_auth_middleware runs FIRST (role check), then access_request_auth_middleware (entity check). This is because axum executes route layers bottom-to-top.

### Unprotected List Endpoints

```
GET /bodhi/v1/toolsets
GET /bodhi/v1/mcps
```

These do NOT have access_request_auth_middleware — external apps can list all resources (filtered by auth-scoped service).

---

## Error Types

**File**: `crates/routes_app/src/middleware/access_requests/error.rs`

| Error | HTTP | Type | Description |
|-------|------|------|-------------|
| `MissingAuth` | 401 | Authentication | No valid auth context or not ExternalApp |
| `EntityNotFound` | 404 | NotFound | Entity ID not parseable from URL path |
| `AccessRequestNotFound { id }` | 403 | Forbidden | Access request UUID not in DB |
| `AccessRequestNotApproved { id, status }` | 403 | Forbidden | Status != Approved |
| `AccessRequestInvalid { id, reason }` | 403 | Forbidden | Missing user_id on record |
| `EntityNotApproved { entity_id }` | 403 | Forbidden | Entity not in approved JSON |
| `AppClientMismatch { expected, found }` | 403 | Forbidden | Token's app != access request's app |
| `UserMismatch { expected, found }` | 403 | Forbidden | Token's user != access request's user |
| `InvalidApprovedJson { error }` | 500 | InternalServer | JSON parse failure |
| `DbError` | varies | varies | DB operation failure |

Error codes follow pattern: `access_request_auth_error-<variant_snake_case>` (auto-generated by `errmeta_derive`).

---

## Approved JSON Shape (Reference)

See data-model.md for full type details.

```json
{
  "toolsets": [
    {
      "toolset_type": "builtin-exa-search",
      "status": "approved",
      "instance": { "id": "01ABC..." }
    },
    {
      "toolset_type": "another-tool",
      "status": "denied"
    }
  ],
  "mcps": [
    {
      "url": "https://mcp.example.com/mcp",
      "status": "approved",
      "instance": { "id": "01DEF..." }
    }
  ]
}
```

The validator checks:
- `status == "approved"` (denied entries are skipped)
- `instance.id == entity_id` (exact UUID match)

Both conditions must be true for the entity to be accessible.

---

## Handler Behavior

Route handlers do NOT perform entity access checks — middleware handles this. Handlers receive `AuthScope` and trust that middleware has validated access.

Example (`toolsets_execute`):
```
auth_scope.tools().execute(&id, &tool_name, request)
```

The auth-scoped service further filters by user ownership, but the middleware has already confirmed the specific entity was approved in the access request.

---

## Key Files

| File | Role |
|------|------|
| `crates/routes_app/src/middleware/access_requests/access_request_middleware.rs` | Main middleware |
| `crates/routes_app/src/middleware/access_requests/access_request_validator.rs` | Validator trait + implementations |
| `crates/routes_app/src/middleware/access_requests/error.rs` | Error enum |
| `crates/routes_app/src/middleware/access_requests/test_access_request_middleware.rs` | Tests |
| `crates/routes_app/src/routes.rs` | Route registration with middleware layers |
| `crates/services/src/app_access_requests/access_request_objs.rs` | ApprovedResources, ApprovalStatus types |
