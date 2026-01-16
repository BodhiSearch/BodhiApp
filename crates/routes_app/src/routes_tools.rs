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
// Endpoint Constants
// ============================================================================

pub const ENDPOINT_TOOLS: &str = "/bodhi/v1/tools";

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes_tools(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .route("/tools", get(list_all_tools_handler))
    .route("/tools/:tool_id/config", get(get_tool_config_handler))
    .route("/tools/:tool_id/config", put(update_tool_config_handler))
    .route("/tools/:tool_id/config", delete(delete_tool_config_handler))
    .route("/tools/:tool_id/execute", post(execute_tool_handler))
    // Admin routes for app-level tool configuration
    .route("/tools/:tool_id/app-config", put(enable_app_tool_handler))
    .route(
      "/tools/:tool_id/app-config",
      delete(disable_app_tool_handler),
    )
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
pub async fn list_all_tools_handler(
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
pub async fn get_tool_config_handler(
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
    (status = 400, description = "Tool is disabled at app level"),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
pub async fn update_tool_config_handler(
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

  // Reject update if tool is disabled at app level
  if !app_enabled {
    return Err(
      objs::BadRequestError::new("Tool is disabled at app level. Enable it first.".to_string())
        .into(),
    );
  }

  let config = tool_service
    .update_user_tool_config(&user_id, &tool_id, request.enabled, request.api_key)
    .await?;

  Ok(Json(EnhancedToolConfigResponse {
    tool_id,
    app_enabled,
    config,
  }))
}

/// Delete user's tool configuration (clears API key)
#[utoipa::path(
  delete,
  path = "/tools/{tool_id}/config",
  tag = "tools",
  params(
    ("tool_id" = String, Path, description = "Tool identifier")
  ),
  responses(
    (status = 204, description = "Tool configuration deleted"),
    (status = 404, description = "Tool not found"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_tool_config_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
) -> Result<axum::http::StatusCode, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  tool_service
    .delete_user_tool_config(&user_id, &tool_id)
    .await?;

  Ok(axum::http::StatusCode::NO_CONTENT)
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
pub async fn execute_tool_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(tool_id): Path<String>,
  headers: HeaderMap,
  Json(request): Json<ExecuteToolRequest>,
) -> Result<Json<ToolExecutionResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  // Check if tool is enabled at app level
  let app_enabled = tool_service
    .is_tool_enabled_for_app(&tool_id)
    .await
    .unwrap_or(false);

  if !app_enabled {
    return Err(objs::BadRequestError::new("Tool is disabled at app level.".to_string()).into());
  }

  let response = tool_service
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
pub async fn enable_app_tool_handler(
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
pub async fn disable_app_tool_handler(
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
  use auth_middleware::KEY_HEADER_BODHIAPP_USER_ID;
  use axum::{
    http::{Request, StatusCode},
    routing::put,
    Router,
  };
  use mockall::predicate::eq;
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use rstest::rstest;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{test_utils::AppServiceStubBuilder, MockToolService};
  use std::sync::Arc;
  use tower::ServiceExt;

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

  fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
    Router::new()
      .route("/tools/{tool_id}/config", put(update_tool_config_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_tool_config_returns_400_when_app_disabled(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    // Setup mock tool service that returns app_enabled = false
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_enabled_for_app()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(false));

    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .build()?;

    let router = test_router(Arc::new(app_service));

    let resp = router
      .oneshot(
        Request::put("/tools/builtin-exa-web-search/config")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "test-user-id")
          .json(serde_json::json!({
            "enabled": true,
            "api_key": "test-api-key"
          }))?,
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, resp.status());
    let body: serde_json::Value = resp.json().await?;
    assert!(body["error"]["message"]
      .as_str()
      .unwrap()
      .contains("disabled at app level"));

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_tool_config_succeeds_when_app_enabled(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    // Setup mock tool service that returns app_enabled = true
    let mut mock_tool_service = MockToolService::new();
    mock_tool_service
      .expect_is_tool_enabled_for_app()
      .with(eq("builtin-exa-web-search"))
      .returning(|_| Ok(true));

    // Mock the update call
    mock_tool_service
      .expect_update_user_tool_config()
      .with(
        eq("test-user-id"),
        eq("builtin-exa-web-search"),
        eq(true),
        eq(Some("test-api-key".to_string())),
      )
      .returning(|_, tool_id, enabled, _| {
        Ok(objs::UserToolConfig {
          tool_id: tool_id.to_string(),
          enabled,
          created_at: chrono::Utc::now(),
          updated_at: chrono::Utc::now(),
        })
      });

    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .build()?;

    let router = test_router(Arc::new(app_service));

    let resp = router
      .oneshot(
        Request::put("/tools/builtin-exa-web-search/config")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "test-user-id")
          .json(serde_json::json!({
            "enabled": true,
            "api_key": "test-api-key"
          }))?,
      )
      .await?;

    assert_eq!(StatusCode::OK, resp.status());
    let body: EnhancedToolConfigResponse = resp.json().await?;
    assert_eq!("builtin-exa-web-search", body.tool_id);
    assert!(body.app_enabled);
    assert!(body.config.enabled);

    Ok(())
  }
}
