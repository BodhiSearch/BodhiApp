use crate::{
  api_models_dto::{
    ApiFormatsResponse, ApiModelResponse, CreateApiModelRequest, FetchModelsRequest,
    FetchModelsResponse, PaginatedApiModelResponse, TestPromptRequest, TestPromptResponse,
    UpdateApiModelRequest,
  },
  PaginationSortParams, ENDPOINT_API_MODELS, ENDPOINT_API_MODELS_API_FORMATS,
  ENDPOINT_API_MODELS_FETCH_MODELS, ENDPOINT_API_MODELS_TEST,
};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiAlias, ApiError, ApiFormat, ObjValidationError, OpenAIApiError, API_TAG_API_MODELS};
use server_core::RouterState;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// List all API model configurations
#[utoipa::path(
    get,
    path = ENDPOINT_API_MODELS,
    tag = API_TAG_API_MODELS,
    operation_id = "listApiModels",
    summary = "List API Model Configurations",
    description = "Retrieves paginated list of all configured API model aliases including external API formats like OpenAI, etc. API keys are masked in list view for security.",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "API model configurations retrieved successfully", body = PaginatedApiModelResponse,
         example = json!({
             "data": [{
                 "id": "openai-gpt4",
                 "api_format": "openai",
                 "base_url": "https://api.openai.com/v1",
                 "api_key": "sk-****"
             }],
             "total": 1,
             "page": 1,
             "page_size": 10
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn list_api_models_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedApiModelResponse>, ApiError> {
  let db_service = state.app_service().db_service();

  // Get all API model aliases
  let aliases = db_service.list_api_model_aliases().await?;

  // Apply pagination
  let total = aliases.len();
  let start = (params.page - 1) * params.page_size;
  let end = std::cmp::min(start + params.page_size, total);

  let page_data: Vec<ApiModelResponse> = aliases[start..end]
    .iter()
    .map(|alias| {
      // For list view, always show masked API key indicator for security
      // (checking individual key existence would be inefficient here)
      ApiModelResponse::from_alias(alias.clone(), true)
    })
    .collect();

  Ok(Json(PaginatedApiModelResponse {
    data: page_data,
    total,
    page: params.page,
    page_size: params.page_size,
  }))
}

/// Get a specific API model configuration
#[utoipa::path(
    get,
    path = ENDPOINT_API_MODELS.to_owned() + "/{id}",
    tag = API_TAG_API_MODELS,
    operation_id = "getApiModel",
    summary = "Get API Model Configuration",
    description = "Retrieves detailed configuration for a specific API model alias by ID. API keys are masked for security unless explicitly requested.",
    params(
        ("id" = String, Path, description = "Unique identifier for the API model alias", example = "openai-gpt4")
    ),
    responses(
        (status = 200, description = "API model configuration retrieved successfully", body = ApiModelResponse,
         example = json!({
             "id": "openai-gpt4",
             "api_format": "openai",
             "base_url": "https://api.openai.com/v1",
             "api_key": "sk-****",
             "model": "gpt-4"
         })),
        (status = 404, description = "API model with specified ID not found", body = objs::OpenAIApiError,
         example = json!({
             "error": {
                 "message": "API model 'invalid-model' not found",
                 "type": "not_found_error",
                 "code": "entity_not_found"
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn get_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<ApiModelResponse>, ApiError> {
  let db_service = state.app_service().db_service();

  let api_alias = db_service.get_api_model_alias(&id).await?.ok_or_else(|| {
    ApiError::from(objs::EntityError::NotFound(format!(
      "API model '{}' not found",
      id
    )))
  })?;

  // Check if API key exists for this model
  let has_api_key = db_service.get_api_key_for_alias(&id).await?.is_some();

  Ok(Json(ApiModelResponse::from_alias(api_alias, has_api_key)))
}

/// Create a new API model configuration
#[utoipa::path(
    post,
    path = ENDPOINT_API_MODELS,
    tag = API_TAG_API_MODELS,
    operation_id = "createApiModel",
    request_body = CreateApiModelRequest,
    responses(
        (status = 201, description = "API model created", body = ApiModelResponse),
        (status = 409, description = "Alias already exists", body = objs::OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn create_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiModelRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiModelResponse>), ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  // Additional validation: forward_all_with_prefix requires a non-empty prefix
  if payload.forward_all_with_prefix && payload.prefix.as_ref().map_or(true, |p| p.is_empty()) {
    return Err(ApiError::from(ObjValidationError::ForwardAllRequiresPrefix));
  }

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Generate a unique UUID for the API model
  let id = Uuid::new_v4().to_string();

  // Create the API model alias
  let now = time_service.utc_now();
  let api_alias = ApiAlias::new(
    id,
    payload.api_format,
    payload.base_url.trim_end_matches('/').to_string(),
    payload.models,
    payload.prefix,
    payload.forward_all_with_prefix,
    now,
  );

  // Convert ApiKey to Option<String> for DB
  let api_key_option = payload.api_key.as_option().map(|s| s.to_string());

  db_service
    .create_api_model_alias(&api_alias, api_key_option)
    .await?;

  let response = ApiModelResponse::from_alias(api_alias, payload.api_key.is_some());

  Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing API model configuration
#[utoipa::path(
    put,
    path = ENDPOINT_API_MODELS.to_owned() + "/{id}",
    tag = API_TAG_API_MODELS,
    operation_id = "updateApiModel",
    params(
        ("id" = String, Path, description = "API model ID")
    ),
    request_body = UpdateApiModelRequest,
    responses(
        (status = 200, description = "API model updated", body = ApiModelResponse),
        (status = 404, description = "API model not found", body = OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn update_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiModelRequest>, ApiError>,
) -> Result<Json<ApiModelResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  // Additional validation: forward_all_with_prefix requires a non-empty prefix
  if payload.forward_all_with_prefix && payload.prefix.as_ref().map_or(true, |p| p.is_empty()) {
    return Err(ApiError::from(ObjValidationError::ForwardAllRequiresPrefix));
  }

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Get existing API model
  let mut api_alias = db_service.get_api_model_alias(&id).await?.ok_or_else(|| {
    ApiError::from(objs::EntityError::NotFound(format!(
      "API model '{}' not found",
      id
    )))
  })?;

  // Update all fields (api_key is handled separately for security)
  api_alias.api_format = payload.api_format;
  api_alias.base_url = payload.base_url.trim_end_matches('/').to_string();
  api_alias.models = payload.models;
  api_alias.prefix = if payload.prefix.as_ref().is_some_and(|p| p.is_empty()) {
    None
  } else {
    payload.prefix
  };
  api_alias.forward_all_with_prefix = payload.forward_all_with_prefix;

  api_alias.updated_at = time_service.utc_now();

  // Convert DTO enum to service enum
  let api_key_update = services::db::ApiKeyUpdate::from(payload.api_key.clone());
  db_service
    .update_api_model_alias(&id, &api_alias, api_key_update)
    .await?;

  // Check if API key exists after update
  let has_api_key = db_service.get_api_key_for_alias(&id).await?.is_some();

  // Return response with masked API key
  Ok(Json(ApiModelResponse::from_alias(api_alias, has_api_key)))
}

/// Delete an API model configuration
#[utoipa::path(
    delete,
    path = ENDPOINT_API_MODELS.to_owned() + "/{id}",
    tag = API_TAG_API_MODELS,
    operation_id = "deleteApiModel",
    params(
        ("id" = String, Path, description = "API model ID")
    ),
    responses(
        (status = 204, description = "API model deleted"),
        (status = 404, description = "API model not found", body = OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn delete_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let db_service = state.app_service().db_service();

  // Check if API model exists
  if db_service.get_api_model_alias(&id).await?.is_none() {
    return Err(ApiError::from(objs::EntityError::NotFound(format!(
      "API model '{}' not found",
      id
    ))));
  }

  // Delete the API model
  db_service.delete_api_model_alias(&id).await?;

  Ok(StatusCode::NO_CONTENT)
}

/// Test API connectivity with a prompt
#[utoipa::path(
    post,
    path = ENDPOINT_API_MODELS_TEST.to_owned(),
    tag = API_TAG_API_MODELS,
    operation_id = "testApiModel",
    request_body = TestPromptRequest,
    responses(
        (status = 200, description = "Test result", body = TestPromptResponse),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn test_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<TestPromptRequest>, ApiError>,
) -> Result<Json<TestPromptResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  let ai_api_service = state.app_service().ai_api_service();
  let db_service = state.app_service().db_service();

  // Resolve credentials using TestCreds enum
  let result = match &payload.creds {
    crate::api_models_dto::TestCreds::ApiKey(api_key) => {
      // Use provided API key directly (or None for no authentication)
      ai_api_service
        .test_prompt(
          api_key.as_option().map(|s| s.to_string()),
          payload.base_url.trim_end_matches('/'),
          &payload.model,
          &payload.prompt,
        )
        .await
    }
    crate::api_models_dto::TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      // Get stored key (may be None if no key configured)
      let stored_key = db_service.get_api_key_for_alias(id).await?;

      ai_api_service
        .test_prompt(
          stored_key,
          api_model.base_url.trim_end_matches('/'),
          &payload.model,
          &payload.prompt,
        )
        .await
    }
  };

  // Return success/failure response based on result
  match result {
    Ok(response) => Ok(Json(TestPromptResponse::success(response))),
    Err(err) => Ok(Json(TestPromptResponse::failure(err.to_string()))),
  }
}

/// Fetch available models from the API
#[utoipa::path(
    post,
    path = ENDPOINT_API_MODELS_FETCH_MODELS.to_owned(),
    tag = API_TAG_API_MODELS,
    operation_id = "fetchApiModels",
    request_body = FetchModelsRequest,
    responses(
        (status = 200, description = "Available models", body = FetchModelsResponse),
        (status = 400, description = "Invalid request", body = OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn fetch_models_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<FetchModelsRequest>, ApiError>,
) -> Result<Json<FetchModelsResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  let ai_api_service = state.app_service().ai_api_service();
  let db_service = state.app_service().db_service();

  // Resolve credentials using TestCreds enum
  let models = match &payload.creds {
    crate::api_models_dto::TestCreds::ApiKey(api_key) => {
      // Use provided API key directly (or None for no authentication)
      ai_api_service
        .fetch_models(
          api_key.as_option().map(|s| s.to_string()),
          payload.base_url.trim_end_matches('/'),
        )
        .await?
    }
    crate::api_models_dto::TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      // Get stored key (may be None if no key configured)
      let stored_key = db_service.get_api_key_for_alias(id).await?;

      ai_api_service
        .fetch_models(stored_key, api_model.base_url.trim_end_matches('/'))
        .await?
    }
  };

  Ok(Json(FetchModelsResponse { models }))
}

/// Get available API formats
#[utoipa::path(
    get,
    path = ENDPOINT_API_MODELS_API_FORMATS.to_owned(),
    tag = API_TAG_API_MODELS,
    operation_id = "getApiFormats",
    summary = "Get Available API Formats",
    description = "Retrieves list of supported API formats/protocols (e.g., OpenAI).",
    responses(
        (status = 200, description = "API formats retrieved successfully", body = ApiFormatsResponse,
         example = json!({
             "data": ["openai"]
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn get_api_formats_handler() -> Result<Json<ApiFormatsResponse>, ApiError> {
  Ok(Json(ApiFormatsResponse {
    data: vec![ApiFormat::OpenAI],
  }))
}

#[cfg(test)]
mod tests {
  use super::{
    create_api_model_handler, delete_api_model_handler, fetch_models_handler,
    get_api_model_handler, list_api_models_handler, test_api_model_handler,
    update_api_model_handler,
  };
  use crate::{
    api_models_dto::{
      ApiModelResponse, CreateApiModelRequest, FetchModelsRequest, TestPromptRequest,
      UpdateApiModelRequest,
    },
    PaginatedApiModelResponse, ENDPOINT_API_MODELS,
  };
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{delete, get, post, put},
    Router,
  };
  use chrono::{DateTime, Utc};
  use objs::{
    test_utils::setup_l10n, ApiAliasBuilder, ApiFormat::OpenAI, FluentLocalizationService,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{
    db::DbService,
    test_utils::{seed_test_api_models, test_db_service, AppServiceStubBuilder, TestDbService},
  };
  use std::sync::Arc;
  use tower::ServiceExt;
  use uuid::Uuid;
  use validator::Validate;

  /// Create expected ApiModelResponse for testing
  fn create_expected_response(
    id: &str,
    api_format: &str,
    base_url: &str,
    api_key_masked: Option<&str>,
    models: Vec<String>,
    prefix: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> ApiModelResponse {
    use std::str::FromStr;
    ApiModelResponse {
      id: id.to_string(),
      api_format: objs::ApiFormat::from_str(api_format).unwrap(),
      base_url: base_url.to_string(),
      api_key_masked: api_key_masked.map(|s| s.to_string()),
      models,
      prefix,
      forward_all_with_prefix: false,
      created_at,
      updated_at,
    }
  }

  /// Create expected ApiModelResponse for list view (masked API key)
  fn create_expected_list_response(
    id: &str,
    models: Vec<String>,
    created_at: DateTime<Utc>,
  ) -> ApiModelResponse {
    create_expected_response(
      id,
      "openai",
      "https://api.openai.com/v1",
      Some("***"), // Masked in list view
      models,
      None, // No prefix in original seed data
      created_at,
      created_at,
    )
  }

  /// Create expected ApiModelResponse for list view with prefix support
  fn create_expected_list_response_with_prefix(
    id: &str,
    models: Vec<String>,
    prefix: Option<String>,
    created_at: DateTime<Utc>,
  ) -> ApiModelResponse {
    create_expected_response(
      id,
      "openai",
      "https://api.openai.com/v1",
      Some("***"), // Masked in list view
      models,
      prefix,
      created_at,
      created_at,
    )
  }

  fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
    Router::new()
      .route(ENDPOINT_API_MODELS, get(list_api_models_handler))
      .route(ENDPOINT_API_MODELS, post(create_api_model_handler))
      .route(
        &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
        get(get_api_model_handler),
      )
      .route(
        &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
        put(update_api_model_handler),
      )
      .route(
        &format!("{}/{}", ENDPOINT_API_MODELS, "{id}"),
        delete(delete_api_model_handler),
      )
      .route(
        &format!("{}/test", ENDPOINT_API_MODELS),
        post(test_api_model_handler),
      )
      .route(
        &format!("{}/fetch-models", ENDPOINT_API_MODELS),
        post(fetch_models_handler),
      )
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_api_models_handler(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let base_time = db_service.now();

    // Seed the database with test API model aliases
    let _expected_aliases = seed_test_api_models(&db_service, base_time).await?;

    // Create app service with the seeded database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Make request to list API models
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::get(ENDPOINT_API_MODELS)
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<PaginatedApiModelResponse>()
      .await?;

    // Verify response structure
    assert_eq!(response.total, 7);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 30);
    assert_eq!(response.data.len(), 7);

    // Create expected response array sorted by created_at DESC (newest first)
    let expected_data = vec![
      create_expected_list_response("openai-gpt4", vec!["gpt-4".to_string()], base_time),
      create_expected_list_response(
        "openai-gpt35-turbo",
        vec!["gpt-3.5-turbo".to_string()],
        base_time - chrono::Duration::seconds(10),
      ),
      create_expected_list_response(
        "openai-gpt4-turbo",
        vec!["gpt-4-turbo".to_string()],
        base_time - chrono::Duration::seconds(20),
      ),
      create_expected_list_response(
        "openai-gpt4-vision",
        vec!["gpt-4-vision-preview".to_string()],
        base_time - chrono::Duration::seconds(30),
      ),
      create_expected_list_response(
        "openai-multi-model",
        vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        base_time - chrono::Duration::seconds(40),
      ),
      create_expected_list_response_with_prefix(
        "azure-openai",
        vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        Some("azure/".to_string()),
        base_time - chrono::Duration::seconds(50),
      ),
      create_expected_list_response_with_prefix(
        "custom-alias",
        vec!["custom-model-1".to_string()],
        Some("my.custom_".to_string()),
        base_time - chrono::Duration::seconds(60),
      ),
    ];

    let expected_response = PaginatedApiModelResponse {
      data: expected_data,
      total: 7,
      page: 1,
      page_size: 30,
    };

    // Use pretty_assertions for comprehensive comparison
    assert_eq!(expected_response, response);

    Ok(())
  }

  #[rstest]
  #[case::no_trailing_slash("https://api.openai.com/v1", "https://api.openai.com/v1")]
  #[case::single_trailing_slash("https://api.openai.com/v1/", "https://api.openai.com/v1")]
  #[case::multiple_trailing_slashes("https://api.openai.com/v1///", "https://api.openai.com/v1")]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_success(
    #[case] input_url: &str,
    #[case] expected_url: &str,
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let create_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: input_url.to_string(),
      api_key: crate::api_models_dto::ApiKey::some("sk-test123456789".to_string()).unwrap(),
      models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
    };

    // Make POST request to create API model
    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify response body
    let api_response = response.json::<ApiModelResponse>().await?;

    // Verify the response structure (note: ID is now auto-generated UUID)
    assert_eq!(api_response.api_format, objs::ApiFormat::OpenAI);
    assert_eq!(api_response.base_url, expected_url);
    assert_eq!(api_response.api_key_masked, Some("***".to_string()));
    assert_eq!(
      api_response.models,
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]
    );
    assert_eq!(api_response.prefix, None);

    // Verify that ID is a valid UUID
    assert!(Uuid::parse_str(&api_response.id).is_ok());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_generates_uuid(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let base_time = db_service.now();

    // Seed database with existing API model
    seed_test_api_models(&db_service, base_time).await?;

    // Create app service with seeded database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let create_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::some("sk-test123456789".to_string()).unwrap(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
    };

    // Make POST request to create API model (should succeed since UUIDs are unique)
    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status is 201 Created (no duplicate ID issue with UUIDs)
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify response structure
    let api_response = response.json::<ApiModelResponse>().await?;
    assert!(Uuid::parse_str(&api_response.id).is_ok());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_validation_error_empty_api_key(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Test with raw JSON to trigger deserialization error for empty API key
    let json_request = json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "",  // Invalid: empty api_key
      "models": ["gpt-4"]
    });

    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(json_request)?)
      .await?;

    // Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error response contains validation error for API key
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("API key must not be empty"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_validation_error_invalid_url(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let create_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "not-a-valid-url".to_string(), // Invalid: not a valid URL
      api_key: crate::api_models_dto::ApiKey::some("sk-test123456789".to_string()).unwrap(),
      models: vec!["gpt-4".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
    };

    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error response contains validation error for URL
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Base URL must be a valid URL"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_validation_error_empty_models(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let create_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::some("sk-test123456789".to_string()).unwrap(),
      models: vec![], // Invalid: empty models array
      prefix: None,
      forward_all_with_prefix: false,
    };

    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error response contains validation error for models length
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("Models list must not be empty"));

    Ok(())
  }

  #[rstest]
  #[case::no_trailing_slash("https://api.openai.com/v2", "https://api.openai.com/v2")]
  #[case::single_trailing_slash("https://api.openai.com/v2/", "https://api.openai.com/v2")]
  #[case::multiple_trailing_slashes("https://api.openai.com/v2///", "https://api.openai.com/v2")]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_handler_success(
    #[case] input_url: &str,
    #[case] expected_url: &str,
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let base_time = db_service.now();

    // Seed database with existing API model
    seed_test_api_models(&db_service, base_time).await?;

    // Create app service with seeded database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let update_request = UpdateApiModelRequest {
      api_format: OpenAI,
      base_url: input_url.to_string(), // Updated URL with potential trailing slashes
      api_key: crate::api_models_dto::ApiKeyUpdateAction::Set(
        crate::api_models_dto::ApiKey::some("sk-updated123456789".to_string()).unwrap(),
      ), // New API key
      models: vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
      prefix: Some("openai".to_string()),
      forward_all_with_prefix: false,
    };

    // Make PUT request to update existing API model
    let response = test_router(Arc::new(app_service))
      .oneshot(Request::put(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS)).json(update_request)?)
      .await?;

    // Verify response status
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response body
    let api_response = response.json::<ApiModelResponse>().await?;

    // Create expected response with updated values
    let expected_response = create_expected_response(
      "openai-gpt4",
      "openai",
      expected_url, // Expected URL with trailing slashes removed
      Some("***"),  // Updated API key masked
      vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
      Some("openai".to_string()), // Updated prefix
      base_time,    // Original created_at
      api_response.updated_at, // Use actual updated_at (FrozenTimeService returns same time)
    );

    assert_eq!(expected_response, api_response);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_handler_not_found(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database (no seeded data)
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    let update_request = UpdateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v2".to_string(),
      api_key: crate::api_models_dto::ApiKeyUpdateAction::Set(
        crate::api_models_dto::ApiKey::some("sk-updated123456789".to_string()).unwrap(),
      ),
      models: vec!["gpt-4-turbo".to_string()],
      prefix: None,
      forward_all_with_prefix: false,
    };

    // Make PUT request to update non-existent API model
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::put(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS))
          .json(update_request)?,
      )
      .await?;

    // Verify response status is 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Verify error message
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("API model 'non-existent-alias' not found"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_api_model_handler_success(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let base_time = db_service.now();

    // Seed database with existing API model
    seed_test_api_models(&db_service, base_time).await?;

    // Create app service with seeded database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Make DELETE request to delete existing API model
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::delete(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    // Verify response status is 204 No Content
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify response has no body
    let body = response.into_body();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
    assert!(body_bytes.is_empty());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_api_model_handler_not_found(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database (no seeded data)
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Make DELETE request to delete non-existent API model
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::delete(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    // Verify response status is 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Verify error message
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("API model 'non-existent-alias' not found"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_api_model_handler_success(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let base_time = db_service.now();

    // Seed database with existing API model
    seed_test_api_models(&db_service, base_time).await?;

    // Create app service with seeded database
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Make GET request to retrieve specific API model
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::get(&format!("{}/openai-gpt4", ENDPOINT_API_MODELS))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    // Verify response status
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response body
    let api_response = response.json::<ApiModelResponse>().await?;

    // Create expected response
    let expected_response = create_expected_response(
      "openai-gpt4",
      "openai",
      "https://api.openai.com/v1",
      Some("***"), // Masked in get view (no API key provided)
      vec!["gpt-4".to_string()],
      None, // No prefix in original seed data
      base_time,
      base_time,
    );

    assert_eq!(expected_response, api_response);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_api_model_handler_not_found(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service with clean database (no seeded data)
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .with_secret_service()
      .build()?;

    // Make GET request to retrieve non-existent API model
    let response = test_router(Arc::new(app_service))
      .oneshot(
        Request::get(&format!("{}/non-existent-alias", ENDPOINT_API_MODELS))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    // Verify response status is 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Verify error message
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("API model 'non-existent-alias' not found"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_success(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = db_service.now();

    // Generate a unique ID for the test
    let test_id = Uuid::new_v4().to_string();

    // Create API model via database
    let api_alias = ApiAliasBuilder::test_default()
      .id(test_id.clone())
      .api_format(OpenAI)
      .base_url("https://api.openai.com/v1")
      .models(vec!["gpt-4".to_string()])
      .build_with_time(now)
      .unwrap();

    db_service
      .create_api_model_alias(&api_alias, Some("sk-test123".to_string()))
      .await?;

    // Verify it was created
    let retrieved = db_service.get_api_model_alias(&test_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, test_id);

    // Verify API key is encrypted but retrievable
    let api_key = db_service.get_api_key_for_alias(&test_id).await?;
    assert_eq!(api_key, Some("sk-test123".to_string()));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_api_model(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = db_service.now();

    // Create API model
    let api_alias = ApiAliasBuilder::test_default()
      .id("to-delete")
      .api_format(OpenAI)
      .base_url("https://api.openai.com/v1")
      .models(vec!["gpt-4".to_string()])
      .build_with_time(now)
      .unwrap();

    db_service
      .create_api_model_alias(&api_alias, Some("sk-test".to_string()))
      .await?;

    // Verify it exists
    assert!(db_service.get_api_model_alias("to-delete").await?.is_some());

    // Delete it
    db_service.delete_api_model_alias("to-delete").await?;

    // Verify it's gone
    assert!(db_service.get_api_model_alias("to-delete").await?.is_none());

    Ok(())
  }

  #[test]
  fn test_api_key_masking() {
    use crate::api_models_dto::mask_api_key;

    assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-...abcdef");
    assert_eq!(mask_api_key("short"), "***");
  }

  #[test]
  fn test_creds_enum_validation() {
    use crate::api_models_dto::{ApiKey, TestCreds};

    // Test with ApiKey credentials
    let test_request_with_key = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-direct-key".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(test_request_with_key.validate().is_ok());

    // Test with Id credentials
    let test_request_with_id = TestPromptRequest {
      creds: TestCreds::Id("stored-model-id".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(test_request_with_id.validate().is_ok());

    // Test with no authentication (ApiKey(None))
    let test_request_no_auth = TestPromptRequest {
      creds: TestCreds::ApiKey(ApiKey::none()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };
    assert!(test_request_no_auth.validate().is_ok());

    // Test FetchModelsRequest variants
    let fetch_request_with_key = FetchModelsRequest {
      creds: TestCreds::ApiKey(ApiKey::some("sk-direct-key".to_string()).unwrap()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(fetch_request_with_key.validate().is_ok());

    let fetch_request_with_id = FetchModelsRequest {
      creds: TestCreds::Id("stored-model-id".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
    };
    assert!(fetch_request_with_id.validate().is_ok());
  }

  #[rstest]
  #[case::prefix_removal(
    json!({
      "id": "test-prefix-removal",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "sk-test-key-123",
      "models": ["gpt-4"],
      "prefix": "azure/"
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"],
      "prefix": null
    }),
    json!({
      "id": "test-prefix-removal",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key_masked": "***",
      "models": ["gpt-4"],
      "prefix": null,
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[case::prefix_addition(
    json!({
      "id": "test-prefix-addition",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1", 
      "api_key": "sk-test-key-123",
      "models": ["gpt-4"],
      "prefix": null
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"],
      "prefix": "azure/"
    }),
    json!({
      "id": "test-prefix-addition",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key_masked": "***", 
      "models": ["gpt-4"],
      "prefix": "azure/",
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[case::prefix_empty_string_removal(
    json!({
      "id": "test-empty-string-removal",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "sk-test-key-123", 
      "models": ["gpt-4"],
      "prefix": "azure/"
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"],
      "prefix": ""
    }),
    json!({
      "id": "test-empty-string-removal",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key_masked": "***",
      "models": ["gpt-4"],
      "prefix": null,
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[case::prefix_change(
    json!({
      "id": "test-prefix-change",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "sk-test-key-123",
      "models": ["gpt-4"],
      "prefix": "azure/"
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "models": ["gpt-4"],
      "prefix": "openai:"
    }),
    json!({
      "id": "test-prefix-change",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key_masked": "***",
      "models": ["gpt-4"],
      "prefix": "openai:",
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[case::no_prefix_no_change(
    json!({
      "id": "test-no-prefix-no-change", 
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "sk-test-key-123",
      "models": ["gpt-4"],
      "prefix": null
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1", 
      "models": ["gpt-4"],
      "prefix": null
    }),
    json!({
      "id": "test-no-prefix-no-change",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key_masked": "***",
      "models": ["gpt-4"],
      "prefix": null,
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[case::models_and_url_update(
    json!({
      "id": "test-models-url-update",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v1",
      "api_key": "sk-old-key-123",
      "models": ["gpt-3.5-turbo"],
      "prefix": null
    }),
    json!({
      "api_format": "openai",
      "base_url": "https://api.openai.com/v2",
      "models": ["gpt-4", "gpt-3.5-turbo"],
      "prefix": null
    }),
    json!({
      "id": "test-models-url-update",
      "api_format": "openai",
      "base_url": "https://api.openai.com/v2",
      "api_key_masked": "***",
      "models": ["gpt-4", "gpt-3.5-turbo"],
      "prefix": null,
      "forward_all_with_prefix": false,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    })
  )]
  #[awt]
  #[tokio::test]
  async fn test_api_model_prefix_lifecycle(
    #[case] create_json: serde_json::Value,
    #[case] update_json: serde_json::Value,
    #[case] expected_get_json: serde_json::Value,
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let _base_time = db_service.now();

    // Parse create request
    let create_request: CreateApiModelRequest = serde_json::from_value(create_json.clone())?;

    // Create app service
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(Arc::new(db_service))
        .with_secret_service()
        .build()?,
    );

    // Step 1: Create the API model
    let create_response = test_router(app_service.clone())
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Get the generated model ID from the response
    let create_api_response: ApiModelResponse = create_response.json().await?;
    let model_id = create_api_response.id;

    // Step 2: Update the API model
    let update_request: UpdateApiModelRequest = serde_json::from_value(update_json)?;
    let update_response = test_router(app_service.clone())
      .oneshot(Request::put(&format!("{}/{}", ENDPOINT_API_MODELS, model_id)).json(update_request)?)
      .await?;

    assert_eq!(update_response.status(), StatusCode::OK);

    // Step 3: Get the API model and verify final state
    let get_response = test_router(app_service)
      .oneshot(
        Request::get(&format!("{}/{}", ENDPOINT_API_MODELS, model_id))
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(get_response.status(), StatusCode::OK);

    let api_response: ApiModelResponse = get_response.json().await?;

    // Build expected response with actual timestamps and generated ID
    let mut expected_response: ApiModelResponse = serde_json::from_value(expected_get_json)?;
    expected_response.id = api_response.id.clone(); // Use the generated UUID
    expected_response.created_at = api_response.created_at;
    expected_response.updated_at = api_response.updated_at;

    // Use pretty_assertions for comprehensive comparison
    assert_eq!(expected_response, api_response);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_forward_all_requires_prefix(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(Arc::new(db_service))
        .with_secret_service()
        .build()?,
    );

    // Try to create API model with forward_all=true but no prefix
    let create_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::none(),
      models: vec!["gpt-4".to_string()],
      prefix: None,                  // No prefix provided
      forward_all_with_prefix: true, // But forward_all is enabled
    };

    let response = test_router(app_service)
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Should return 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error message
    let error_body: serde_json::Value = response.json().await?;
    assert_eq!(error_body["error"]["code"].as_str().unwrap(), "obj_validation_error-forward_all_requires_prefix");

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_duplicate_prefix_error(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(Arc::new(db_service))
        .with_secret_service()
        .build()?,
    );

    // Create first API model with prefix
    let first_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::none(),
      models: vec!["gpt-4".to_string()],
      prefix: Some("azure/".to_string()),
      forward_all_with_prefix: false,
    };

    let response = test_router(app_service.clone())
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(first_request)?)
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create second API model with same prefix
    let second_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.anthropic.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::none(),
      models: vec!["claude-3".to_string()],
      prefix: Some("azure/".to_string()), // Same prefix
      forward_all_with_prefix: false,
    };

    let response = test_router(app_service)
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(second_request)?)
      .await?;

    // Should return 400 Bad Request with prefix_exists error
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error_body: serde_json::Value = response.json().await?;
    assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_duplicate_prefix_error(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create app service
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(Arc::new(db_service))
        .with_secret_service()
        .build()?,
    );

    // Create first API model with prefix "azure/"
    let first_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::none(),
      models: vec!["gpt-4".to_string()],
      prefix: Some("azure/".to_string()),
      forward_all_with_prefix: false,
    };

    let response = test_router(app_service.clone())
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(first_request)?)
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);

    // Create second API model with different prefix "anthropic/"
    let second_request = CreateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.anthropic.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKey::none(),
      models: vec!["claude-3".to_string()],
      prefix: Some("anthropic/".to_string()),
      forward_all_with_prefix: false,
    };

    let response = test_router(app_service.clone())
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(second_request)?)
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);
    let second_model: ApiModelResponse = response.json().await?;
    let second_model_id = second_model.id;

    // Try to update second model to use first model's prefix
    let update_request = UpdateApiModelRequest {
      api_format: OpenAI,
      base_url: "https://api.anthropic.com/v1".to_string(),
      api_key: crate::api_models_dto::ApiKeyUpdateAction::Keep,
      models: vec!["claude-3".to_string()],
      prefix: Some("azure/".to_string()), // Trying to use existing prefix
      forward_all_with_prefix: false,
    };

    let response = test_router(app_service)
      .oneshot(
        Request::put(&format!("{}/{}", ENDPOINT_API_MODELS, second_model_id))
          .json(update_request)?,
      )
      .await?;

    // Should return 400 Bad Request with prefix_exists error
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error_body: serde_json::Value = response.json().await?;
    assert_eq!(error_body["error"]["code"], "db_error-prefix_exists");

    Ok(())
  }
}
