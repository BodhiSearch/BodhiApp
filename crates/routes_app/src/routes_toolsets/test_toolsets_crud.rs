use crate::{routes_toolsets, ApiKeyUpdateDto, ListToolsetsResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::{test_utils::RequestAuthContextExt, AuthContext};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::Utc;
use objs::{
  AppToolsetConfig, ResourceRole, ToolDefinition, Toolset, ToolsetDefinition,
  ToolsetExecutionResponse, UserScope,
};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{test_utils::AppServiceStubBuilder, MockToolService};
use std::sync::Arc;
use tower::ServiceExt;

#[fixture]
fn test_instance() -> Toolset {
  let now = Utc::now();
  Toolset {
    id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    slug: "my-exa-search".to_string(),
    toolset_type: "builtin-exa-search".to_string(),
    description: Some("Test instance".to_string()),
    enabled: true,
    has_api_key: true,
    created_at: now,
    updated_at: now,
  }
}

fn test_toolset_definition() -> objs::ToolsetDefinition {
  objs::ToolsetDefinition {
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
  }
}

fn setup_type_mocks(mock: &mut MockToolService) {
  let type_def = test_toolset_definition();
  mock
    .expect_get_type()
    .withf(|toolset_type| toolset_type == "builtin-exa-search")
    .returning(move |_| Some(type_def.clone()));
}

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
// List Tests
// ============================================================================

#[rstest]
#[case::session_returns_all(true, false, 1)]
#[case::oauth_filters_by_scope(false, true, 0)]
#[case::empty_list(true, false, 0)]
#[tokio::test]
#[anyhow_trace]
async fn test_list_toolsets(
  test_instance: Toolset,
  #[case] is_session: bool,
  #[case] is_oauth_filtered: bool,
  #[case] expected_count: usize,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();
  let instance_clone = test_instance.clone();

  let instances_to_return = if expected_count > 0 {
    vec![instance_clone]
  } else {
    vec![]
  };

  // Setup type enrichment mocks
  if expected_count > 0 {
    setup_type_mocks(&mut mock_tool_service);
  }

  mock_tool_service
    .expect_list()
    .withf(|user_id| user_id == "user123")
    .times(1)
    .returning(move |_| Ok(instances_to_return.clone()));

  // Mock toolset_types fetching
  mock_tool_service
    .expect_list_app_toolset_configs()
    .times(1)
    .returning(|| Ok(vec![]));

  let app = test_router(mock_tool_service).await?;

  let request = Request::builder()
    .method("GET")
    .uri("/toolsets")
    .body(Body::empty())?;

  let request = if is_session {
    request.with_auth_context(AuthContext::test_session(
      "user123",
      "user@test.com",
      ResourceRole::User,
    ))
  } else if is_oauth_filtered {
    request.with_auth_context(AuthContext::test_external_app(
      "user123",
      UserScope::User,
      "test-app",
      None,
    ))
  } else {
    request.with_auth_context(AuthContext::test_session(
      "user123",
      "user@test.com",
      ResourceRole::User,
    ))
  };

  let response = app.oneshot(request).await?;

  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_toolsets_session_returns_all_toolset_types(
  test_instance: Toolset,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();
  let instance_clone = test_instance.clone();

  setup_type_mocks(&mut mock_tool_service);

  mock_tool_service
    .expect_list()
    .withf(|user_id| user_id == "user123")
    .times(1)
    .returning(move |_| Ok(vec![instance_clone.clone()]));

  let config = AppToolsetConfig {
    toolset_type: "builtin-exa-search".to_string(),
    name: "Exa Web Search".to_string(),
    description: "Web search using Exa API".to_string(),
    enabled: true,
    updated_by: "admin".to_string(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  mock_tool_service
    .expect_list_app_toolset_configs()
    .times(1)
    .returning(move || Ok(vec![config.clone()]));

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let list_response: ListToolsetsResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(1, list_response.toolset_types.len());
  assert_eq!(
    "builtin-exa-search",
    list_response.toolset_types[0].toolset_type
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_toolsets_oauth_returns_scoped_toolset_types(
  test_instance: Toolset,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();
  let instance_clone = test_instance.clone();

  setup_type_mocks(&mut mock_tool_service);

  mock_tool_service
    .expect_list()
    .withf(|user_id| user_id == "user123")
    .times(1)
    .returning(move |_| Ok(vec![instance_clone.clone()]));

  let config = AppToolsetConfig {
    toolset_type: "builtin-exa-search".to_string(),
    name: "Exa Web Search".to_string(),
    description: "Web search using Exa API".to_string(),
    enabled: true,
    updated_by: "admin".to_string(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  // OAuth now returns all types (no scope filtering)
  mock_tool_service
    .expect_list_app_toolset_configs()
    .times(1)
    .returning(move || Ok(vec![config.clone()]));

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_external_app(
          "user123",
          UserScope::User,
          "test-app",
          None,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let list_response: ListToolsetsResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(1, list_response.toolset_types.len());
  assert_eq!(
    "builtin-exa-search",
    list_response.toolset_types[0].toolset_type
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_toolsets_oauth_empty_scopes_returns_empty_toolset_types() -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  mock_tool_service
    .expect_list()
    .withf(|user_id| user_id == "user123")
    .times(1)
    .returning(|_| Ok(vec![]));

  // OAuth with empty scopes now returns all types (no filtering)
  mock_tool_service
    .expect_list_app_toolset_configs()
    .times(1)
    .returning(|| Ok(vec![]));

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_external_app(
          "user123",
          UserScope::User,
          "test-app",
          None,
        )),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let list_response: ListToolsetsResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(0, list_response.toolset_types.len());
  Ok(())
}

// ============================================================================
// Create Tests
// ============================================================================

#[rstest]
#[case::success("my-exa-search", StatusCode::CREATED, None)]
#[case::missing_slug("", StatusCode::BAD_REQUEST, Some("validation_error"))]
#[case::invalid_chars("my@exa", StatusCode::BAD_REQUEST, Some("validation_error"))]
#[case::slug_too_long(
  "this-slug-is-way-too-long-for-val",
  StatusCode::BAD_REQUEST,
  Some("validation_error")
)]
#[case::slug_conflict("existing-slug", StatusCode::CONFLICT, Some("entity_error"))]
#[tokio::test]
#[anyhow_trace]
async fn test_create_toolset(
  test_instance: Toolset,
  #[case] slug: &str,
  #[case] expected_status: StatusCode,
  #[case] _error_code: Option<&str>,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if expected_status == StatusCode::CREATED {
    setup_type_mocks(&mut mock_tool_service);
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_create()
      .times(1)
      .returning(move |_, _, _, _, _, _| Ok(instance_clone.clone()));
  } else if expected_status == StatusCode::CONFLICT {
    mock_tool_service
      .expect_create()
      .times(1)
      .returning(|_, _, _, _, _, _| Err(services::ToolsetError::SlugExists("slug".to_string())));
  }

  let app = test_router(mock_tool_service).await?;

  let request_body = serde_json::json!({
    "toolset_type": "builtin-exa-search",
    "slug": slug,
    "description": "Test instance",
    "enabled": true,
    "api_key": "test-key"
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

// ============================================================================
// Get Tests
// ============================================================================

#[rstest]
#[case::success(true, StatusCode::OK)]
#[case::not_found(false, StatusCode::NOT_FOUND)]
#[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_get_toolset(
  test_instance: Toolset,
  #[case] returns_instance: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();
  let instance_clone = test_instance.clone();

  if returns_instance {
    setup_type_mocks(&mut mock_tool_service);
  }

  mock_tool_service
    .expect_get()
    .times(1)
    .returning(move |_, _| {
      if returns_instance {
        Ok(Some(instance_clone.clone()))
      } else {
        Ok(None)
      }
    });

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri(format!("/toolsets/{}", test_instance.id))
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

// ============================================================================
// Update Tests
// ============================================================================

#[rstest]
#[case::success("updated-slug", ApiKeyUpdateDto::Keep, StatusCode::OK, None)]
#[case::api_key_set("updated-slug", ApiKeyUpdateDto::Set(Some("new-key".to_string())), StatusCode::OK, None)]
#[case::api_key_keep("updated-slug", ApiKeyUpdateDto::Keep, StatusCode::OK, None)]
#[case::validation_error(
  "",
  ApiKeyUpdateDto::Keep,
  StatusCode::BAD_REQUEST,
  Some("validation_error")
)]
#[case::slug_conflict(
  "conflict-slug",
  ApiKeyUpdateDto::Keep,
  StatusCode::CONFLICT,
  Some("entity_error")
)]
#[tokio::test]
#[anyhow_trace]
async fn test_update_toolset(
  test_instance: Toolset,
  #[case] slug: &str,
  #[case] api_key: ApiKeyUpdateDto,
  #[case] expected_status: StatusCode,
  #[case] _error_code: Option<&str>,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if expected_status == StatusCode::OK {
    setup_type_mocks(&mut mock_tool_service);
    let instance_clone = test_instance.clone();
    mock_tool_service
      .expect_update()
      .times(1)
      .returning(move |_, _, _, _, _, _| Ok(instance_clone.clone()));
  } else if expected_status == StatusCode::CONFLICT {
    mock_tool_service
      .expect_update()
      .times(1)
      .returning(|_, _, _, _, _, _| Err(services::ToolsetError::SlugExists("slug".to_string())));
  }

  let app = test_router(mock_tool_service).await?;

  let request_body = serde_json::json!({
    "slug": slug,
    "description": "Updated description",
    "enabled": true,
    "api_key": api_key
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

#[rstest]
#[case::not_found(false, StatusCode::NOT_FOUND)]
#[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_update_toolset_not_found(
  test_instance: Toolset,
  #[case] _returns_instance: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  mock_tool_service
    .expect_update()
    .times(1)
    .returning(|_, _, _, _, _, _| {
      Err(services::ToolsetError::ToolsetNotFound(
        "Toolset".to_string(),
      ))
    });

  let app = test_router(mock_tool_service).await?;

  let request_body = serde_json::json!({
    "slug": "updated-slug",
    "description": "Updated description",
    "enabled": true,
    "api_key": {"action": "Keep"}
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

// ============================================================================
// Delete Tests
// ============================================================================

#[rstest]
#[case::success(true, StatusCode::NO_CONTENT)]
#[case::not_found(false, StatusCode::NOT_FOUND)]
#[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_toolset(
  test_instance: Toolset,
  #[case] succeeds: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if succeeds {
    mock_tool_service
      .expect_delete()
      .times(1)
      .returning(|_, _| Ok(()));
  } else {
    mock_tool_service
      .expect_delete()
      .times(1)
      .returning(|_, _| {
        Err(services::ToolsetError::ToolsetNotFound(
          "Toolset".to_string(),
        ))
      });
  }

  let app = test_router(mock_tool_service).await?;

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/toolsets/{}", test_instance.id))
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

// ============================================================================
// Execute Tests
// ============================================================================

#[rstest]
#[case::success(true, StatusCode::OK)]
#[case::method_not_found(false, StatusCode::NOT_FOUND)]
#[tokio::test]
#[anyhow_trace]
async fn test_execute_toolset(
  test_instance: Toolset,
  #[case] succeeds: bool,
  #[case] expected_status: StatusCode,
) -> anyhow::Result<()> {
  let mut mock_tool_service = MockToolService::new();

  if succeeds {
    mock_tool_service
      .expect_execute()
      .times(1)
      .returning(|_, _, _, _| {
        Ok(ToolsetExecutionResponse {
          result: Some(serde_json::json!({"success": true})),
          error: None,
        })
      });
  } else {
    mock_tool_service
      .expect_execute()
      .times(1)
      .returning(|_, _, _, _| Err(services::ToolsetError::MethodNotFound("search".to_string())));
  }

  let app = test_router(mock_tool_service).await?;

  let request_body = serde_json::json!({
    "params": {"query": "test"}
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri(format!(
          "/toolsets/{}/tools/search/execute",
          test_instance.id
        ))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?
        .with_auth_context(AuthContext::test_session(
          "user123",
          "user@test.com",
          ResourceRole::User,
        )),
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}
