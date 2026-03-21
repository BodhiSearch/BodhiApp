# Toolset Multi-Instance: Middleware & Routes Layer

## Context Summary

Routes shift from toolset-type-based endpoints to instance-UUID-based REST endpoints. Middleware adapts to resolve UUID to instance and validate ownership before authorization checks.

---

## Authorization Middleware

### File: `crates/auth_middleware/src/toolset_auth_middleware.rs`

**Reference:** Current `toolset_auth_middleware` function:
- Extracts `toolset_id` from path
- Checks auth type (session vs OAuth)
- Validates app-level enabled
- For OAuth: checks app-client registration and scope

**New flow with instance UUID:**

```rust
/// Middleware for toolset instance operations
/// Path parameter is now instance UUID, not toolset type ID
pub async fn toolset_auth_middleware(
    State(state): State<Arc<dyn RouterState>>,
    Path(instance_id): Path<String>,  // UUID
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = req.headers();
    let user_id = extract_user_id(headers)?;
    let tool_service = state.app_service().tool_service();

    // 1. Get instance (validates existence)
    let instance = tool_service
        .get_instance(&user_id, &instance_id)
        .await?
        .ok_or(ToolsetError::InstanceNotFound(instance_id.clone()))?;

    // 2. Validate ownership
    // (get_instance already filters by user_id, but explicit check is defensive)

    // 3. Check app-level type enabled
    if !instance.app_enabled {
        return Err(ToolsetError::ToolsetAppDisabled.into());
    }

    // 4. For OAuth token auth: check scope
    let auth_type = determine_auth_type(headers);
    if matches!(auth_type, AuthType::OAuth) {
        let tool_scopes = extract_tool_scopes(headers)?;
        let required_scope = format!("scope_toolset-{}", instance.instance.toolset_type);
        if !tool_scopes.contains(&required_scope) {
            return Err(ToolsetError::ToolsetScopeNotGranted.into());
        }
    }

    // 5. Instance-level checks (enabled, has_api_key) done in execute handler
    // Middleware just validates access rights

    Ok(next.run(req).await)
}
```

**Reference:** Existing helper functions:
- `extract_user_id(headers)` - gets user ID from header
- `determine_auth_type(headers)` - session vs API token vs OAuth
- `extract_tool_scopes(headers)` - parses tool scopes from header

---

## API Routes

### File: `crates/routes_app/src/routes_toolsets.rs`

**Reference:** Current route registration in `routes_toolsets` function.

**New route structure:**

```rust
pub fn routes_toolsets(state: Arc<dyn RouterState>) -> Router {
    Router::new()
        // Toolset CRUD (UUID-based)
        .route("/toolsets", get(list_toolsets_handler))
        .route("/toolsets", post(create_toolset_handler))
        .route("/toolsets/:id", get(get_toolset_handler))
        .route("/toolsets/:id", put(update_toolset_handler))
        .route("/toolsets/:id", delete(delete_toolset_handler))

        // Execute (with auth middleware)
        .route(
            "/toolsets/:id/execute/:method",
            post(execute_toolset_handler)
                .layer(from_fn_with_state(
                    state.clone(),
                    toolset_auth_middleware
                ))
        )

        // Type-level admin routes (keep separate path)
        .route("/toolsets/types", get(list_toolset_types_handler))
        .route(
            "/toolsets/types/:type_id/app-config",
            put(enable_app_toolset_handler)
                .delete(disable_app_toolset_handler)
        )

        .with_state(state)
}
```

**Note:** `/toolsets/types` path avoids collision with `/toolsets/:id` by being more specific.

---

## DTOs

### File: `crates/routes_app/src/toolsets_dto.rs`

**Reference:** Current DTOs:
- `UpdateToolsetConfigRequest` - enabled, api_key
- `ExecuteToolsetRequest` - tool_call_id, params
- `ToolsetWithTools` - toolset definition with tools

**New/Updated DTOs:**

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// === Request DTOs ===

/// Create new toolset instance
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateToolsetRequest {
    /// The toolset type to instantiate
    #[validate(length(min = 1, message = "toolset_type required"))]
    pub toolset_type: String,

    /// User-defined instance name
    #[validate(
        length(min = 1, max = 64),
        regex(path = "TOOLSET_NAME_REGEX", message = "alphanumeric and hyphens only")
    )]
    pub name: String,

    /// Optional description
    #[validate(length(max = 255))]
    #[serde(default)]
    pub description: Option<String>,

    /// Whether enabled (required, typically true)
    pub enabled: bool,

    /// API key (required on create)
    #[validate(length(min = 1, message = "api_key required"))]
    pub api_key: String,
}

/// Update toolset instance (partial update)
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateToolsetRequest {
    /// Update name
    #[validate(
        length(min = 1, max = 64),
        regex(path = "TOOLSET_NAME_REGEX", message = "alphanumeric and hyphens only")
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Update description (explicit null = clear)
    #[validate(length(max = 255))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,

    /// Update enabled status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Update API key (cannot be cleared, only changed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Execute tool request (unchanged from current)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExecuteToolsetRequest {
    pub tool_call_id: String,
    pub params: serde_json::Value,
}

// === Response DTOs ===

/// Single instance response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ToolsetResponse {
    pub id: String,
    pub name: String,
    pub toolset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    pub has_api_key: bool,
    pub app_enabled: bool,
    pub tools: Vec<ToolDefinition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List instances response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetsResponse {
    pub instances: Vec<ToolsetResponse>,
}

/// Toolset type info for admin listing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ToolsetTypeResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub app_enabled: bool,
    pub tools: Vec<ToolDefinition>,
}

/// List toolset types response (admin)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListToolsetTypesResponse {
    pub types: Vec<ToolsetTypeResponse>,
}

// Regex for validation
use once_cell::sync::Lazy;
use regex::Regex;

static TOOLSET_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9-]+$").unwrap()
});
```

---

## Route Handlers

### File: `crates/routes_app/src/routes_toolsets.rs`

**Reference:** Current handler patterns:
- Extract state, headers, path params
- Call service method
- Map result to response

**Handler signatures:**

```rust
/// GET /toolsets - List user's instances
async fn list_toolsets_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
) -> Result<Json<ListToolsetsResponse>, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    // For session auth: return all instances
    // For OAuth: filter by toolset types in token scopes
    let auth_type = determine_auth_type(&headers);
    let instances = match auth_type {
        AuthType::Session => {
            tool_service.list_user_instances(&user_id).await?
        }
        AuthType::OAuth => {
            let allowed_types = extract_allowed_toolset_types(&headers)?;
            let all = tool_service.list_user_instances(&user_id).await?;
            all.into_iter()
                .filter(|i| allowed_types.contains(&i.instance.toolset_type))
                .collect()
        }
        _ => vec![],  // API token: no toolset access
    };

    Ok(Json(ListToolsetsResponse {
        instances: instances.into_iter().map(Into::into).collect()
    }))
}

/// POST /toolsets - Create new instance
async fn create_toolset_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
    ValidatedJson(req): ValidatedJson<CreateToolsetRequest>,
) -> Result<Json<ToolsetResponse>, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    let instance = tool_service.create_instance(
        &user_id,
        &req.toolset_type,
        &req.name,
        req.description,
        req.enabled,
        req.api_key,
    ).await?;

    // Fetch full instance with tools for response
    let full = tool_service.get_instance(&user_id, &instance.id).await?
        .ok_or(ToolsetError::InstanceNotFound(instance.id.clone()))?;

    Ok(Json(full.into()))
}

/// GET /toolsets/:id - Get instance by UUID
async fn get_toolset_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ToolsetResponse>, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    let instance = tool_service.get_instance(&user_id, &id).await?
        .ok_or(ToolsetError::InstanceNotFound(id))?;

    Ok(Json(instance.into()))
}

/// PUT /toolsets/:id - Update instance (partial)
async fn update_toolset_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    ValidatedJson(req): ValidatedJson<UpdateToolsetRequest>,
) -> Result<Json<ToolsetResponse>, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    let updates = InstanceUpdates {
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        api_key: req.api_key,
    };

    let instance = tool_service.update_instance(&user_id, &id, updates).await?;

    // Fetch full instance with tools for response
    let full = tool_service.get_instance(&user_id, &instance.id).await?
        .ok_or(ToolsetError::InstanceNotFound(instance.id.clone()))?;

    Ok(Json(full.into()))
}

/// DELETE /toolsets/:id - Delete instance
async fn delete_toolset_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    tool_service.delete_instance(&user_id, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /toolsets/:id/execute/:method - Execute tool
async fn execute_toolset_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
    Path((id, method)): Path<(String, String)>,
    Json(req): Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
    let user_id = extract_user_id(&headers)?;
    let tool_service = state.app_service().tool_service();

    let response = tool_service
        .execute_instance_tool(&user_id, &id, &method, req)
        .await?;

    Ok(Json(response))
}
```

---

## OAuth Scope Handling

**Reference:** `crates/objs/src/toolsets.rs` - `ToolsetScope` struct

OAuth scopes remain type-level: `scope_toolset-builtin-exa-web-search`

When listing instances for OAuth client:
1. Parse token scopes to get allowed toolset types
2. Filter instances by `instance.toolset_type` matching allowed types
3. Return only instances of allowed types

---

## OpenAPI Updates

### File: `crates/routes_app/src/openapi.rs`

**Reference:** Current schema registration for toolset types.

Add new schemas:
- `CreateToolsetRequest`
- `UpdateToolsetRequest`
- `ToolsetResponse`
- `ListToolsetsResponse`
- `ToolsetTypeResponse`
- `ListToolsetTypesResponse`

Update endpoint documentation with new paths.

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Adapt for UUID, resolve instance, type-level scope check |
| `crates/routes_app/src/routes_toolsets.rs` | New REST endpoints, handlers |
| `crates/routes_app/src/toolsets_dto.rs` | New request/response DTOs |
| `crates/routes_app/src/openapi.rs` | Register new schemas |

---

## Test Considerations

### Route Tests (`crates/routes_app/src/routes_toolsets_test.rs`)

- Create instance: valid request, missing fields, invalid name
- Update instance: partial updates, name collision
- Delete instance: success, not found, not owned
- List instances: session vs OAuth filtering
- Execute: authorized, not enabled, no API key

### Middleware Tests

- Valid instance UUID resolves
- Invalid UUID returns 404
- User can only access own instances (403)
- OAuth scope checked for type
- App-disabled type returns appropriate error

**Reference:** Existing test patterns in `routes_toolsets_test.rs` with mock services.
