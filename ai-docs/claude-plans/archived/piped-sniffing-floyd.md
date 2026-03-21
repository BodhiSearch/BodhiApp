# Toolset Multi-Instance: Middleware & Routes Implementation

## Summary

Update middleware and routes from type-based (`/toolsets/{toolset_id}`) to UUID-based instance architecture (`/toolsets/{id}`). Middleware handles execute auth; handlers manage CRUD ownership checks.

## User Decisions Applied

| Decision | Applied |
|----------|---------|
| Method names | Use actual ToolService: `get`, `list`, `create`, `update`, `delete`, `execute` |
| Update pattern | Regular PUT (required fields); `ApiKeyUpdate` enum for api_key only |
| Auth scope | Middleware for execute only; ownership checks in handlers for CRUD |
| OAuth filtering | Hide instances of non-scoped types completely |
| Admin routes | Rely on existing auth middleware (no explicit role check) |
| Tests | Parameterized rstest with fixtures |
| Method validation | Service layer (not middleware) |
| Error assertions | Verify status + error.code |
| Missing user ID | BadRequest (400) |
| Response enrichment | Service returns enriched type with tools |
| Validation errors | Generic 'validation_error' code with details |

---

## Phase middleware-update: Toolset Execute Middleware

### File: `crates/auth_middleware/src/toolset_auth_middleware.rs`

**Changes:**

1. **Update path extraction** - `Path((id, method)): Path<(String, String)>` (UUID + method)

2. **New authorization flow:**
   - Extract user_id from headers
   - Determine auth type (session vs OAuth)
   - Call `tool_service.get(user_id, &id)` - returns None if not found OR not owned
   - Extract `toolset_type` from instance for scope validation
   - Check `is_type_enabled(toolset_type)` - app-level
   - OAuth: check `is_app_client_registered_for_toolset(azp, toolset_type)`
   - OAuth: validate `scope_toolset-{type}` in token
   - Check `is_available(user_id, &id)` - instance availability

3. **Add error variant:**
```rust
#[error("instance_not_found")]
#[error_meta(error_type = ErrorType::NotFound)]
InstanceNotFound,
```

4. **Update error messages in `en-US/messages.ftl`:**
```ftl
instance_not_found = Toolset instance not found
```

**Key code structure:**
```rust
pub async fn toolset_auth_middleware(
  State(state): State<Arc<dyn RouterState>>,
  Path((id, method)): Path<(String, String)>,  // UUID + method
  req: Request,
  next: Next,
) -> Result<Response, ApiError>
```

---

## Phase middleware-tests: Middleware Test Updates

### File: `crates/auth_middleware/src/toolset_auth_middleware.rs` (test module)

**Replace tests with parameterized rstest:**

```rust
// Fixture for test toolset instance
#[fixture]
fn test_instance() -> Toolset { ... }

// Session auth tests
#[rstest]
#[case::success(true, true, StatusCode::OK)]
#[case::instance_not_found(false, false, StatusCode::NOT_FOUND)]
#[case::type_disabled(true, false, StatusCode::BAD_REQUEST)]
#[case::instance_not_available(true, false, StatusCode::BAD_REQUEST)]
#[tokio::test]
async fn test_session_auth(...) { ... }

// OAuth auth tests
#[rstest]
#[case::success(StatusCode::OK)]
#[case::app_client_not_registered(StatusCode::FORBIDDEN)]
#[case::missing_scope(StatusCode::FORBIDDEN)]
#[tokio::test]
async fn test_oauth_auth(...) { ... }

// Error condition tests
#[rstest]
#[case::missing_user_id(StatusCode::BAD_REQUEST)]
#[case::missing_auth(StatusCode::UNAUTHORIZED)]
#[tokio::test]
async fn test_error_conditions(...) { ... }
```

**Test count: ~12 parameterized cases**

---

## Phase routes-dto: DTO Updates

### File: `crates/routes_app/src/toolsets_dto.rs`

**Remove:**
- `UpdateToolsetConfigRequest`
- `GetToolsetConfigResponse`
- `EnhancedToolsetConfigResponse`

**Add:**

```rust
// Create instance request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateToolsetRequest {
  #[validate(length(min = 1))]
  pub toolset_type: String,
  #[validate(length(min = 1, max = 24), regex(path = "TOOLSET_NAME_REGEX"))]
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[validate(length(max = 255))]
  pub description: Option<String>,
  #[serde(default = "default_true")]
  pub enabled: bool,
  pub api_key: String,
}

// Update instance request (full PUT - all fields required except api_key)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateToolsetRequest {
  #[validate(length(min = 1, max = 24), regex(path = "TOOLSET_NAME_REGEX"))]
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[validate(length(max = 255))]
  pub description: Option<String>,
  pub enabled: bool,
  #[serde(default)]
  pub api_key: ApiKeyUpdateDto,  // Only api_key uses Keep/Set pattern
}

// API key update enum (mirrors services::db::ApiKeyUpdate)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "action", content = "value")]
pub enum ApiKeyUpdateDto {
  #[default]
  Keep,
  Set(Option<String>),
}

// Instance response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetResponse {
  pub id: String,
  pub name: String,
  pub toolset_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub enabled: bool,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

// List response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetsResponse {
  pub toolsets: Vec<ToolsetResponse>,
}

// Type response (for admin listing)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolsetTypeResponse {
  pub toolset_id: String,
  pub name: String,
  pub description: String,
  pub app_enabled: bool,
  pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetTypesResponse {
  pub types: Vec<ToolsetTypeResponse>,
}
```

---

## Phase routes-handlers: Route Handler Updates

### File: `crates/routes_app/src/routes_toolsets.rs`

**New route structure:**
```rust
pub fn routes_toolsets(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    // Instance CRUD
    .route("/toolsets", get(list_toolsets_handler))
    .route("/toolsets", post(create_toolset_handler))
    .route("/toolsets/:id", get(get_toolset_handler))
    .route("/toolsets/:id", put(update_toolset_handler))
    .route("/toolsets/:id", delete(delete_toolset_handler))
    // Execute (middleware at routes_all level)
    .route("/toolsets/:id/execute/:method", post(execute_toolset_handler))
    // Type listing and admin (separate namespace avoids :id collision)
    .route("/toolset_types", get(list_toolset_types_handler))
    .route("/toolset_types/:type_id/app-config", put(enable_type_handler))
    .route("/toolset_types/:type_id/app-config", delete(disable_type_handler))
    .with_state(state)
}
```

**Handlers to implement:**

| Handler | Service Call | Notes |
|---------|--------------|-------|
| `list_toolsets_handler` | `tool_service.list(&user_id)` | OAuth: filter by scopes |
| `create_toolset_handler` | `tool_service.create(...)` | Returns 201 Created |
| `get_toolset_handler` | `tool_service.get(&user_id, &id)` | None → 404 (hide existence) |
| `update_toolset_handler` | `tool_service.update(...)` | Full PUT semantics |
| `delete_toolset_handler` | `tool_service.delete(&user_id, &id)` | Returns 204 No Content |
| `execute_toolset_handler` | `tool_service.execute(...)` | Middleware pre-validated |
| `list_toolset_types_handler` | `tool_service.list_all_toolsets()` | OAuth: filter by scopes |
| `enable_type_handler` | `tool_service.set_app_toolset_enabled(...)` | Admin via middleware |
| `disable_type_handler` | `tool_service.set_app_toolset_enabled(...)` | Admin via middleware |

**Helper functions:**
```rust
fn is_oauth_auth(headers: &HeaderMap) -> bool { ... }
fn extract_allowed_toolset_types(headers: &HeaderMap) -> HashSet<String> { ... }
```

---

## Phase routes-tests: Route Handler Tests

### File: `crates/routes_app/src/routes_toolsets.rs` (test module)

**Test fixtures:**
```rust
#[fixture]
fn mock_tool_service() -> MockToolService { ... }

#[fixture]
fn test_instance() -> Toolset { ... }

fn test_router(mock: MockToolService) -> Router { ... }
```

**Parameterized tests by endpoint:**

| Endpoint | Test Cases |
|----------|------------|
| `list` | session_returns_all, oauth_filters_by_scope, empty_list |
| `create` | success, missing_name, invalid_chars, name_too_long, invalid_type, type_disabled, name_exists |
| `get` | success, not_found, not_owned_returns_404 |
| `update` | success, api_key_set, api_key_keep, not_found, not_owned_returns_404, name_conflict, validation_error |
| `delete` | success, not_found, not_owned_returns_404 |
| `execute` | success, method_not_found |
| `list_types` | session_returns_all, oauth_filters |
| `enable/disable` | success, type_not_found |

**Test count: ~30 parameterized cases**

---

## Phase integration: Wire Up Execute Middleware

### File: `crates/routes_all/src/lib.rs` (or equivalent)

Apply middleware to execute endpoint:
```rust
.route(
  "/toolsets/:id/execute/:method",
  post(execute_toolset_handler)
    .route_layer(from_fn_with_state(state.clone(), toolset_auth_middleware))
)
```

### File: `crates/routes_app/src/openapi.rs`

Register new schemas:
- `CreateToolsetRequest`
- `UpdateToolsetRequest`
- `ApiKeyUpdateDto`
- `ToolsetResponse`
- `ListToolsetsResponse`
- `ToolsetTypeResponse`
- `ListToolsetTypesResponse`

---

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Update path, auth flow, error, tests |
| `crates/routes_app/src/toolsets_dto.rs` | Remove old, add new DTOs |
| `crates/routes_app/src/routes_toolsets.rs` | Replace routes, handlers, tests |
| `crates/routes_app/src/openapi.rs` | Register new schemas |
| `crates/routes_all/src/lib.rs` | Wire execute middleware |
| `crates/objs/src/resources/en-US/messages.ftl` | Add error message |

---

## Verification

**After each phase:**
```bash
cargo fmt --all
cargo test -p auth_middleware  # middleware phases
cargo test -p routes_app       # routes phases
cargo clippy -p auth_middleware -p routes_app
```

**Final verification:**
```bash
make test.backend
cargo run --package xtask openapi
cd ts-client && npm run generate
```

---

## Out of Scope

- Frontend changes (bodhi crate)
- `app_toolset_configs` table (unchanged)
- `app_client_toolset_configs` table (unchanged)
- Integration tests (separate phase)
