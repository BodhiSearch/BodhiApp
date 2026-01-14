use crate::tools_dto::{
  ExecuteToolRequest, GetToolConfigResponse, ListToolsResponse, UpdateToolConfigRequest,
};
use axum::{
  extract::{Path, State},
  http::HeaderMap,
  routing::{get, post, put},
  Json, Router,
};
use objs::{ApiError, ToolExecutionResponse};
use server_core::RouterState;
use std::sync::Arc;

// Temporary helper until Phase 7 integrates proper auth middleware
// In Phase 7, this will be replaced with proper Claims extraction
fn extract_user_id_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
  // This is a placeholder - Phase 7 will add proper middleware that extracts Claims
  // For now, routes will compile but require auth middleware to be functional
  headers
    .get("x-user-id")
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
    .ok_or_else(|| {
      objs::BadRequestError::new("User ID not found in request headers".to_string()).into()
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
    .with_state(state)
}

// ============================================================================
// Handlers
// ============================================================================

/// List all available tool definitions (for UI)
#[utoipa::path(
  get,
  path = "/tools",
  tag = "tools",
  responses(
    (status = 200, description = "List of all available tools", body = ListToolsResponse),
  ),
  security(("bearer" = []))
)]
async fn list_all_tools(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<ListToolsResponse>, ApiError> {
  let tools = state
    .app_service()
    .tool_service()
    .list_all_tool_definitions();

  Ok(Json(ListToolsResponse { tools }))
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
  let tools = state
    .app_service()
    .tool_service()
    .list_tools_for_user(&user_id)
    .await?;

  Ok(Json(ListToolsResponse { tools }))
}

/// Get user's configuration for a specific tool
#[utoipa::path(
  get,
  path = "/tools/{tool_id}/config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  responses(
    (status = 200, description = "Tool configuration", body = GetToolConfigResponse),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn get_tool_config(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<GetToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let config = state
    .app_service()
    .tool_service()
    .get_user_tool_config(&user_id, &tool_id)
    .await?;

  let config = match config {
    Some(config) => config,
    None => {
      // Return default config if not found
      let now = chrono::Utc::now();
      objs::UserToolConfig {
        tool_id,
        enabled: false,
        created_at: now,
        updated_at: now,
      }
    }
  };

  Ok(Json(GetToolConfigResponse { config }))
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
    (status = 200, description = "Updated tool configuration", body = GetToolConfigResponse),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
async fn update_tool_config(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
  Json(request): Json<UpdateToolConfigRequest>,
) -> Result<Json<GetToolConfigResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let config = state
    .app_service()
    .tool_service()
    .update_user_tool_config(&user_id, &tool_id, request.enabled, request.api_key)
    .await?;

  Ok(Json(GetToolConfigResponse { config }))
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
