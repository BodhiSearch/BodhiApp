# Routes & API - Tools Feature

> Layer: `routes_app` crate | Status: ✅ Complete (6 tests passing)

## Implementation Note

Routes are currently implemented without final middleware integration. User extraction uses temporary header-based approach that will be replaced when routes are integrated into `routes_all`.

## Endpoints Summary

**Files**: `crates/routes_app/src/{routes_tools.rs, tools_dto.rs}`

| Method | Path | Handler | Status |
|--------|------|---------|--------|
| GET | `/tools` | `list_all_tools()` | ✅ Complete |
| GET | `/tools/configured` | `list_configured_tools()` | ✅ Complete |
| GET | `/tools/:tool_id/config` | `get_tool_config()` | ✅ Complete |
| PUT | `/tools/:tool_id/config` | `update_tool_config()` | ✅ Complete |
| POST | `/tools/:tool_id/execute` | `execute_tool()` | ✅ Complete |

Note: Paths shown above are route-level paths. When integrated into `routes_all`, they will be prefixed with `/bodhi/v1`.

## Endpoint Details

### GET /bodhi/v1/tools

List tools configured by current user (only enabled with API key).

```rust
#[utoipa::path(
    get,
    path = "/bodhi/v1/tools",
    tag = "tools",
    operation_id = "listUserTools",
    responses(
        (status = 200, body = ToolListResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn list_user_tools_handler(
    State(state): State<Arc<dyn RouterState>>,
    headers: HeaderMap,
) -> Result<Json<ToolListResponse>, ApiError>
```

### GET /bodhi/v1/tools/available

List all available tool definitions (for UI tool picker).

```rust
#[utoipa::path(
    get,
    path = "/bodhi/v1/tools/available",
    tag = "tools",
    operation_id = "listAvailableTools",
    responses(
        (status = 200, body = AvailableToolsResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn list_available_tools_handler(...) -> Result<Json<AvailableToolsResponse>, ApiError>
```

### GET /bodhi/v1/tools/{tool_id}

Get user's config for a specific tool.

```rust
#[utoipa::path(
    get,
    path = "/bodhi/v1/tools/{tool_id}",
    tag = "tools",
    operation_id = "getToolConfig",
    params(("tool_id" = String, Path, description = "Tool identifier")),
    responses(
        (status = 200, body = ToolConfigResponse),
        (status = 404, description = "Tool not found"),
    ),
)]
pub async fn get_tool_config_handler(...) -> Result<Json<ToolConfigResponse>, ApiError>
```

### PUT /bodhi/v1/tools/{tool_id}

Update tool config. **Session-only** (no API tokens) for security.

```rust
#[utoipa::path(
    put,
    path = "/bodhi/v1/tools/{tool_id}",
    tag = "tools",
    operation_id = "updateToolConfig",
    request_body = UpdateToolConfigRequest,
    responses(
        (status = 200, body = ToolConfigResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])  // session only
    ),
)]
pub async fn update_tool_config_handler(...) -> Result<Json<ToolConfigResponse>, ApiError>
```

### POST /bodhi/v1/tools/{tool_id}/execute

Execute a tool. Authorization:
- **First-party (session, bodhiapp_)**: Check tool is configured for user
- **OAuth tokens**: Check tool scope in token

```rust
#[utoipa::path(
    post,
    path = "/bodhi/v1/tools/{tool_id}/execute",
    tag = "tools",
    operation_id = "executeTool",
    request_body = ToolExecutionRequest,
    responses(
        (status = 200, body = ToolExecutionResponse),
        (status = 403, description = "Tool not configured or missing scope"),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),  // first-party, checks config
        ("bearer_oauth_token" = ["scope_tool-builtin-exa-web-search"]),  // OAuth needs scope
        ("session_auth" = ["resource_user"])  // first-party, checks config
    ),
)]
pub async fn execute_tool_handler(...) -> Result<Json<ToolExecutionResponse>, ApiError>
```

## DTOs

```rust
// crates/routes_app/src/tools_dto.rs

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ToolListResponse {
    pub object: String,  // "list"
    pub data: Vec<ToolDefinition>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AvailableToolsResponse {
    pub tools: Vec<AvailableToolInfo>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AvailableToolInfo {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub configured: bool,     // has user configured this?
    pub enabled: bool,        // is it enabled?
    pub scope_required: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ToolConfigResponse {
    pub tool_id: String,
    pub enabled: bool,
    pub has_api_key: bool,  // don't expose actual key
    pub scope_required: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateToolConfigRequest {
    pub enabled: bool,
    #[serde(default)]
    pub api_key: Option<String>,  // only set if changing
}

// Re-export from objs
pub use objs::{ToolExecutionRequest, ToolExecutionResponse};
```

## Route Registration

```rust
// crates/routes_app/src/lib.rs - add to exports
pub mod routes_tools;

// crates/routes_all/src/routes.rs - add routes
let tool_config_apis = Router::new()
    .route("/bodhi/v1/tools", get(list_user_tools_handler))
    .route("/bodhi/v1/tools/available", get(list_available_tools_handler))
    .route("/bodhi/v1/tools/:tool_id", get(get_tool_config_handler))
    .route_layer(from_fn_with_state(state.clone(), move |s, r, n| {
        api_auth_middleware(ResourceRole::User, Some(TokenScope::User), Some(UserScope::User), s, r, n)
    }));

let tool_config_session_apis = Router::new()
    .route("/bodhi/v1/tools/:tool_id", put(update_tool_config_handler))
    .route_layer(from_fn_with_state(state.clone(), move |s, r, n| {
        api_auth_middleware(ResourceRole::User, None, None, s, r, n)  // session only
    }));

let tool_execute_apis = Router::new()
    .route("/bodhi/v1/tools/:tool_id/execute", post(execute_tool_handler))
    .route_layer(from_fn_with_state(state.clone(), tool_auth_middleware));
```
