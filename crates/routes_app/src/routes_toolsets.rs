use crate::toolsets_dto::{
  ApiKeyUpdateDto, AppToolsetConfigResponse, CreateToolsetRequest, ExecuteToolsetRequest,
  ListToolsetTypesResponse, ListToolsetsResponse, ToolsetResponse, ToolsetTypeResponse,
  UpdateToolsetRequest,
};
use crate::{ENDPOINT_TOOLSETS, ENDPOINT_TOOLSET_TYPES};
use auth_middleware::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
  KEY_HEADER_BODHIAPP_TOOL_SCOPES, KEY_HEADER_BODHIAPP_USER_ID,
};
use axum::{
  extract::{Path, State},
  http::{HeaderMap, StatusCode},
  routing::{delete, get, post, put},
  Json, Router,
};
use objs::API_TAG_TOOLSETS;
use objs::{ApiError, Toolset, ToolsetExecutionResponse, ToolsetScope};
use server_core::RouterState;
use services::db::ApiKeyUpdate;
use services::ToolsetError;
use std::collections::HashSet;
use std::sync::Arc;
use validator::Validate;

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_user_id_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
  headers
    .get(KEY_HEADER_BODHIAPP_USER_ID)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
    .ok_or_else(|| {
      objs::BadRequestError::new("User ID not found in request headers".to_string()).into()
    })
}

fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
  headers
    .get(KEY_HEADER_BODHIAPP_TOKEN)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
    .ok_or_else(|| {
      objs::BadRequestError::new("Access token not found in request headers".to_string()).into()
    })
}

fn is_oauth_auth(headers: &HeaderMap) -> bool {
  !headers.contains_key(KEY_HEADER_BODHIAPP_ROLE)
    && headers
      .get(KEY_HEADER_BODHIAPP_SCOPE)
      .and_then(|v| v.to_str().ok())
      .map(|s| s.starts_with("scope_user_"))
      .unwrap_or(false)
}

fn extract_allowed_toolset_scopes(headers: &HeaderMap) -> HashSet<String> {
  let toolset_scopes_header = headers
    .get(KEY_HEADER_BODHIAPP_TOOL_SCOPES)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

  let allowed_scopes = ToolsetScope::from_scope_string(toolset_scopes_header);
  allowed_scopes.iter().map(|s| s.to_string()).collect()
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes_toolsets(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    // Toolset CRUD
    .route("/toolsets", get(list_toolsets_handler))
    .route("/toolsets", post(create_toolset_handler))
    .route("/toolsets/{id}", get(get_toolset_handler))
    .route("/toolsets/{id}", put(update_toolset_handler))
    .route("/toolsets/{id}", delete(delete_toolset_handler))
    // Execute (middleware at routes_all level)
    .route(
      "/toolsets/{id}/execute/{method}",
      post(execute_toolset_handler),
    )
    // Type listing and admin (separate namespace avoids {id} collision)
    .route("/toolset_types", get(list_toolset_types_handler))
    .route(
      "/toolset_types/{scope}/app-config",
      put(enable_type_handler),
    )
    .route(
      "/toolset_types/{scope}/app-config",
      delete(disable_type_handler),
    )
    .with_state(state)
}

// ============================================================================
// Toolset CRUD Handlers
// ============================================================================

/// List all toolsets for the authenticated user
///
/// For OAuth tokens, filters toolsets by scope_toolset-* scopes in the token.
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSETS,
  tag = API_TAG_TOOLSETS,
  operation_id = "listToolsets",
  responses(
    (status = 200, description = "List of user's toolsets", body = ListToolsetsResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_toolsets_handler(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsetsResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  let toolsets = tool_service.list(&user_id).await?;

  // OAuth filtering: hide toolsets of types not in scopes
  let filtered_toolsets = if is_oauth_auth(&headers) {
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    toolsets
      .into_iter()
      .filter(|toolset| allowed_scopes.contains(&toolset.scope))
      .collect()
  } else {
    toolsets
  };

  // Enrich each toolset with type information
  let mut responses = Vec::new();
  for toolset in filtered_toolsets {
    responses.push(toolset_to_response(toolset, &tool_service).await?);
  }

  // Fetch toolset_types based on auth type
  let toolset_types = if is_oauth_auth(&headers) {
    // For OAuth: fetch only configs for scopes in the token (efficient database query)
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    let scopes_vec: Vec<String> = allowed_scopes.into_iter().collect();
    tool_service
      .list_app_toolset_configs_by_scopes(&scopes_vec)
      .await?
  } else {
    // For session: return all toolset configs (session users have access to all scopes)
    tool_service.list_app_toolset_configs().await?
  };

  Ok(Json(ListToolsetsResponse {
    toolsets: responses,
    toolset_types,
  }))
}

/// Create a new toolset
#[utoipa::path(
  post,
  path = ENDPOINT_TOOLSETS,
  tag = API_TAG_TOOLSETS,
  operation_id = "createToolset",
  request_body = CreateToolsetRequest,
  responses(
    (status = 201, description = "Toolset created", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn create_toolset_handler(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
  Json(request): Json<CreateToolsetRequest>,
) -> Result<(StatusCode, Json<ToolsetResponse>), ApiError> {
  request
    .validate()
    .map_err(|e| objs::BadRequestError::new(format!("Validation error: {}", e)))?;

  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  let toolset = tool_service
    .create(
      &user_id,
      &request.scope_uuid,
      &request.name,
      request.description,
      request.enabled,
      request.api_key,
    )
    .await?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific toolset by ID
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "getToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  responses(
    (status = 200, description = "Toolset", body = ToolsetResponse),
    (status = 404, description = "Toolset not found or not owned"),
  ),
  security(("bearer" = []))
)]
pub async fn get_toolset_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  headers: HeaderMap,
) -> Result<Json<ToolsetResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  let toolset = tool_service
    .get(&user_id, &id)
    .await?
    .ok_or_else(|| objs::EntityError::NotFound("Toolset".to_string()))?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok(Json(response))
}

/// Update a toolset (full PUT semantics)
#[utoipa::path(
  put,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "updateToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  request_body = UpdateToolsetRequest,
  responses(
    (status = 200, description = "Toolset updated", body = ToolsetResponse),
    (status = 400, description = "Validation error"),
    (status = 404, description = "Toolset not found or not owned"),
    (status = 409, description = "Name already exists"),
  ),
  security(("bearer" = []))
)]
pub async fn update_toolset_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  headers: HeaderMap,
  Json(request): Json<UpdateToolsetRequest>,
) -> Result<Json<ToolsetResponse>, ApiError> {
  request
    .validate()
    .map_err(|e| objs::BadRequestError::new(format!("Validation error: {}", e)))?;

  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  let api_key_update = match request.api_key {
    ApiKeyUpdateDto::Keep => ApiKeyUpdate::Keep,
    ApiKeyUpdateDto::Set(value) => ApiKeyUpdate::Set(value),
  };

  let toolset = tool_service
    .update(
      &user_id,
      &id,
      &request.name,
      request.description,
      request.enabled,
      api_key_update,
    )
    .await?;

  let response = toolset_to_response(toolset, &tool_service).await?;
  Ok(Json(response))
}

/// Delete a toolset
#[utoipa::path(
  delete,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}",
  tag = API_TAG_TOOLSETS,
  operation_id = "deleteToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID")
  ),
  responses(
    (status = 204, description = "Toolset deleted"),
    (status = 404, description = "Toolset not found or not owned"),
  ),
  security(("bearer" = []))
)]
pub async fn delete_toolset_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  tool_service.delete(&user_id, &id).await?;

  Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Execute Handler
// ============================================================================

/// Execute a tool method on a toolset
#[utoipa::path(
  post,
  path = ENDPOINT_TOOLSETS.to_owned() + "/{id}/execute/{method}",
  tag = API_TAG_TOOLSETS,
  operation_id = "executeToolset",
  params(
    ("id" = String, Path, description = "Toolset instance UUID"),
    ("method" = String, Path, description = "Tool method name")
  ),
  request_body = ExecuteToolsetRequest,
  responses(
    (status = 200, description = "Tool execution result", body = ToolsetExecutionResponse),
    (status = 400, description = "Validation error or toolset not configured"),
    (status = 404, description = "Toolset or method not found"),
  ),
  security(("bearer" = []))
)]
pub async fn execute_toolset_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path((id, method)): Path<(String, String)>,
  headers: HeaderMap,
  Json(request): Json<ExecuteToolsetRequest>,
) -> Result<Json<ToolsetExecutionResponse>, ApiError> {
  let user_id = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  let response = tool_service
    .execute(&user_id, &id, &method, request.into())
    .await?;

  Ok(Json(response))
}

// ============================================================================
// Toolset Type Handlers (Admin)
// ============================================================================

/// List all available toolset types with their tools
///
/// For OAuth tokens, filters types by scope_toolset-* scopes in the token.
#[utoipa::path(
  get,
  path = ENDPOINT_TOOLSET_TYPES,
  tag = API_TAG_TOOLSETS,
  operation_id = "listToolsetTypes",
  responses(
    (status = 200, description = "List of all toolset types", body = ListToolsetTypesResponse),
  ),
  security(("bearer" = []))
)]
pub async fn list_toolset_types_handler(
  State(state): State<Arc<dyn RouterState>>,
  headers: HeaderMap,
) -> Result<Json<ListToolsetTypesResponse>, ApiError> {
  let tool_service = state.app_service().tool_service();

  let toolsets = tool_service.list_all_toolsets().await?;

  // OAuth filtering: hide types not in scopes
  let filtered_toolsets = if is_oauth_auth(&headers) {
    let allowed_scopes = extract_allowed_toolset_scopes(&headers);
    toolsets
      .into_iter()
      .filter(|t| allowed_scopes.contains(&t.scope))
      .collect()
  } else {
    toolsets
  };

  let types: Vec<ToolsetTypeResponse> = filtered_toolsets
    .into_iter()
    .map(|t| ToolsetTypeResponse {
      scope_uuid: t.scope_uuid,
      scope: t.scope,
      name: t.name,
      description: t.description,
      app_enabled: t.app_enabled,
      tools: t.tools,
    })
    .collect();

  Ok(Json(ListToolsetTypesResponse { types }))
}

/// Enable a toolset type at app level (admin only - enforced by auth middleware)
#[utoipa::path(
  put,
  path = ENDPOINT_TOOLSET_TYPES.to_owned() + "/{type_id}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "enableToolsetType",
  params(
    ("type_id" = String, Path, description = "Toolset type identifier")
  ),
  responses(
    (status = 200, description = "Toolset type enabled", body = AppToolsetConfigResponse),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn enable_type_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(scope): Path<String>,
  headers: HeaderMap,
) -> Result<Json<AppToolsetConfigResponse>, ApiError> {
  let admin_token = extract_token_from_headers(&headers)?;
  let updated_by = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  // Find the toolset definition to get scope_uuid
  let toolset_def = tool_service
    .list_types()
    .into_iter()
    .find(|def| def.scope == scope)
    .ok_or_else(|| services::ToolsetError::ToolsetNotFound(scope.clone()))?;

  let config = tool_service
    .set_app_toolset_enabled(
      &admin_token,
      &scope,
      &toolset_def.scope_uuid,
      true,
      &updated_by,
    )
    .await?;

  Ok(Json(AppToolsetConfigResponse { config }))
}

/// Disable a toolset type at app level (admin only - enforced by auth middleware)
#[utoipa::path(
  delete,
  path = ENDPOINT_TOOLSET_TYPES.to_owned() + "/{type_id}/app-config",
  tag = API_TAG_TOOLSETS,
  operation_id = "disableToolsetType",
  params(
    ("type_id" = String, Path, description = "Toolset type identifier")
  ),
  responses(
    (status = 200, description = "Toolset type disabled", body = AppToolsetConfigResponse),
    (status = 404, description = "Toolset type not found"),
  ),
  security(("bearer" = []))
)]
pub async fn disable_type_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(scope): Path<String>,
  headers: HeaderMap,
) -> Result<Json<AppToolsetConfigResponse>, ApiError> {
  let admin_token = extract_token_from_headers(&headers)?;
  let updated_by = extract_user_id_from_headers(&headers)?;
  let tool_service = state.app_service().tool_service();

  // Find the toolset definition to get scope_uuid
  let toolset_def = tool_service
    .list_types()
    .into_iter()
    .find(|def| def.scope == scope)
    .ok_or_else(|| services::ToolsetError::ToolsetNotFound(scope.clone()))?;

  let config = tool_service
    .set_app_toolset_enabled(
      &admin_token,
      &scope,
      &toolset_def.scope_uuid,
      false,
      &updated_by,
    )
    .await?;

  Ok(Json(AppToolsetConfigResponse { config }))
}

// ============================================================================
// Conversion Helpers
// ============================================================================

async fn toolset_to_response(
  toolset: Toolset,
  tool_service: &Arc<dyn services::ToolService>,
) -> Result<ToolsetResponse, ApiError> {
  // Get type information for enrichment
  let type_def = tool_service
    .get_type(&toolset.scope_uuid)
    .ok_or_else(|| ToolsetError::InvalidToolsetType(toolset.scope_uuid.clone()))?;

  Ok(ToolsetResponse {
    id: toolset.id,
    name: toolset.name,
    scope_uuid: toolset.scope_uuid,
    scope: toolset.scope,
    description: toolset.description,
    enabled: toolset.enabled,
    has_api_key: toolset.has_api_key,
    tools: type_def.tools,
    created_at: toolset.created_at,
    updated_at: toolset.updated_at,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use chrono::Utc;
  use objs::{
    AppToolsetConfig, ResourceRole, ToolDefinition, ToolsetExecutionResponse, ToolsetWithTools,
  };
  use rstest::{fixture, rstest};
  use server_core::{DefaultRouterState, MockSharedContext};
  use services::{test_utils::AppServiceStubBuilder, MockToolService};
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

  fn test_router(mock_tool_service: MockToolService) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .with_tool_service(Arc::new(mock_tool_service))
      .build()
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    routes_toolsets(state)
  }

  // ============================================================================
  // List Tests
  // ============================================================================

  #[rstest]
  #[case::session_returns_all(true, false, 1)]
  #[case::oauth_filters_by_scope(false, true, 0)]
  #[case::empty_list(true, false, 0)]
  #[tokio::test]
  async fn test_list_toolsets(
    test_instance: Toolset,
    #[case] is_session: bool,
    #[case] is_oauth_filtered: bool,
    #[case] expected_count: usize,
  ) {
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

    let app = test_router(mock_tool_service);

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
      .oneshot(request_builder.body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(StatusCode::OK, response.status());
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_toolsets_session_returns_all_toolset_types(test_instance: Toolset) {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("GET")
          .uri("/toolsets")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let list_response: crate::toolsets_dto::ListToolsetsResponse =
      serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(1, list_response.toolset_types.len());
    assert_eq!(
      "scope_toolset-builtin-exa-web-search",
      list_response.toolset_types[0].scope
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_toolsets_oauth_returns_scoped_toolset_types(test_instance: Toolset) {
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

    let app = test_router(mock_tool_service);

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
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let list_response: crate::toolsets_dto::ListToolsetsResponse =
      serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(1, list_response.toolset_types.len());
    assert_eq!(
      "scope_toolset-builtin-exa-web-search",
      list_response.toolset_types[0].scope
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_toolsets_oauth_empty_scopes_returns_empty_toolset_types() {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("GET")
          .uri("/toolsets")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_SCOPE, "scope_user_user")
          .header(KEY_HEADER_BODHIAPP_TOOL_SCOPES, "")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let list_response: crate::toolsets_dto::ListToolsetsResponse =
      serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(0, list_response.toolset_types.len());
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
  async fn test_create_toolset(
    test_instance: Toolset,
    #[case] name: &str,
    #[case] expected_status: StatusCode,
    #[case] _error_code: Option<&str>,
  ) {
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

    let app = test_router(mock_tool_service);

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
          .body(Body::from(serde_json::to_string(&request_body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  // ============================================================================
  // Get Tests
  // ============================================================================

  #[rstest]
  #[case::success(true, StatusCode::OK)]
  #[case::not_found(false, StatusCode::NOT_FOUND)]
  #[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_get_toolset(
    test_instance: Toolset,
    #[case] returns_instance: bool,
    #[case] expected_status: StatusCode,
  ) {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("GET")
          .uri(format!("/toolsets/{}", test_instance.id))
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
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
  async fn test_update_toolset(
    test_instance: Toolset,
    #[case] name: &str,
    #[case] api_key: ApiKeyUpdateDto,
    #[case] expected_status: StatusCode,
    #[case] _error_code: Option<&str>,
  ) {
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

    let app = test_router(mock_tool_service);

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
          .body(Body::from(serde_json::to_string(&request_body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  #[rstest]
  #[case::not_found(false, StatusCode::NOT_FOUND)]
  #[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_update_toolset_not_found(
    test_instance: Toolset,
    #[case] _returns_instance: bool,
    #[case] expected_status: StatusCode,
  ) {
    let mut mock_tool_service = MockToolService::new();

    mock_tool_service
      .expect_update()
      .times(1)
      .returning(|_, _, _, _, _, _| {
        Err(services::ToolsetError::ToolsetNotFound(
          "Toolset".to_string(),
        ))
      });

    let app = test_router(mock_tool_service);

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
          .body(Body::from(serde_json::to_string(&request_body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  // ============================================================================
  // Delete Tests
  // ============================================================================

  #[rstest]
  #[case::success(true, StatusCode::NO_CONTENT)]
  #[case::not_found(false, StatusCode::NOT_FOUND)]
  #[case::not_owned_returns_404(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_delete_toolset(
    test_instance: Toolset,
    #[case] succeeds: bool,
    #[case] expected_status: StatusCode,
  ) {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("DELETE")
          .uri(format!("/toolsets/{}", test_instance.id))
          .header(KEY_HEADER_BODHIAPP_USER_ID, "user123")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::User.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  // ============================================================================
  // Execute Tests
  // ============================================================================

  #[rstest]
  #[case::success(true, StatusCode::OK)]
  #[case::method_not_found(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_execute_toolset(
    test_instance: Toolset,
    #[case] succeeds: bool,
    #[case] expected_status: StatusCode,
  ) {
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

    let app = test_router(mock_tool_service);

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
          .body(Body::from(serde_json::to_string(&request_body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  // ============================================================================
  // List Toolset Types Tests
  // ============================================================================

  #[rstest]
  #[case::session_returns_all(true, 1)]
  #[case::oauth_filters(false, 0)]
  #[tokio::test]
  async fn test_list_toolset_types(#[case] is_session: bool, #[case] expected_count: usize) {
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

    let app = test_router(mock_tool_service);

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
      .oneshot(request_builder.body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(StatusCode::OK, response.status());
  }

  // ============================================================================
  // Enable/Disable Type Tests
  // ============================================================================

  #[rstest]
  #[case::success(true, StatusCode::OK)]
  #[case::type_not_found(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_enable_type(#[case] succeeds: bool, #[case] expected_status: StatusCode) {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("PUT")
          .uri("/toolset_types/scope_toolset-builtin-exa-web-search/app-config")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "admin123")
          .header(KEY_HEADER_BODHIAPP_TOKEN, "admin-token")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::Admin.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }

  #[rstest]
  #[case::success(true, StatusCode::OK)]
  #[case::type_not_found(false, StatusCode::NOT_FOUND)]
  #[tokio::test]
  async fn test_disable_type(#[case] succeeds: bool, #[case] expected_status: StatusCode) {
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

    let app = test_router(mock_tool_service);

    let response = app
      .oneshot(
        Request::builder()
          .method("DELETE")
          .uri("/toolset_types/scope_toolset-builtin-exa-web-search/app-config")
          .header(KEY_HEADER_BODHIAPP_USER_ID, "admin123")
          .header(KEY_HEADER_BODHIAPP_TOKEN, "admin-token")
          .header(KEY_HEADER_BODHIAPP_ROLE, ResourceRole::Admin.to_string())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(expected_status, response.status());
  }
}
