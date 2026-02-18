---
name: Generic Access Request Middleware
overview: Refactor toolset_auth_middleware to a generic access_request_auth_middleware using a trait-based design, extend the access request flow to support MCP servers alongside toolsets, update MCP routes for OAuth access, and add frontend review UI and e2e tests.
todos:
  - id: phase-1-domain-types
    content: "Phase 1: Add McpServerRequest, McpServerApproval, McpInstanceApproval types to objs/src/access_request.rs"
    status: pending
  - id: phase-2a-trait
    content: "Phase 2a: Define AccessRequestValidator trait in auth_middleware crate"
    status: pending
  - id: phase-2b-middleware
    content: "Phase 2b: Implement generic access_request_auth_middleware function with common OAuth validation logic"
    status: pending
  - id: phase-2c-toolset-validator
    content: "Phase 2c: Implement ToolsetAccessRequestValidator (extract toolset UUID, validate against toolset_types in approved JSON)"
    status: pending
  - id: phase-2d-mcp-validator
    content: "Phase 2d: Implement McpAccessRequestValidator (extract MCP UUID, validate against mcp_servers[].instances[] in approved JSON)"
    status: pending
  - id: phase-2e-error-types
    content: "Phase 2e: Rename ToolsetAuthError to AccessRequestAuthError with generic variants"
    status: pending
  - id: phase-2f-tests
    content: "Phase 2f: Migrate existing toolset_auth_middleware tests to trait-based approach, add MCP validator tests"
    status: pending
  - id: phase-3-services
    content: "Phase 3: Update access_request_service create_draft and approve_request to handle mcp_servers in requested/approved JSON"
    status: pending
  - id: phase-4a-dtos
    content: "Phase 4a: Update RequestedResources and ApprovedResources DTOs in routes_apps/types.rs"
    status: pending
  - id: phase-4b-handlers
    content: "Phase 4b: Update access request create/review/approve handlers for MCP support"
    status: pending
  - id: phase-4c-toolset-handler
    content: "Phase 4c: Move toolset domain checks (type_enabled, has_api_key, enabled) from middleware to execute_toolset_handler"
    status: pending
  - id: phase-4d-mcp-routes
    content: "Phase 4d: Create mcp_exec_apis router group with access_request_middleware for GET /mcps/{id}, POST refresh, POST execute"
    status: pending
  - id: phase-4e-remove-tools-endpoint
    content: "Phase 4e: Remove GET /mcps/{id}/tools endpoint (tools already in GET responses)"
    status: pending
  - id: phase-4f-oauth-filter
    content: "Phase 4f: Update GET /mcps handler to filter by approved instance IDs for OAuth access"
    status: pending
  - id: phase-5-frontend
    content: "Phase 5: Update access-request review page for MCP server approvals and MCP pages for OAuth compatibility"
    status: pending
  - id: phase-6-e2e
    content: "Phase 6: E2E test - OAuth full flow: request MCP access -> approve instances -> execute MCP tool"
    status: pending
isProject: false
---

# Generic Access Request Auth Middleware for MCPs

## Context

Currently, `toolset_auth_middleware` in [crates/auth_middleware/src/toolset_auth_middleware.rs](crates/auth_middleware/src/toolset_auth_middleware.rs) handles both access-request validation AND domain-specific checks (type enabled, instance configured) for toolset execution. MCP routes are session-only with no OAuth support.

The goal is to:

1. Refactor to a generic, trait-based access_request_auth_middleware that handles ONLY access-request validation
2. Move domain-specific checks to route handlers
3. Extend the access request system to support MCP servers (requested by URL)
4. Enable OAuth access to MCP routes via the new middleware

## Key Design Decisions

- **Approved JSON structure** evolves from `{"toolset_types": [...]}` to:

```json
{
  "toolset_types": [
    {"toolset_type": "builtin-exa-search", "status": "approved", "instance_id": "uuid"}
  ],
  "mcp_servers": [
    {
      "url": "https://mcp.deepwiki.com/mcp",
      "status": "approved",
      "instances": [{"id": "uuid", "status": "approved"}]
    }
  ]
}
```

- **Requested JSON** extends to: `{"toolset_types": [...], "mcp_servers": [{"url": "https://..."}]}`
- **Middleware scope**: Only access-request checks (OAuth flow). Session users pass through. Domain validation moves to handlers.
- **Trait-based design**: `AccessRequestValidator` trait with methods for entity ID extraction and approved-list validation
- **Backward compat**: Missing `mcp_servers` key treated as empty array (no migration needed)
- **Remove** `GET /mcps/{id}/tools` endpoint (tools already included in `GET /mcps` and `GET /mcps/{id}` responses via `tools_cache`/`tools_filter` fields)

## Architecture

### Middleware Flow (OAuth only - Session passes through)

```mermaid
flowchart TD
    A[Request] --> B{AuthContext?}
    B -->|Session| C[Pass through to handler]
    B -->|ExternalApp with access_request_id| D[Fetch access_request from DB]
    B -->|Other/Anonymous| E[Reject: MissingAuth]
    D --> F{status == approved?}
    F -->|No| G[Reject: NotApproved]
    F -->|Yes| H{app_client_id matches?}
    H -->|No| I[Reject: AppClientMismatch]
    H -->|Yes| J{user_id matches?}
    J -->|No| K[Reject: UserMismatch]
    J -->|Yes| L["validator.extract_entity_id(path)"]
    L --> M["validator.validate_approved(approved_json, entity_id)"]
    M -->|Pass| C
    M -->|Fail| N[Reject: NotApproved]
```



### Route Composition Changes

```mermaid
flowchart LR
    subgraph current [Current Layout]
        A1[user_session_apis] -->|"session-only"| B1["MCP CRUD + tools + execute"]
        A2[user_oauth_apis] -->|"session+OAuth"| B2["GET /mcps list"]
        A3[toolset_exec_apis] -->|"toolset_auth_middleware"| B3["POST toolsets/.../execute"]
    end

    subgraph newLayout [New Layout]
        C1[user_session_apis] -->|"session-only"| D1["MCP create/update/delete"]
        C2[user_oauth_apis] -->|"session+OAuth"| D2["GET /mcps list with filtering"]
        C3[toolset_exec_apis] -->|"access_request_middleware+ToolsetValidator"| D3["POST toolsets/.../execute"]
        C4[mcp_exec_apis] -->|"access_request_middleware+McpValidator"| D4["GET /mcps/{id}, POST refresh, POST execute"]
    end
```



---

## Phase 1: Domain Types (`objs` crate)

**File**: [crates/objs/src/access_request.rs](crates/objs/src/access_request.rs)

Add MCP-related access request types alongside existing `ToolsetTypeRequest` and `ToolsetApproval`:

```rust
// Request types (what the external app asks for)
pub struct McpServerRequest {
  pub url: String,
}

// Approval types (what the resource owner grants)
pub struct McpServerApproval {
  pub url: String,
  pub status: String, // "approved" | "denied"
  pub instances: Option<Vec<McpInstanceApproval>>,
}

pub struct McpInstanceApproval {
  pub id: String,
  pub status: String, // "approved" | "denied"
}
```

---

## Phase 2: Middleware Refactoring (`auth_middleware` crate)

**Rename**: `toolset_auth_middleware.rs` -> `access_request_auth_middleware.rs`

### 2a. Define `AccessRequestValidator` trait

```rust
pub trait AccessRequestValidator: Send + Sync + 'static {
  fn extract_entity_id(&self, path: &str) -> Result<String, ApiError>;
  fn validate_approved(&self, approved_json: &Option<String>, entity_id: &str) -> Result<(), ApiError>;
}
```

### 2b. Generic middleware function

```rust
pub async fn access_request_auth_middleware(
  validator: Arc<dyn AccessRequestValidator>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, ApiError>
```

- Session flow: pass through (no access_request checks)
- OAuth flow: common validation (status, app_client_id, user_id) then `validator.validate_approved()`

### 2c. Implement `ToolsetAccessRequestValidator`

Extracts toolset UUID from path, validates against `toolset_types` in approved JSON. Domain checks (type_enabled, has_api_key) removed from middleware.

### 2d. Implement `McpAccessRequestValidator`

Extracts MCP instance UUID from path, validates against `mcp_servers[].instances[].id` in approved JSON.

### 2e. Rename error type

`ToolsetAuthError` -> `AccessRequestAuthError` with generic variants (remove toolset-specific ones like `ToolsetNotApproved`, `ToolsetNotFound`). Keep generic: `MissingAuth`, `AccessRequestNotFound`, `AccessRequestNotApproved`, `AppClientMismatch`, `UserMismatch`, `EntityNotApproved`, `InvalidApprovedJson`.

### 2f. Update tests

Migrate existing tests to use the new trait-based approach. Keep parametric test structure.

---

## Phase 3: Access Request Service Updates (`services` crate)

**File**: `crates/services/src/access_request_service/service.rs`

### 3a. Update `create_draft`

Serialize `mcp_servers` alongside `toolset_types` into the `requested` JSON. Validate each requested MCP server URL exists in `mcp_servers` table and is enabled.

### 3b. Update `approve_request`

Accept `McpServerApproval` list alongside `ToolsetApproval` list. Serialize both into `approved` JSON. Validate each approved MCP instance belongs to the user and its server URL matches the requested URL.

---

## Phase 4: Route Updates (`routes_app` crate)

### 4a. Access Request DTOs

**File**: [crates/routes_app/src/routes_apps/types.rs](crates/routes_app/src/routes_apps/types.rs)

Update `RequestedResources` and `ApprovedResources`:

```rust
pub struct RequestedResources {
  #[serde(default)]
  pub toolset_types: Vec<ToolsetTypeRequest>,
  #[serde(default)]
  pub mcp_servers: Vec<McpServerRequest>,
}

pub struct ApprovedResources {
  #[serde(default)]
  pub toolset_types: Vec<ToolsetApproval>,
  #[serde(default)]
  pub mcp_servers: Vec<McpServerApproval>,
}
```

### 4b. Access request handlers

- **create_access_request_handler**: Accept `mcp_servers` in request body
- **get_access_request_review_handler**: Return user's MCP instances matching requested URLs
- **approve_access_request_handler**: Validate MCP instance approvals (instance exists, user owns it, server URL matches, instance enabled)

### 4c. Toolset handler migration

**File**: [crates/routes_app/src/routes_toolsets/](crates/routes_app/src/routes_toolsets/)

Move domain checks from middleware to `execute_toolset_handler`:

- `tool_service.is_type_enabled(&toolset.toolset_type)`
- `toolset.enabled` and `toolset.has_api_key` checks

### 4d. MCP route restructuring

**File**: [crates/routes_app/src/routes.rs](crates/routes_app/src/routes.rs)

Split MCP routes:

- **user_session_apis** (session-only): `POST /mcps` (create), `PUT /mcps/{id}` (update), `DELETE /mcps/{id}` (delete)
- **user_oauth_apis** (session+OAuth): `GET /mcps` (list, with handler-level filtering by approved instance IDs)
- **mcp_exec_apis** (new, session+OAuth+middleware): `GET /mcps/{id}`, `POST /mcps/{id}/tools/refresh`, `POST /mcps/{id}/tools/{tool_name}/execute`

The `mcp_exec_apis` router applies `access_request_auth_middleware` with `McpAccessRequestValidator`.

### 4e. Remove `GET /mcps/{id}/tools` endpoint

Tools are already returned in `GET /mcps` and `GET /mcps/{id}` responses via `tools_cache` and `tools_filter` fields in `McpResponse`.

### 4f. Update `GET /mcps` handler for OAuth filtering

When `AuthContext::ExternalApp`, extract approved instance IDs from the access request's `approved` JSON and filter the list to only those instances.

---

## Phase 5: Frontend Updates (`crates/bodhi/src`)

### 5a. Access request review page

Update the review/approve UI to display MCP servers (by URL) alongside toolset types. Show the user's MCP instances connected to each requested URL. Allow instance-level approval/denial.

### 5b. MCP pages OAuth compatibility

Ensure MCP list and detail pages work correctly when accessed via OAuth-filtered responses.

---

## Phase 6: E2E Tests

### 6a. OAuth MCP access flow

Test the full flow:

1. External app creates access request with `mcp_servers` URL
2. Resource owner reviews and sees their MCP instances for the URL
3. Resource owner approves specific instances
4. External app uses OAuth token to list MCPs (filtered), get MCP details, refresh tools, and execute a tool
5. Verify unauthorized instance access is denied

---

## Key Files to Modify


| Crate             | File                                    | Change                                  |
| ----------------- | --------------------------------------- | --------------------------------------- |
| `objs`            | `src/access_request.rs`                 | Add MCP request/approval types          |
| `auth_middleware` | `src/toolset_auth_middleware.rs`        | Rename, refactor to trait-based generic |
| `auth_middleware` | `src/lib.rs`                            | Update exports                          |
| `services`        | `src/access_request_service/service.rs` | Handle MCP in create/approve            |
| `routes_app`      | `src/routes_apps/types.rs`              | Extend DTOs                             |
| `routes_app`      | `src/routes_apps/handlers.rs`           | Update review/approve handlers          |
| `routes_app`      | `src/routes.rs`                         | Add mcp_exec_apis group                 |
| `routes_app`      | `src/routes_mcps/mcps.rs`               | Update handlers, remove tools endpoint  |
| `routes_app`      | `src/routes_toolsets/`                  | Move domain checks to handler           |
| `bodhi/src`       | Access request review page              | MCP approval UI                         |


