# Routes & API - Toolsets Feature

> Layer: `routes_app` crate | Status: ✅ Complete

## Implementation Note

Routes are implemented with proper middleware integration. Endpoints use `/toolsets` prefix with `toolset_id` path parameter.

## Endpoints Summary

**Files**: `crates/routes_app/src/{routes_toolsets.rs, toolsets_dto.rs}`

| Method | Path | Handler | Auth | Description |
|--------|------|---------|------|-------------|
| GET | `/toolsets` | `list_all_toolsets()` | Session, OAuth | List toolsets (OAuth: filtered by scope) |
| GET | `/toolsets/:toolset_id/config` | `get_toolset_config()` | Session only | Get user's toolset config |
| PUT | `/toolsets/:toolset_id/config` | `update_toolset_config()` | Session only | Update toolset config |
| DELETE | `/toolsets/:toolset_id/config` | `delete_toolset_config()` | Session only | Delete toolset config |
| POST | `/toolsets/:toolset_id/execute/:method` | `execute_toolset()` | Session, OAuth | Execute a tool method |
| PUT | `/toolsets/:toolset_id/app-config` | `enable_app_toolset()` | Admin only | Admin enable |
| DELETE | `/toolsets/:toolset_id/app-config` | `disable_app_toolset()` | Admin only | Admin disable |

Note: Paths shown are route-level. When integrated into `routes_all`, they are prefixed with `/bodhi/v1`.

**Important:** API tokens (`bodhiapp_*`) return 401 for all toolset endpoints.

## Endpoint Details

### GET /bodhi/v1/toolsets

List all available toolsets with their tools and configuration status.

**Auth behavior:**
- **Session**: Returns all toolsets
- **OAuth**: Returns only toolsets matching `scope_toolset-*` scopes in token
- **API Token**: Returns 401

**Response:**
```json
{
  "toolsets": [
    {
      "toolset_id": "builtin-exa-web-search",
      "name": "Exa Web Search",
      "description": "Search the web using Exa AI semantic search",
      "app_enabled": true,
      "user_config": { "enabled": true, "has_api_key": true },
      "tools": [
        {
          "type": "function",
          "function": {
            "name": "toolset__builtin-exa-web-search__search",
            "description": "Search the web for current information",
            "parameters": { ... }
          }
        },
        { /* find_similar tool */ },
        { /* get_contents tool */ },
        { /* answer tool */ }
      ]
    }
  ]
}
```

### GET /bodhi/v1/toolsets/{toolset_id}/config

Get user's configuration for a specific toolset (with app-level status).

**Response:**
```json
{
  "toolset_id": "builtin-exa-web-search",
  "app_enabled": true,
  "config": {
    "toolset_id": "builtin-exa-web-search",
    "enabled": true,
    "created_at": "2026-01-15T10:00:00Z",
    "updated_at": "2026-01-15T10:00:00Z"
  }
}
```

### PUT /bodhi/v1/toolsets/{toolset_id}/config

Update toolset config. **Session-only** (no API tokens) for security.

**Request:**
```json
{
  "enabled": true,
  "api_key": "exa-api-key-here"
}
```

**Response:** `EnhancedToolsetConfigResponse` (same as GET)

**Note:** Returns 400 if toolset is disabled at app level.

### DELETE /bodhi/v1/toolsets/{toolset_id}/config

Delete toolset configuration (clears API key). Always allowed.

**Response:** 204 No Content

### POST /bodhi/v1/toolsets/{toolset_id}/execute/{method}

Execute a specific method within the toolset. The `method` path parameter specifies which tool to execute (e.g., `search`, `find_similar`, `get_contents`, `answer`).

**Authorization:**
- **Session**: Check app-level enabled + user has API key configured
- **OAuth tokens**: Check app-level enabled + app-client registered + scope in token + user has API key
- **API Token**: Returns 401 (blocked at route level)

**Request:**
```json
{
  "tool_call_id": "call_abc123",
  "params": {
    "query": "rust programming language",
    "num_results": 5
  }
}
```

**Response:**
```json
{
  "tool_call_id": "call_abc123",
  "result": {
    "results": [
      { "title": "...", "url": "...", "text": "..." }
    ]
  }
}
```

**Error Response:**
```json
{
  "tool_call_id": "call_abc123",
  "result": null,
  "error": "exa_rate_limited: Rate limit exceeded"
}
```

### PUT /bodhi/v1/toolsets/{toolset_id}/app-config

Enable toolset at app level (admin only).

**Response:**
```json
{
  "config": {
    "toolset_id": "builtin-exa-web-search",
    "enabled": true,
    "updated_by": "admin-user-id",
    "created_at": "...",
    "updated_at": "..."
  }
}
```

### DELETE /bodhi/v1/toolsets/{toolset_id}/app-config

Disable toolset at app level (admin only).

**Response:** Same as PUT (with `enabled: false`)

## DTOs

```rust
// crates/routes_app/src/toolsets_dto.rs

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListToolsetsResponse {
    pub toolsets: Vec<ToolsetListItem>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ToolsetListItem {
    pub toolset_id: String,
    pub name: String,
    pub description: String,
    pub app_enabled: bool,
    pub user_config: Option<UserToolsetConfigSummary>,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserToolsetConfigSummary {
    pub enabled: bool,
    pub has_api_key: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EnhancedToolsetConfigResponse {
    pub toolset_id: String,
    pub app_enabled: bool,
    pub config: UserToolsetConfig,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateToolsetConfigRequest {
    pub enabled: bool,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ExecuteToolsetRequest {
    pub tool_call_id: String,
    pub params: serde_json::Value,  // Method-specific parameters
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AppToolsetConfigResponse {
    pub config: AppToolsetConfig,
}

// Re-export from objs
pub use objs::ToolsetExecutionResponse;
```

## Route Registration

```rust
// crates/routes_all/src/routes.rs

// Session-only config APIs (no API tokens, no OAuth)
let user_session_apis = Router::new()
    .route("/bodhi/v1/toolsets/:toolset_id/config", get(get_toolset_config_handler))
    .route("/bodhi/v1/toolsets/:toolset_id/config", put(update_toolset_config_handler))
    .route("/bodhi/v1/toolsets/:toolset_id/config", delete(delete_toolset_config_handler))
    .route_layer(from_fn_with_state(state.clone(), move |s, r, n| {
        api_auth_middleware(ResourceRole::User, None, None, s, r, n)  // session only
    }));

// OAuth-allowed APIs (session + OAuth, NOT API tokens)
let user_oauth_apis = Router::new()
    .route("/bodhi/v1/toolsets", get(list_all_toolsets_handler))
    .route("/bodhi/v1/toolsets/:toolset_id/execute/:method", post(execute_toolset_handler))
    .route_layer(from_fn_with_state(state.clone(), move |s, r, n| {
        api_auth_middleware(ResourceRole::User, None, Some(UserScope::User), s, r, n)
    }));

let toolset_admin_apis = Router::new()
    .route("/bodhi/v1/toolsets/:toolset_id/app-config", put(enable_app_toolset))
    .route("/bodhi/v1/toolsets/:toolset_id/app-config", delete(disable_app_toolset))
    .route_layer(from_fn_with_state(state.clone(), move |s, r, n| {
        api_auth_middleware(ResourceRole::Admin, None, None, s, r, n)
    }));
```

**Note:** The route middleware configuration ensures:
- `user_session_apis`: `TokenScope=None` means API tokens are rejected (401)
- `user_oauth_apis`: `TokenScope=None, UserScope=User` allows session and OAuth tokens, rejects API tokens

## API-Level Validation

- `PUT /toolsets/:toolset_id/config` returns 400 if app-level disabled
- `POST /toolsets/:toolset_id/execute` returns 400 if app-level disabled
- `DELETE /toolsets/:toolset_id/config` always allowed (cleanup)

## Test Coverage

Tests cover:
- All handler endpoints
- Request/response validation
- Error cases (not found, not configured, app disabled)
- Tool name parsing from execute request
