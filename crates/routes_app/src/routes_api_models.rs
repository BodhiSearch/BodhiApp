use crate::{
  api_models_dto::{
    ApiModelResponse, CreateApiModelRequest, FetchModelsRequest, FetchModelsResponse,
    PaginatedApiModelResponse, TestPromptRequest, TestPromptResponse, UpdateApiModelRequest,
  },
  PaginationSortParams, ENDPOINT_API_MODELS,
};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{ApiAlias, ApiError, BadRequestError, ObjValidationError, API_TAG_API_MODELS};
use server_core::RouterState;
use std::sync::Arc;
use validator::Validate;

/// List all API model configurations
#[utoipa::path(
    get,
    path = ENDPOINT_API_MODELS,
    tag = API_TAG_API_MODELS,
    operation_id = "listApiModels",
    params(PaginationSortParams),
    responses(
        (status = 200, description = "List of API models", body = PaginatedApiModelResponse),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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
      // For list view, we don't show the actual API key, just masked version
      ApiModelResponse::from_alias(alias.clone(), None)
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
    params(
        ("id" = String, Path, description = "API model ID")
    ),
    responses(
        (status = 200, description = "API model configuration", body = ApiModelResponse),
        (status = 404, description = "API model not found", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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

  Ok(Json(ApiModelResponse::from_alias(api_alias, None)))
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
        (status = 400, description = "Invalid request", body = objs::OpenAIApiError),
        (status = 409, description = "Alias already exists", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Check if id already exists
  if db_service.get_api_model_alias(&payload.id).await?.is_some() {
    return Err(ApiError::from(BadRequestError::new(format!(
      "API model ID '{}' already exists",
      payload.id
    ))));
  }

  // Create the API model alias
  let now = time_service.utc_now();
  let api_alias = ApiAlias::new(
    payload.id,
    payload.provider,
    payload.base_url,
    payload.models,
    now,
  );

  // Save to database with encrypted API key
  db_service
    .create_api_model_alias(&api_alias, &payload.api_key)
    .await?;

  // Return response with masked API key
  let response = ApiModelResponse::from_alias(api_alias, Some(payload.api_key));

  Ok((StatusCode::CREATED, Json(response)))
}

/// Update an existing API model configuration
#[utoipa::path(
    put,
    path = ENDPOINT_API_MODELS.to_owned() + "/{alias}",
    tag = API_TAG_API_MODELS,
    operation_id = "updateApiModel",
    params(
        ("alias" = String, Path, description = "API model alias")
    ),
    request_body = UpdateApiModelRequest,
    responses(
        (status = 200, description = "API model updated", body = ApiModelResponse),
        (status = 400, description = "Invalid request", body = objs::OpenAIApiError),
        (status = 404, description = "API model not found", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Get existing API model
  let mut api_alias = db_service.get_api_model_alias(&id).await?.ok_or_else(|| {
    ApiError::from(objs::EntityError::NotFound(format!(
      "API model '{}' not found",
      id
    )))
  })?;

  // Update fields if provided
  if let Some(provider) = payload.provider {
    api_alias.provider = provider;
  }
  if let Some(base_url) = payload.base_url {
    api_alias.base_url = base_url;
  }
  if let Some(models) = payload.models {
    api_alias.models = models;
  }

  api_alias.updated_at = time_service.utc_now();

  // Update in database
  db_service
    .update_api_model_alias(&id, &api_alias, payload.api_key.clone())
    .await?;

  // Return response with masked API key
  Ok(Json(ApiModelResponse::from_alias(
    api_alias,
    payload.api_key,
  )))
}

/// Delete an API model configuration
#[utoipa::path(
    delete,
    path = ENDPOINT_API_MODELS.to_owned() + "/{alias}",
    tag = API_TAG_API_MODELS,
    operation_id = "deleteApiModel",
    params(
        ("alias" = String, Path, description = "API model alias")
    ),
    responses(
        (status = 204, description = "API model deleted"),
        (status = 404, description = "API model not found", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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
    path = ENDPOINT_API_MODELS.to_owned() + "/test",
    tag = API_TAG_API_MODELS,
    operation_id = "testApiModel",
    request_body = TestPromptRequest,
    responses(
        (status = 200, description = "Test result", body = TestPromptResponse),
        (status = 400, description = "Invalid request", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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

  // Resolve API key and base URL - api_key takes preference if both are provided
  let (api_key, base_url) = match (&payload.api_key, &payload.id) {
    (Some(key), _) => {
      // Use provided API key directly (takes preference over id)
      (key.clone(), payload.base_url.clone())
    }
    (None, Some(id)) => {
      // Look up stored API key and use stored base URL
      let stored_key = db_service.get_api_key_for_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      (stored_key, api_model.base_url)
    }
    (None, None) => {
      // This should not happen due to validation, but handle gracefully
      return Err(ApiError::from(BadRequestError::new(
        "Either api_key or id must be provided".to_string(),
      )));
    }
  };

  // Test the API connection with resolved parameters
  match ai_api_service
    .test_prompt(&api_key, &base_url, &payload.model, &payload.prompt)
    .await
  {
    Ok(response) => Ok(Json(TestPromptResponse::success(response))),
    Err(err) => Ok(Json(TestPromptResponse::failure(err.to_string()))),
  }
}

/// Fetch available models from the API provider
#[utoipa::path(
    post,
    path = ENDPOINT_API_MODELS.to_owned() + "/fetch-models",
    tag = API_TAG_API_MODELS,
    operation_id = "fetchApiModels",
    request_body = FetchModelsRequest,
    responses(
        (status = 200, description = "Available models", body = FetchModelsResponse),
        (status = 400, description = "Invalid request", body = objs::OpenAIApiError),
        (status = 500, description = "Internal server error", body = objs::OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
        ("session_auth" = [])
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

  // Resolve API key and base URL - api_key takes preference if both are provided
  let (api_key, base_url) = match (&payload.api_key, &payload.id) {
    (Some(key), _) => {
      // Use provided API key directly (takes preference over id)
      (key.clone(), payload.base_url.clone())
    }
    (None, Some(id)) => {
      // Look up stored API key and use stored base URL
      let stored_key = db_service.get_api_key_for_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(objs::EntityError::NotFound(format!(
          "API model '{}' not found",
          id
        )))
      })?;

      (stored_key, api_model.base_url)
    }
    (None, None) => {
      // This should not happen due to validation, but handle gracefully
      return Err(ApiError::from(BadRequestError::new(
        "Either api_key or id must be provided".to_string(),
      )));
    }
  };

  // Fetch models from the API with resolved parameters
  let models = ai_api_service.fetch_models(&api_key, &base_url).await?;

  Ok(Json(FetchModelsResponse { models }))
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
  use objs::{test_utils::setup_l10n, ApiAlias, FluentLocalizationService};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
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
  use validator::Validate;

  /// Create expected ApiModelResponse for testing
  fn create_expected_response(
    id: &str,
    provider: &str,
    base_url: &str,
    api_key_masked: &str,
    models: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
  ) -> ApiModelResponse {
    ApiModelResponse {
      id: id.to_string(),
      provider: provider.to_string(),
      base_url: base_url.to_string(),
      api_key_masked: api_key_masked.to_string(),
      models,
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
      "***", // Masked in list view
      models,
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
        &format!("{}/{}", ENDPOINT_API_MODELS, "{alias}"),
        get(get_api_model_handler),
      )
      .route(
        &format!("{}/{}", ENDPOINT_API_MODELS, "{alias}"),
        put(update_api_model_handler),
      )
      .route(
        &format!("{}/{}", ENDPOINT_API_MODELS, "{alias}"),
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
    assert_eq!(response.total, 5);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 30);
    assert_eq!(response.data.len(), 5);

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
    ];

    let expected_response = PaginatedApiModelResponse {
      data: expected_data,
      total: 5,
      page: 1,
      page_size: 30,
    };

    // Use pretty_assertions for comprehensive comparison
    assert_eq!(expected_response, response);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_success(
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
      id: "openai-test".to_string(),
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test123456789".to_string(),
      models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
    };

    // Make POST request to create API model
    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify response body
    let api_response = response.json::<ApiModelResponse>().await?;

    // Create expected response (note: we can't predict the exact timestamp, so we'll check it separately)
    let expected_response = create_expected_response(
      "openai-test",
      "openai",
      "https://api.openai.com/v1",
      "sk-...456789",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      api_response.created_at, // Use actual timestamp
      api_response.updated_at, // Use actual timestamp
    );

    assert_eq!(expected_response, api_response);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_duplicate_alias(
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
      id: "openai-gpt4".to_string(), // This id already exists in seed data
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test123456789".to_string(),
      models: vec!["gpt-4".to_string()],
    };

    // Make POST request to create API model with duplicate alias
    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error message
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("API model ID 'openai-gpt4' already exists"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_handler_validation_error_empty_alias(
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
      id: "".to_string(), // Invalid: empty id
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test123456789".to_string(),
      models: vec!["gpt-4".to_string()],
    };

    let response = test_router(Arc::new(app_service))
      .oneshot(Request::post(ENDPOINT_API_MODELS).json(create_request)?)
      .await?;

    // Verify response status is 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify error response contains validation error for ID length
    let error_response = response.json::<serde_json::Value>().await?;
    let error_message = error_response["error"]["message"].as_str().unwrap();
    assert!(error_message.contains("ID must not be empty"));

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
      id: "test-alias".to_string(),
      provider: "openai".to_string(),
      base_url: "not-a-valid-url".to_string(), // Invalid: not a valid URL
      api_key: "sk-test123456789".to_string(),
      models: vec!["gpt-4".to_string()],
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
      id: "test-alias-2".to_string(),
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test123456789".to_string(),
      models: vec![], // Invalid: empty models array
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
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_handler_success(
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
      provider: Some("openai".to_string()),
      base_url: Some("https://api.openai.com/v2".to_string()), // Updated URL
      api_key: Some("sk-updated123456789".to_string()),        // New API key
      models: Some(vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()]), // Updated models
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
      "https://api.openai.com/v2",                          // Updated URL
      "sk-...456789",                                       // Updated API key masked
      vec!["gpt-4-turbo".to_string(), "gpt-4".to_string()], // Updated models
      base_time,                                            // Original created_at
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
      provider: Some("openai".to_string()),
      base_url: Some("https://api.openai.com/v2".to_string()),
      api_key: Some("sk-updated123456789".to_string()),
      models: Some(vec!["gpt-4-turbo".to_string()]),
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
      "***", // Masked in get view (no API key provided)
      vec!["gpt-4".to_string()],
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

    // Create request
    let request = CreateApiModelRequest {
      id: "openai-test".to_string(),
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      api_key: "sk-test123".to_string(),
      models: vec!["gpt-4".to_string()],
    };

    // Create API model via database
    let api_alias = ApiAlias::new(
      request.id.clone(),
      request.provider.clone(),
      request.base_url.clone(),
      request.models.clone(),
      now,
    );

    db_service
      .create_api_model_alias(&api_alias, &request.api_key)
      .await?;

    // Verify it was created
    let retrieved = db_service.get_api_model_alias(&request.id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "openai-test");

    // Verify API key is encrypted but retrievable
    let api_key = db_service.get_api_key_for_alias(&request.id).await?;
    assert_eq!(api_key, Some("sk-test123".to_string()));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = db_service.now();

    // Create initial API model
    let api_alias = ApiAlias::new(
      "test-alias".to_string(),
      "openai".to_string(),
      "https://api.openai.com/v1".to_string(),
      vec!["gpt-3.5-turbo".to_string()],
      now,
    );

    db_service
      .create_api_model_alias(&api_alias, "sk-old-key")
      .await?;

    // Update it
    let mut updated = api_alias.clone();
    updated.models = vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()];
    updated.base_url = "https://api.openai.com/v2".to_string();

    db_service
      .update_api_model_alias("test-alias", &updated, Some("sk-new-key".to_string()))
      .await?;

    // Verify updates
    let retrieved = db_service.get_api_model_alias("test-alias").await?.unwrap();
    assert_eq!(retrieved.models.len(), 2);
    assert_eq!(retrieved.base_url, "https://api.openai.com/v2");

    // Verify new API key
    let api_key = db_service.get_api_key_for_alias("test-alias").await?;
    assert_eq!(api_key, Some("sk-new-key".to_string()));

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
    let api_alias = ApiAlias::new(
      "to-delete".to_string(),
      "openai".to_string(),
      "https://api.openai.com/v1".to_string(),
      vec!["gpt-4".to_string()],
      now,
    );

    db_service
      .create_api_model_alias(&api_alias, "sk-test")
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
  fn test_api_key_preference_over_id() {
    // Test that when both api_key and id are provided, api_key is preferred
    let test_request = TestPromptRequest {
      api_key: Some("sk-direct-key".to_string()),
      id: Some("stored-model-id".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
      model: "gpt-4".to_string(),
      prompt: "Hello".to_string(),
    };

    // Validation should pass (both are provided, api_key takes preference)
    assert!(test_request.validate().is_ok());

    let fetch_request = FetchModelsRequest {
      api_key: Some("sk-direct-key".to_string()),
      id: Some("stored-model-id".to_string()),
      base_url: "https://api.openai.com/v1".to_string(),
    };

    // Validation should pass (both are provided, api_key takes preference)
    assert!(fetch_request.validate().is_ok());
  }
}
