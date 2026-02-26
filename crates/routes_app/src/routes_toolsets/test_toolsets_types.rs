use crate::routes_toolsets;
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use objs::{AppToolsetConfig, ResourceRole, ToolDefinition, ToolsetDefinition, UserScope};
use rstest::rstest;
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{test_utils::AppServiceStubBuilder, MockToolService};
use std::sync::Arc;
use tower::ServiceExt;

async fn test_router(mock_tool_service: MockToolService) -> anyhow::Result<Router> {
  let app_service = AppServiceStubBuilder::default()
    .with_tool_service(Arc::new(mock_tool_service))
    .build()
    .await?;

  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));

  Ok(routes_toolsets(state))
}

// ============================================================================
// List Toolset Types Tests
// ============================================================================

#[rstest]
#[case::session_returns_all(true, 1)]
#[case::oauth_filters(false, 0)]
#[tokio::test]
#[anyhow_trace]
async fn test_list_toolset_types(
  #[case] is_session: bool,
  #[case] expected_count: usize,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  let toolset_type = ToolsetDefinition {
    toolset_type: "builtin-exa-search".to_string(),
    name: "Exa Web Search".to_string(),
    description: "Web search using Exa API".to_string(),
    tools: vec![ToolDefinition {
      tool_type: "function".to_string(),
      function: objs::FunctionDefinition {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        parameters: serde_json::json!({}),
      },
    }],
  };

  let types_to_return = if expected_count > 0 {
    vec![toolset_type]
  } else {
    vec![]
  };

  mock_tool_service
    .expect_list_types()
    .times(1)
    .returning(move || types_to_return.clone());

  let app = test_router(mock_tool_service).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/toolset_types")
    .body(Body::empty())?;

  let request = if is_session {
    request.with_auth_context(AuthContext::test_session(
      "user123",
      "user@test.com",
      ResourceRole::Admin,
    ))
  } else {
    request.with_auth_context(AuthContext::test_external_app(
      "user123",
      UserScope::PowerUser,
      "test-app",
      None,
    ))
  };

  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

// ============================================================================
// Enable/Disable Type Tests
// ============================================================================

#[rstest]
#[case::success(true, StatusCode::OK)]
#[case::type_not_found(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_enable_type(
  #[case] succeeds: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if succeeds {
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .withf(|toolset_type, enabled, _user_id| toolset_type == "builtin-exa-search" && *enabled)
      .times(1)
      .returning(|toolset_type, _, updated_by| {
        Ok(AppToolsetConfig {
          toolset_type: toolset_type.to_string(),
          name: "Exa Web Search".to_string(),
          description: "Web search using Exa API".to_string(),
          enabled: true,
          updated_by: updated_by.to_string(),
          created_at: chrono::Utc::now(),
          updated_at: chrono::Utc::now(),
        })
      });
  } else {
    // Validation fails, returning error
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .times(1)
      .returning(|toolset_type, _, _| {
        Err(services::ToolsetError::InvalidToolsetType(
          toolset_type.to_string(),
        ))
      });
  }

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/toolset_types/builtin-exa-search/app-config")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_with_token(
          "admin123",
          "admin@test.com",
          ResourceRole::Admin,
          "admin-token",
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

#[rstest]
#[case::success(true, StatusCode::OK)]
#[case::type_not_found(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_disable_type(
  #[case] succeeds: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if succeeds {
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .withf(|toolset_type, enabled, _user_id| toolset_type == "builtin-exa-search" && !(*enabled))
      .times(1)
      .returning(|toolset_type, _, updated_by| {
        Ok(AppToolsetConfig {
          toolset_type: toolset_type.to_string(),
          name: "Exa Web Search".to_string(),
          description: "Web search using Exa API".to_string(),
          enabled: false,
          updated_by: updated_by.to_string(),
          created_at: chrono::Utc::now(),
          updated_at: chrono::Utc::now(),
        })
      });
  } else {
    // Validation fails, returning error
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .times(1)
      .returning(|toolset_type, _, _| {
        Err(services::ToolsetError::InvalidToolsetType(
          toolset_type.to_string(),
        ))
      });
  }

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/toolset_types/builtin-exa-search/app-config")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session_with_token(
          "admin123",
          "admin@test.com",
          ResourceRole::Admin,
          "admin-token",
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}
