use crate::tools_dto::{
  AppToolConfigResponse, EnhancedToolConfigResponse, ExecuteToolRequest, ListToolsResponse,
  ToolListItem, UpdateToolConfigRequest, UserToolConfigSummary,
};
use auth_middleware::{KEY_HEADER_BODHIAPP_TOKEN, KEY_HEADER_BODHIAPP_USER_ID};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
  routing::{delete, get, post, put},
  Json, Router,
};
use objs::{ApiError, ToolExecutionResponse};
use server_core::RouterState;
use std::sync::Arc;

// Extract user_id from headers (set by auth middleware)
fn extract_user_id_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
  headers
    .get(KEY_HEADER_BODHIAPP_USER_ID)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
    .ok_or_else(|| {
      objs::BadRequestError::new("User ID not found in request headers".to_string()).into()
    })
}

// Extract access token from headers (for admin operations)
fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
  headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
    .ok_or_else(|| {
      objs::BadRequestError::new("Access token not found in request headers".to_string()).into()
    })
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes_tools(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .route("/tools", get(list_all_tools))
    .route("/tools/configured", get(list_configured_tools))
    .route("/tools/:tool_id/config", get(get_tool_config))
    .route("/tools/:tool_id/config", put(update_tool_config))
    .route("/tools/:tool_id/execute", post(execute_tool))
    // Admin routes for app-level tool configuration
    .route("/tools/:tool_id/app-config", put(enable_app_tool))
    .route("/tools/:tool_id/app-config", delete(disable_app_tool))
    .with_state(state)
}

// ============================================================================
// Handlers
// ============================================================================

/// List all available tool definitions with app-enabled status (for UI)
#[utoipa::path(
  get,
  path = "/tools",
  tag = "tools",
  responses(
    (status = 200, description = "List of all available tools with status", body = ListToolsResponse),
  ),
  security(("bearer" = []))
)]
async fn list_all_tools(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();
  let tools = tool_service.list_all_tool_definitions();

  // Get user_id if available (for user config summary)
  let user_id = extract_user_id_from_headers(&headers).ok();

  // Build enhanced tool list with app-enabled status
  let mut items = Vec::new();
  for tool in tools {
    let tool_id = &tool.function.name;

    // Get app-level enabled status
    let app_enabled = tool_service
      .is_tool_enabled_for_app(tool_id)
      .await
      .unwrap_or(false);

    // Get user config summary if user_id is available
    let user_config = if let Some(ref uid) = user_id {
      tool_service
        .get_user_tool_config(uid, tool_id)
        .await
        .ok()
        .flatten()
        .map(|c| UserToolConfigSummary {
          enabled: c.enabled,
          has_api_key: true, // If we got a config, assume API key was validated
        })
    } else {
      None
    };

    items.push(ToolListItem {
      definition: tool,
      app_enabled,
      user_config,
    });
  }

  Ok(Json(ListToolsResponse { tools: items }))
}

/// List configured and enabled tools for current user
#[utoipa::path(
  get,
  path = "/tools/configured",
  tag = "tools",
  responses(
    (status = 200, description = "List of configured tools for user", body = ListToolsResponse),
  ),
  security(("bearer" = []))
)]
async fn list_configured_tools(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();
  let tools = tool_service.list_tools_for_user(&user_id).await?;

  // Build enhanced tool list - these are already configured tools
  let mut items = Vec::new();
  for tool in tools {
    let tool_id = &tool.function.name;

    // Get app-level enabled status
    let app_enabled = tool_service
      .is_tool_enabled_for_app(tool_id)
      .await
      .unwrap_or(false);

    items.push(ToolListItem {
      definition: tool,
      app_enabled,
      user_config: Some(UserToolConfigSummary {
        enabled: true,
        has_api_key: true,
      }),
    });
  }

  Ok(Json(ListToolsResponse { tools: items }))
}

/// Get user's configuration for a specific tool (with app-level status)
#[utoipa::path(
  get,
  path = "/tools/{tool_id}/config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  responses(
    (status = 200, description = "Tool configuration with app status", body = EnhancedToolConfigResponse),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn get_tool_config(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<EnhancedToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  // Get app-level enabled status
  let app_enabled = tool_service
    .is_tool_enabled_for_app(&tool_id)
    .await
    .unwrap_or(false);

  let config = tool_service
    .get_user_tool_config(&user_id, &tool_id)
    .await?;

  let config = match config {
    Some(config) => config,
    None => {
      // Return default config if not found
      let now = chrono::Utc::now();
      objs::UserToolConfig {
        tool_id: tool_id.clone(),
        enabled: false,
        created_at: now,
        updated_at: now,
      }
    }
  };

  Ok(Json(EnhancedToolConfigResponse {
    tool_id,
    app_enabled,
    config,
  }))
}

/// Update user's tool configuration
#[utoipa::path(
  put,
  path = "/tools/{tool_id}/config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  request_body = UpdateToolConfigRequest,
  responses(
    (status = 200, description = "Updated tool configuration", body = EnhancedToolConfigResponse),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn update_tool_config(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
  Json(request): Json<UpdateToolConfigRequest>,
) -> Result<Json<EnhancedToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  // Get app-level enabled status
  let app_enabled = tool_service
    .is_tool_enabled_for_app(&tool_id)
    .await
    .unwrap_or(false);

  let config = tool_service
    .update_user_tool_config(&user_id, &tool_id, request.enabled, request.api_key)
    .await?;

  Ok(Json(EnhancedToolConfigResponse {
    tool_id,
    app_enabled,
    config,
  }))
}

/// Execute a tool for the user
#[utoipa::path(
  post,
  path = "/tools/{tool_id}/execute",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  request_body = ExecuteToolRequest,
  responses(
    (status = 200, description = "Tool execution result", body = ToolExecutionResponse),
    (status = 400, description = "Tool not configured or disabled"),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn execute_tool(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
  Json(request): Json<ExecuteToolRequest>,
) -> Result<Json<ToolExecutionResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let response = state
    .app_service()
    .tool_service()
    .execute_tool(&user_id, &tool_id, request.into())
    .await?;

  Ok(Json(response))
}

// ============================================================================
// Admin Handlers (App-level tool configuration)
// ============================================================================

/// Enable a tool for this app instance (admin only)
#[utoipa::path(
  put,
  path = "/tools/{tool_id}/app-config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  responses(
    (status = 200, description = "Tool enabled for app instance", body = AppToolConfigResponse),
    (status = 403, description = "Admin access required"),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn enable_app_tool(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<AppToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let admin_token = extract_token_from_headers(&headers)?;

  let config = state
    .app_service()
    .tool_service()
    .set_app_tool_enabled(&admin_token, &tool_id, true, &user_id)
    .await?;

  Ok(Json(AppToolConfigResponse { config }))
}

/// Disable a tool for this app instance (admin only)
#[utoipa::path(
  delete,
  path = "/tools/{tool_id}/app-config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  responses(
    (status = 200, description = "Tool disabled for app instance", body = AppToolConfigResponse),
    (status = 403, description = "Admin access required"),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn disable_app_tool(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<AppToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let admin_token = extract_token_from_headers(&headers)?;

  let config = state
    .app_service()
    .tool_service()
    .set_app_tool_enabled(&admin_token, &tool_id, false, &user_id)
    .await?;

  Ok(Json(AppToolConfigResponse { config }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  // Note: These handlers require Claims extractor which needs the full auth middleware stack.
  // Integration tests in Phase 9 will test the complete flow with authentication.
  // Unit tests here focus on DTO serialization/deserialization.

  #[rstest]
  fn test_update_tool_config_request_serialization() {
    let req = UpdateToolConfigRequest {
      enabled: true,
      api_key: Some("test-key".to_string()),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(true, json["enabled"]);
    assert_eq!("test-key", json["api_key"]);
  }

  #[rstest]
  fn test_execute_tool_request_serialization() {
    let req = ExecuteToolRequest {
      tool_call_id: "call_123".to_string(),
      arguments: serde_json::json!({"query": "test"}),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!("call_123", json["tool_call_id"]);
    assert_eq!("test", json["arguments"]["query"]);
  }
}
