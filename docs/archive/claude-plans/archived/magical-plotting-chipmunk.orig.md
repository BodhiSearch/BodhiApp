# Phase 3: API Endpoints + Toolset Scope Removal

## Context

Phase 0-1-2 established the access request foundation (database schema, domain objects, service layer). Phase 3 adds HTTP route handlers for the access request flow and removes the deprecated KC scope-based authorization from toolsets, replacing it with the new `tool_type` identifier that aligns with the access-request-based permission model.

**Key design decisions from Q&A session** (see `phase-3-ctx.md`):
- Third-party apps call POST via LNA (unauthenticated), get review_url back
- Auto-approve when no tools requested (empty `requested`)
- `tool_type` replaces `scope`/`scope_uuid` everywhere (e.g., `"builtin-exa-search"`)
- `ToolsetScope` type removed entirely
- `app_toolset_configs` already dropped (migration 0010) — related code is dead
- Approval is single-step PUT; deny is separate POST (no body)
- Poll endpoint requires `?app_client_id=` for security
- Server appends `?id=<uuid>` to redirect_url at draft creation
- KC `register_access_request_consent()` already implemented in AuthService (user-token flow)
- Auto-approve uses separate KC endpoint `/resources/apps/request-access` with resource service token (no user needed)
- **Two KC response variations**: User-approve returns `{scope, access_request_id, access_request_scope}` (all 3 fields); Auto-approve returns only `{scope}` (resource_scope) — no `access_request_scope`
- AccessRequestService already registered in AppService

## Implementation Process

Each phase is designed for sub-agent execution. For each phase:
1. **Implement source code** for the phase
2. **Review existing tests** — analyze test files for the modified code
3. **Update/add tests one-by-one** — write one test, run it, pass it, then write next
4. **Prefer extending existing tests** over creating new test files
5. **Verify compilation** with `cargo check -p <crate>` before running tests
6. **Run crate tests** with `cargo test -p <crate>` after each test addition

---

## Phase 3a: objs Crate — Domain Object Changes

**Sub-agent type:** `general-purpose`

**Goal:** Remove scope/scope_uuid from toolset types, add tool_type, remove dead types, update access_request types.

### Implementation

#### 3a.1 — Update ToolsetDefinition, Toolset, ToolsetWithTools

**File:** `crates/objs/src/toolsets.rs`

- `ToolsetDefinition`: Remove `scope_uuid`, `scope`. Add `tool_type: String`.
- `Toolset`: Remove `scope_uuid`, `scope`. Add `tool_type: String`.
- `ToolsetWithTools`: Remove `scope_uuid`, `scope`. Add `tool_type: String`. Remove `app_enabled` (dead — app_toolset_configs dropped).
- **Remove** `AppToolsetConfig` struct entirely (dead).
- **Remove** `ToolsetTypeInfo` enum entirely (dead — references scope).
- **Remove** `ToolsetScope` struct, `ParseToolsetScopeError`, `toolset_scopes` module, and all related impls/tests entirely.

#### 3a.2 — Update access_request.rs

**File:** `crates/objs/src/access_request.rs`

- Update `ToolApproval`: rename `toolset_id` → `instance_id` (matches Q&A decision)
- Review if any new types needed for the `requested` wrapper (`RequestedResources` struct)

#### 3a.3 — Update lib.rs re-exports

**File:** `crates/objs/src/lib.rs`

- Remove re-exports of deleted types: `AppToolsetConfig`, `ToolsetTypeInfo`, `ToolsetScope`, `ParseToolsetScopeError`

### Testing

- Review existing tests in `crates/objs/src/toolsets.rs` (inline `mod tests`)
- Remove tests for deleted types (ToolsetScope parsing, AppToolsetConfig)
- Update tests that reference `scope_uuid`/`scope` → `tool_type`
- One test at a time, verify: `cargo test -p objs`

### Verification
```
cargo check -p objs
cargo test -p objs
```

---

## Phase 3b: services Crate — DB Schema & Repository Changes

**Sub-agent type:** `general-purpose`

**Goal:** Update migrations, DB row types, and repository SQL for column renames, new columns, and scope→tool_type migration.

### Implementation

#### 3b.1 — Modify existing migration 0011 (unreleased)

**File:** `crates/services/migrations/0011_app_access_requests.up.sql`

Changes:
- Rename `tools_requested` → `requested` (JSON column)
- Rename `tools_approved` → `approved` (JSON column)
- Add `app_name TEXT` column (nullable)
- Add `app_description TEXT` column (nullable)
- `access_request_scope` remains nullable (auto-approve doesn't set it)

Also update `0011_app_access_requests.down.sql` to match.

#### 3b.2 — New migration 0012: toolsets scope → tool_type

**File:** `crates/services/migrations/0012_toolsets_scope_to_tool_type.up.sql`

```sql
ALTER TABLE toolsets ADD COLUMN tool_type TEXT;
UPDATE toolsets SET tool_type = 'builtin-exa-search' WHERE scope_uuid IN (
  '4ff0e163-36fb-47d6-a5ef-26e396f067d6',  -- dev
  '7a89e236-9d23-4856-aa77-b52823ff9972'   -- prod
);
```

Note: Check if SQLite version supports ALTER TABLE DROP COLUMN (3.35+). If not, use CREATE TABLE AS SELECT pattern. Down migration restores scope_uuid.

#### 3b.3 — Update AppAccessRequestRow

**File:** `crates/services/src/db/objs.rs`

- Rename field `tools_requested` → `requested`
- Rename field `tools_approved` → `approved`
- Add fields: `app_name: Option<String>`, `app_description: Option<String>`

#### 3b.4 — Update ToolsetRow

**File:** `crates/services/src/db/objs.rs`

- Replace `scope_uuid: String` → `tool_type: String`

#### 3b.5 — Remove AppToolsetConfigRow

**File:** `crates/services/src/db/objs.rs`

- **Remove** `AppToolsetConfigRow` struct (dead — table dropped).

#### 3b.6 — Update AccessRequestRepository

**File:** `crates/services/src/db/access_request_repository.rs`

- Update column references in SQL queries (tools_requested → requested, tools_approved → approved)
- Add app_name/app_description to insert and update operations
- **Change `update_approval` signature**: `access_request_scope` parameter must change from `&str` to `Option<&str>` — auto-approve flow does NOT return `access_request_scope`, so it must be stored as NULL

#### 3b.7 — Update toolset repository

**File:** `crates/services/src/db/toolset_repository.rs`

- Update SQL queries: `scope_uuid` → `tool_type` in column references
- Remove app_toolset_config repository methods (dead)

#### 3b.8 — Update db/mod.rs exports

**File:** `crates/services/src/db/mod.rs`

- Remove app_toolset_config exports

### Testing

- Review existing tests in `crates/services/src/db/access_request_repository.rs` (inline tests)
- Update tests for renamed columns
- Add test for `update_approval` with `access_request_scope=None` (auto-approve case)
- Review existing tests in `crates/services/src/db/toolset_repository.rs`
- Update tests for scope_uuid → tool_type
- Remove app_toolset_config test cases
- One test at a time: `cargo test -p services`

### Verification
```
cargo check -p services
cargo test -p services
```

---

## Phase 3c: services Crate — ToolService Updates

**Sub-agent type:** `general-purpose`

**Goal:** Update ToolService trait and DefaultToolService impl for tool_type, remove dead app_toolset_config methods.

### Implementation

#### 3c.1 — Update ToolService trait

**File:** `crates/services/src/tool_service/service.rs`

Methods to update (scope_uuid → tool_type):
- `create(user_id, scope_uuid, ...)` → `create(user_id, tool_type, ...)`
- `get_type(scope_uuid)` → `get_type(tool_type)`
- `validate_type(scope_uuid)` → `validate_type(tool_type)`
- `is_type_enabled(scope_uuid)` → `is_type_enabled(tool_type)`

Methods to **remove** (dead — app_toolset_configs dropped):
- `get_app_toolset_config(scope)`
- `is_toolset_enabled_for_app(scope)`
- `set_app_toolset_enabled(admin_token, scope, scope_uuid, enabled, updated_by)`
- `list_app_toolset_configs()`
- `list_app_toolset_configs_by_scopes(scopes)`
- `is_app_client_registered_for_toolset(app_client_id, scope_uuid)`

Update `list_all_toolsets()` return type — `ToolsetWithTools` no longer has `app_enabled`.

#### 3c.2 — Update DefaultToolService impl

- `builtin_toolsets()`: Replace `scope_uuid`/`scope` with `tool_type: "builtin-exa-search"`
- `toolset_row_to_model()`: Use `row.tool_type` instead of looking up scope from scope_uuid
- Remove `app_row_to_config()` (dead)
- Remove all app_toolset_config-related methods
- Remove `is_type_enabled()` if it was checking app_toolset_configs (verify what it does)

### Testing

- Review existing tests in `crates/services/src/tool_service/` (separate tests file or inline)
- Update tests for scope_uuid → tool_type parameter changes
- Remove tests for dead methods (app_toolset_config, etc.)
- One test at a time: `cargo test -p services`

### Verification
```
cargo check -p services
cargo test -p services
```

---

## Phase 3d: services Crate — AccessRequestService & AuthService Updates

**Sub-agent type:** `general-purpose`

**Goal:** Update AccessRequestService for new column names, add auto-approve logic. Add new AuthService methods for auto-approve and app client info.

### Implementation

#### 3d.1 — Update AccessRequestService

**File:** `crates/services/src/access_request_service/service.rs`

- Update `create_draft()` to use `requested` column name
- Update field references (`tools_requested` → `requested`, `tools_approved` → `approved`)
- Add auto-approve logic: if `tools_requested` is empty, call `register_resource_access()` (no user needed)
  - **Auto-approve stores only `resource_scope`** — `access_request_scope` is NULL in DB
  - KC `/resources/apps/request-access` returns only `{scope}`, NOT `access_request_scope`
- Add redirect_url modification: append `?id=<uuid>` to redirect_uri before storing
- Add `app_name`/`app_description` to row construction
- Update `approve_request()` for new column names — stores BOTH `resource_scope` AND `access_request_scope`

#### 3d.2 — Add new AuthService methods

**File:** `crates/services/src/auth_service.rs`

**Method 1 — Auto-approve resource access (no user token):**
- Add trait method: `async fn register_resource_access(app_client_id: &str, access_request_id: &str) -> Result<RegisterResourceAccessResponse>`
- Add new response type: `RegisterResourceAccessResponse { scope: String }` — only `resource_scope`, NO `access_request_scope`
- Implement in `KeycloakAuthService`: call KC `/resources/apps/request-access` with resource client service token
- **Different from user-consent flow**: returns only `scope` (resource_scope), NOT `access_request_scope`

**Method 2 — App client info:**
- Add trait method: `async fn get_app_client_info(app_client_id: &str, user_token: &str) -> Result<AppClientInfo>`
- Add `AppClientInfo { name: String, description: String }` struct
- Implement in `KeycloakAuthService`: call KC `/users/apps/{app_client_id}/info` endpoint
- If KC endpoint not ready yet, return a placeholder or error that the handler can gracefully handle

**Both methods:** Add `MockAuthService` expectations for tests

#### 3d.3 — Update test utils / mock builders

**File:** `crates/services/src/test_utils/`

- Update mock builders for new ToolService trait (removed methods)
- Update mock builders for new AuthService trait methods

### Testing

- Update existing AccessRequestService tests for new column names
- Add test for auto-approve flow: empty `tools_requested` → calls `register_resource_access` → stores `resource_scope` only, `access_request_scope=NULL`
- Add test for redirect_url modification (appending ?id=)
- Add test for `register_resource_access()` (mockito KC response with only `scope`)
- Add test for `get_app_client_info()` (mockito KC response)
- **Test two KC response variations**: auto-approve vs user-approve
- One test at a time: `cargo test -p services`

### Verification
```
cargo check -p services
cargo test -p services
```

---

## Phase 3e: auth_middleware Crate — Toolset Auth Middleware Updates

**Sub-agent type:** `general-purpose`

**Goal:** Update toolset auth middleware for tool_type, remove dead scope-based OAuth checks.

### Implementation

**File:** `crates/auth_middleware/src/toolset_auth_middleware.rs`

- Replace `toolset.scope_uuid` → `toolset.tool_type` for type lookups
- Replace `is_type_enabled(scope_uuid)` → `is_type_enabled(tool_type)`
- **Remove** OAuth scope check block (lines 110-142) — scope-based auth is dead. Phase 4 will implement access_request-based auth. Either remove entirely or mark as TODO.
- Remove `ToolsetAuthError::AppClientNotRegistered` and `MissingToolsetScope` variants if checks are removed.
- Remove import of `ToolsetScope`.

### Testing

- Update existing toolset auth middleware tests for tool_type
- Remove scope-based test cases
- One test at a time: `cargo test -p auth_middleware`

### Verification
```
cargo check -p auth_middleware
cargo test -p auth_middleware
```

---

## Phase 3f: routes_app Crate — Toolset Route Updates

**Sub-agent type:** `general-purpose`

**Goal:** Update toolset routes for tool_type, remove dead scope-based handlers and types.

### Implementation

#### 3f.1 — Update toolset route types

**File:** `crates/routes_app/src/routes_toolsets/types.rs`

- `CreateToolsetRequest`: `scope_uuid` → `tool_type`
- `ToolsetResponse`: Remove `scope_uuid`, `scope`. Add `tool_type`.
- `ToolsetTypeResponse`: Remove `scope_uuid`, `scope`. Add `tool_type`.
- Remove any scope-based types.

#### 3f.2 — Update toolset route handlers

**File:** `crates/routes_app/src/routes_toolsets/toolsets.rs`

- `create_toolset_handler`: Use `request.tool_type` instead of `request.scope_uuid`
- `toolset_to_response()`: Use `toolset.tool_type` instead of looking up scope via scope_uuid
- Remove `enable_type_handler` and `disable_type_handler` (dead — app_toolset_configs dropped)
- Remove `is_oauth_auth()` helper (dead — scope-based filtering removed)
- Remove `extract_allowed_toolset_scopes()` (dead — ToolsetScope removed)
- Update all response construction

#### 3f.3 — Update routes registration

**File:** `crates/routes_app/src/routes.rs`

- Remove routes for enable/disable type endpoints (dead)
- Update any scope-related endpoint constants

#### 3f.4 — Update OpenAPI annotations

**File:** `crates/routes_app/src/shared/openapi.rs`

- Remove scope-related endpoint constants

### Testing

- Review existing toolset route tests
- Update tests for scope_uuid → tool_type
- Remove tests for dead handlers (enable/disable type)
- One test at a time: `cargo test -p routes_app`

### Verification
```
cargo check -p routes_app
cargo test -p routes_app
```

---

## Phase 3g: routes_app Crate — Access Request Route Handlers

**Sub-agent type:** `general-purpose`

**Goal:** Implement the 5 new HTTP endpoints for the access request flow.

### Implementation

#### 3g.1 — Create route module structure

**Files:**
- `crates/routes_app/src/routes_apps/mod.rs` (new)
- `crates/routes_app/src/routes_apps/handlers.rs` (new)
- `crates/routes_app/src/routes_apps/types.rs` (new)
- `crates/routes_app/src/routes_apps/error.rs` (new)

#### 3g.2 — Define request/response types

**File:** `crates/routes_app/src/routes_apps/types.rs`

```rust
// POST /apps/request-access
pub struct CreateAccessRequestBody {
  pub app_client_id: String,
  pub flow_type: String,  // "redirect" | "popup"
  pub redirect_url: Option<String>,
  pub requested: Option<RequestedResources>,  // None = auto-approve
}

pub struct RequestedResources {
  pub tool_types: Vec<ToolTypeRequest>,
}

pub struct CreateAccessRequestResponse {
  pub id: String,
  pub status: String,
  pub review_url: Option<String>,       // Present for "draft"
  pub resource_scope: Option<String>,   // Present for "approved" (both auto and user)
  pub access_request_scope: Option<String>,  // Present ONLY for user-approved; None for auto-approved
}

// GET /apps/access-request/:id
pub struct AccessRequestStatusResponse {
  pub id: String,
  pub status: String,
  pub resource_scope: Option<String>,          // Present for "approved" (both auto and user)
  pub access_request_scope: Option<String>,    // Present ONLY for user-approved; None for auto-approved
}

// GET /apps/access-request/:id/review (session auth)
pub struct AccessRequestReviewResponse {
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub flow_type: String,
  pub status: String,
  pub requested: Option<RequestedResources>,
  pub approved: Option<ApprovedResources>,
  pub tools_info: Vec<ToolTypeReviewInfo>,  // Enriched tool info
  pub expires_at: i64,
  pub created_at: i64,
}

pub struct ToolTypeReviewInfo {
  pub tool_type: String,
  pub display_name: String,
  pub user_instances: Vec<ToolInstanceInfo>,
}

pub struct ToolInstanceInfo {
  pub id: String,
  pub name: String,
  pub enabled: bool,
  pub has_api_key: bool,
}

// PUT /apps/access-request/:id/approve (session auth)
pub struct ApproveAccessRequestBody {
  pub approved: ApprovedResources,
}

pub struct ApprovedResources {
  pub tool_types: Vec<ToolApproval>,
}
```

#### 3g.3 — Define error types

**File:** `crates/routes_app/src/routes_apps/error.rs`

```rust
pub enum AppAccessRequestError {
  NotFound(String),
  Expired(String),
  AlreadyProcessed(String),
  InvalidFlowType(String),
  MissingRedirectUrl,
  AppClientNotFound(String),
  InvalidToolType(String),
  ToolInstanceNotOwned(String),
  ToolInstanceNotConfigured(String),
  // Transparent wrappers
  ServiceError(#[from] AccessRequestError),
  AuthServiceError(#[from] AuthServiceError),
}
```

#### 3g.4 — Implement handlers

**File:** `crates/routes_app/src/routes_apps/handlers.rs`

**POST /bodhi/v1/apps/request-access** (`create_access_request_handler`):
1. Parse `CreateAccessRequestBody` from JSON
2. Validate app_client_id via `auth_service.get_app_client_info()` — 404 if not found, store name/description
3. If `requested` is None or `requested.tool_types` is empty → auto-approve flow:
   - Call `access_request_service.create_draft()` with empty tools
   - Call `auth_service.register_resource_access(app_client_id, access_request_id)` — uses resource service token, no user needed
   - Update DB: `status='approved'`, `resource_scope=<KC response>`, `access_request_scope=NULL`
   - Return {id, status: 'approved', resource_scope, access_request_scope: null}
4. If tools requested → draft flow:
   - Validate tool types exist via `tool_service.get_type(tool_type)`
   - Call `access_request_service.create_draft()`
   - Get review_url from service
   - Return {id, status: 'draft', review_url}
5. In both cases: redirect_url gets `?id=<uuid>` appended by the service layer

**GET /bodhi/v1/apps/access-request/:id** (`get_access_request_status_handler`):
1. Extract `id` from path, `app_client_id` from query params
2. Call `access_request_service.get_request(id)`
3. If None or `row.app_client_id != app_client_id` → 404
4. Return limited fields: {id, status, resource_scope, access_request_scope} — `access_request_scope` null for auto-approved

**GET /bodhi/v1/apps/access-request/:id/review** (`get_access_request_review_handler`):
1. Session auth required (ExtractUserId, ExtractToken)
2. Call `access_request_service.get_request(id)` — return data regardless of status
3. For each tool_type in `requested`:
   - Get ToolsetDefinition name via `tool_service.get_type(tool_type)`
   - Get user's instances via `tool_service.list(user_id)` filtered by tool_type
4. Return enriched response (non-editable if status != "draft")

**PUT /bodhi/v1/apps/access-request/:id/approve** (`approve_access_request_handler`):
1. Session auth required (ExtractUserId, ExtractToken)
2. Parse `ApproveAccessRequestBody`
3. Validate each approved tool instance:
   - Instance belongs to user (`tool_service.get(user_id, instance_id)`)
   - Instance is enabled and has API key
   - Instance's tool_type matches the requested tool_type
4. Call `access_request_service.approve_request(id, user_id, user_token, tool_approvals)`
5. Return updated access request

**POST /bodhi/v1/apps/access-request/:id/deny** (`deny_access_request_handler`):
1. Session auth required (ExtractUserId)
2. Call `access_request_service.deny_request(id, user_id)`
3. Return success

#### 3g.5 — Register routes

**File:** `crates/routes_app/src/routes.rs`

Route registration pattern (from existing codebase):
- **Public (no auth)**: Add to `public_apis` Router:
  - `POST /bodhi/v1/apps/request-access`
  - `GET /bodhi/v1/apps/access-request/:id`
- **Session auth (`user_session_apis`)**: Add to `user_session_apis` Router (uses `api_auth_middleware(ResourceRole::User, None, None)`):
  - `GET /bodhi/v1/apps/access-request/:id/review`
  - `PUT /bodhi/v1/apps/access-request/:id/approve`
  - `POST /bodhi/v1/apps/access-request/:id/deny`

Add endpoint constants in `shared/openapi.rs`.
Export module from routes_app.

#### 3g.6 — Add OpenAPI documentation

- Add `#[utoipa::path(...)]` annotations to all handlers
- Add request/response types to OpenAPI schema
- Update API tags

### Testing

Follow `test-routes-app` skill canonical patterns:
- Test each handler with mock services, one at a time
- Test error paths (not found, expired, validation failures, KC errors)
- Test auth requirements (session vs unauthenticated)
- Test app_client_id validation on poll endpoint
- Test auto-approve flow (response has `resource_scope` only, `access_request_scope` null)
- Test user-approve flow (response has both `resource_scope` and `access_request_scope`)
- One test at a time: `cargo test -p routes_app`

### Verification
```
cargo check -p routes_app
cargo test -p routes_app
```

---

## Critical Files (Complete Change List)

### objs crate
| File | Action |
|------|--------|
| `crates/objs/src/toolsets.rs` | Major: remove scope/scope_uuid, add tool_type, remove ToolsetScope, AppToolsetConfig, ToolsetTypeInfo |
| `crates/objs/src/access_request.rs` | Minor: rename toolset_id → instance_id in ToolApproval |
| `crates/objs/src/lib.rs` | Remove re-exports of deleted types |

### services crate
| File | Action |
|------|--------|
| `crates/services/migrations/0011_app_access_requests.up.sql` | Modify: rename columns, add columns |
| `crates/services/migrations/0011_app_access_requests.down.sql` | Modify: match up.sql |
| `crates/services/migrations/0012_*.up.sql` | New: toolsets scope_uuid → tool_type |
| `crates/services/migrations/0012_*.down.sql` | New: reverse migration |
| `crates/services/src/db/objs.rs` | Update: AppAccessRequestRow (rename fields, add fields), ToolsetRow (scope_uuid→tool_type), remove AppToolsetConfigRow |
| `crates/services/src/db/access_request_repository.rs` | Update: SQL column names, `update_approval` signature |
| `crates/services/src/db/toolset_repository.rs` | Update: SQL scope_uuid→tool_type, remove app_toolset_config methods |
| `crates/services/src/db/mod.rs` | Remove app_toolset_config exports |
| `crates/services/src/tool_service/service.rs` | Major: trait + impl updates |
| `crates/services/src/access_request_service/service.rs` | Update: column names, auto-approve, redirect_url |
| `crates/services/src/auth_service.rs` | Add: register_resource_access, get_app_client_info, RegisterResourceAccessResponse, AppClientInfo |
| `crates/services/src/test_utils/` | Update mock builders |

### auth_middleware crate
| File | Action |
|------|--------|
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Update: scope_uuid→tool_type, remove scope checks |

### routes_app crate
| File | Action |
|------|--------|
| `crates/routes_app/src/routes_apps/` | New module: handlers, types, error, mod |
| `crates/routes_app/src/routes_toolsets/types.rs` | Update: scope_uuid→tool_type |
| `crates/routes_app/src/routes_toolsets/toolsets.rs` | Update: remove dead handlers, update for tool_type |
| `crates/routes_app/src/routes.rs` | Register new routes in public_apis + user_session_apis, remove dead routes |
| `crates/routes_app/src/shared/openapi.rs` | Add endpoint constants |

---

## Open Items / Risks

1. **SQLite DROP COLUMN**: Need to verify SQLite version supports ALTER TABLE DROP COLUMN (3.35+). If not, use CREATE TABLE AS SELECT pattern for migration 0012.

2. **KC endpoints availability**: User is building KC endpoints (`/users/apps/{id}/info` for app info, `/resources/apps/request-access` for auto-approve). AuthService methods should handle "not yet available" gracefully with clear errors.

3. **Scope removal breaks OAuth toolset access**: Removing ToolsetScope and scope checks from auth middleware means OAuth-based tool execution auth doesn't work until Phase 4 implements access_request-based auth. Session-based toolset access continues working.

## Tech Debt Notes

- KC `/apps/{app-client-id}/info` custom SPI endpoint (user building separately)
- KC `/users/apps/{app-client-id}/info` BodhiApp route for app display info
- Multiple tool instances per tool_type (future)
- Per-entity scopes for downscoped access (future)
- New entity types under `requested` (MCP, workspace) (future)
- Phase 4: access_request-based auth in toolset middleware (replaces scope-based auth)

## Final Verification

After all phases complete:
1. `cargo check -p objs && cargo check -p services && cargo check -p auth_middleware && cargo check -p routes_app` — all compile
2. `cargo test -p objs && cargo test -p services && cargo test -p auth_middleware && cargo test -p routes_app` — all pass
3. `make test.backend` — full backend test suite passes
4. Manual verification: `cargo run --package xtask openapi` generates updated spec
