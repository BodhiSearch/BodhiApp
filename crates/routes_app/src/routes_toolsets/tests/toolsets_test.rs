use crate::{routes_toolsets, ApiKeyUpdateDto, ListToolsetsResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOOL_SCOPES,
};
use auth_middleware::KEY_HEADER_BODHIAPP_USER_ID;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use chrono::Utc;
use objs::{
  AppToolsetConfig, ResourceRole, ToolDefinition, Toolset, ToolsetExecutionResponse,
  ToolsetWithTools,
};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use server_core::test_utils::RequestAuthExt;
use server_core::{DefaultRouterState, MockSharedContext, RouterState};
use services::{test_utils::AppServiceStubBuilder, MockToolService};
use std::sync::Arc;
use tower::ServiceExt;

#[fixture]
fn test_instance() -> Toolset {
  let now = Utc::now();
  Toolset {
    id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    name: "My Exa Search".to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
    description: Some("Test instance".to_string()),
    enabled: true,
    has_api_key: true,
    created_at: now,
    updated_at: now,
  }
}

fn test_toolset_definition() -> objs::ToolsetDefinition {
  objs::ToolsetDefinition {
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
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
    .withf(|scope_uuid| scope_uuid == "4ff0e163-36fb-47d6-a5ef-26e396f067d6")
    .returning(move |_| Some(type_def.clone()));
}

fn test_router(mock_tool_service: MockToolService) -> anyhow::Result<Router> {
  let app_service = AppServiceStubBuilder::default()
    .with_tool_service(Arc::new(mock_tool_service))
    .build()?;

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

  // Mock toolset_types fetching based on auth type
  if is_session {
    mock_tool_service
      .expect_list_app_toolset_configs()
      .times(1)
      .returning(|| Ok(vec![]));
  } else if is_oauth_filtered {
    mock_tool_service
      .expect_list_app_toolset_configs_by_scopes()
      .times(1)
      .returning(|_| Ok(vec![]));
  }

  let app = test_router(mock_tool_service)?;

  let mut request_builder = Request::builder()
    .method("GET")
    .uri("/toolsets")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "user123");

  if is_session {
    request_builder =
      request_builder.header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string());
  } else if is_oauth_filtered {
    request_builder = request_builder
      .header(KEY_HEADER_BODHIAPP_SCOPE, "scope_user_user")
      .header(KEY_HEADER_BODHIAPP_TOOL_SCOPES, "");
  }

  let response = app
    .oneshot(request_builder.body(Body::empty())?)
    .await?;

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
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    enabled: true,
    updated_by: "admin".to_string(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  mock_tool_service
    .expect_list_app_toolset_configs()
    .times(1)
    .returning(move || Ok(vec![config.clone()]));

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let list_response: ListToolsetsResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(1, list_response.toolset_types.len());
  assert_eq!(
    "scope_toolset-builtin-exa-web-search",
    list_response.toolset_types[0].scope
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
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    enabled: true,
    updated_by: "admin".to_string(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  // OAuth should use the scoped query method
  mock_tool_service
    .expect_list_app_toolset_configs_by_scopes()
    .withf(|scopes| scopes.len() == 1 && scopes[0] == "scope_toolset-builtin-exa-web-search")
    .times(1)
    .returning(move |_| Ok(vec![config.clone()]));

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_SCOPE, "scope_user_user")
        .header(
          KEY_HEADER_BODHIAPP_TOOL_SCOPES,
          "scope_toolset-builtin-exa-web-search",
        )
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());

  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
  let list_response: ListToolsetsResponse = serde_json::from_slice(&body_bytes)?;

  assert_eq!(1, list_response.toolset_types.len());
  assert_eq!(
    "scope_toolset-builtin-exa-web-search",
    list_response.toolset_types[0].scope
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

  // OAuth with empty scopes should call with empty vec
  mock_tool_service
    .expect_list_app_toolset_configs_by_scopes()
    .withf(|scopes| scopes.is_empty())
    .times(1)
    .returning(|_| Ok(vec![]));

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/toolsets")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_SCOPE, "scope_user_user")
        .header(KEY_HEADER_BODHIAPP_TOOL_SCOPES, "")
        .body(Body::empty())?,
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
#[case::success("My Exa Search", StatusCode::CREATED, None)]
#[case::missing_name("", StatusCode::BAD_REQUEST, Some("validation_error"))]
#[case::invalid_chars("My@Exa", StatusCode::BAD_REQUEST, Some("validation_error"))]
#[case::name_too_long(
  "This name is way too long for validation",
  StatusCode::BAD_REQUEST,
  Some("validation_error")
)]
#[case::name_conflict("Existing Name", StatusCode::CONFLICT, Some("entity_error"))]
#[tokio::test]
#[anyhow_trace]
async fn test_create_toolset(
  test_instance: Toolset,
  #[case] name: &str,
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
      .returning(|_, _, _, _, _, _| Err(services::ToolsetError::NameExists("name".to_string())));
  }

  let app = test_router(mock_tool_service)?;

  let request_body = serde_json::json!({
    "scope_uuid": "4ff0e163-36fb-47d6-a5ef-26e396f067d6",
    "name": name,
    "description": "Test instance",
    "enabled": true,
    "api_key": "test-key"
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/toolsets")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?,
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

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("GET")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}

// ============================================================================
// Update Tests
// ============================================================================

#[rstest]
#[case::success("Updated Name", ApiKeyUpdateDto::Keep, StatusCode::OK, None)]
#[case::api_key_set("Updated Name", ApiKeyUpdateDto::Set(Some("new-key".to_string())), StatusCode::OK, None)]
#[case::api_key_keep("Updated Name", ApiKeyUpdateDto::Keep, StatusCode::OK, None)]
#[case::validation_error(
  "",
  ApiKeyUpdateDto::Keep,
  StatusCode::BAD_REQUEST,
  Some("validation_error")
)]
#[case::name_conflict(
  "Conflict Name",
  ApiKeyUpdateDto::Keep,
  StatusCode::CONFLICT,
  Some("entity_error")
)]
#[tokio::test]
#[anyhow_trace]
async fn test_update_toolset(
  test_instance: Toolset,
  #[case] name: &str,
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
      .returning(|_, _, _, _, _, _| Err(services::ToolsetError::NameExists("name".to_string())));
  }

  let app = test_router(mock_tool_service)?;

  let request_body = serde_json::json!({
    "name": name,
    "description": "Updated description",
    "enabled": true,
    "api_key": api_key
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?,
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

  let app = test_router(mock_tool_service)?;

  let request_body = serde_json::json!({
    "name": "Updated Name",
    "description": "Updated description",
    "enabled": true,
    "api_key": {"action": "Keep"}
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?,
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

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/toolsets/{}", test_instance.id))
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .body(Body::empty())?,
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

  let app = test_router(mock_tool_service)?;

  let request_body = serde_json::json!({
    "params": {"query": "test"}
  });

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri(format!("/toolsets/{}/execute/search", test_instance.id))
        .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
        .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body)?))?,
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
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

  let toolset_type = ToolsetWithTools {
    scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
    scope: "scope_toolset-builtin-exa-web-search".to_string(),
    name: "Exa Web Search".to_string(),
    description: "Web search using Exa API".to_string(),
    app_enabled: true,
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
    .expect_list_all_toolsets()
    .times(1)
    .returning(move || Ok(types_to_return.clone()));

  let app = test_router(mock_tool_service)?;

  let mut request_builder = Request::builder()
    .method("GET")
    .uri("/toolset_types")
    .header(KEY_HEADER_BODHIAPP_USER_ID, "user123");

  if is_session {
    request_builder =
      request_builder.header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::Admin.to_string());
  } else {
    request_builder = request_builder
      .header(KEY_HEADER_BODHIAPP_SCOPE, "scope_user_admin")
      .header(KEY_HEADER_BODHIAPP_TOOL_SCOPES, "");
  }

  let response = app
    .oneshot(request_builder.body(Body::empty())?)
    .await?;

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

  // Mock list_types to return toolset definition
  mock_tool_service
    .expect_list_types()
    .times(1)
    .returning(move || {
      if succeeds {
        vec![test_toolset_definition()]
      } else {
        vec![]
      }
    });

  if succeeds {
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .times(1)
      .returning(|_, _, _, _, _| {
        Ok(AppToolsetConfig {
          scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
          scope: "scope_toolset-builtin-exa-web-search".to_string(),
          enabled: true,
          updated_by: "admin123".to_string(),
          created_at: chrono::Utc::now(),
          updated_at: chrono::Utc::now(),
        })
      });
  }
  // When succeeds is false, list_types returns empty vec and handler returns early with Not Found
  // No need to mock set_app_toolset_enabled in that case

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/toolset_types/scope_toolset-builtin-exa-web-search/app-config")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "admin123")
        .with_user_auth("admin-token", &ResourceRole::Admin.to_string())
        .body(Body::empty())?,
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

  // Mock list_types to return toolset definition
  mock_tool_service
    .expect_list_types()
    .times(1)
    .returning(move || {
      if succeeds {
        vec![test_toolset_definition()]
      } else {
        vec![]
      }
    });

  if succeeds {
    mock_tool_service
      .expect_set_app_toolset_enabled()
      .times(1)
      .returning(|_, _, _, _, _| {
        Ok(AppToolsetConfig {
          scope_uuid: "4ff0e163-36fb-47d6-a5ef-26e396f067d6".to_string(),
          scope: "scope_toolset-builtin-exa-web-search".to_string(),
          enabled: false,
          updated_by: "admin123".to_string(),
          created_at: chrono::Utc::now(),
          updated_at: chrono::Utc::now(),
        })
      });
  }
  // When succeeds is false, list_types returns empty vec and handler returns early with Not Found
  // No need to mock set_app_toolset_enabled in that case

  let app = test_router(mock_tool_service)?;

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/toolset_types/scope_toolset-builtin-exa-web-search/app-config")
        .header(KEY_HEADER_BODHIAPP_USER_ID, "admin123")
        .with_user_auth("admin-token", &ResourceRole::Admin.to_string())
        .body(Body::empty())?,
    )
    .await?;

  assert_eq!(expected_status, response.status());
  Ok(())
}
