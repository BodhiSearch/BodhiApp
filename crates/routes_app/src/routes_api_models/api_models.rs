use crate::{
  ApiFormatsResponse, ApiModelResponse, CreateApiModelRequest, FetchModelsRequest,
  FetchModelsResponse, PaginatedApiModelResponse, PaginationSortParams, TestCreds,
  TestPromptRequest, TestPromptResponse, UpdateApiModelRequest, API_TAG_API_MODELS,
  ENDPOINT_API_MODELS, ENDPOINT_API_MODELS_API_FORMATS, ENDPOINT_API_MODELS_FETCH_MODELS,
  ENDPOINT_API_MODELS_TEST,
};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use axum_extra::extract::WithRejection;
use server_core::RouterState;
use services::{ApiAlias, ApiFormat};
use services::{ApiError, JsonRejectionError, ObjValidationError, OpenAIApiError};
use std::sync::Arc;
use ulid::Ulid;
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
        (status = 404, description = "API model with specified ID not found", body = services::OpenAIApiError,
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
    ApiError::from(services::EntityError::NotFound(format!(
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
        (status = 409, description = "Alias already exists", body = services::OpenAIApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn create_api_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateApiModelRequest>, JsonRejectionError>,
) -> Result<(StatusCode, Json<ApiModelResponse>), ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  // Additional validation: forward_all_with_prefix mode validation
  payload.validate_forward_all().map_err(|e| {
    if e.code.as_ref() == "prefix_required" {
      ApiError::from(ObjValidationError::ForwardAllRequiresPrefix)
    } else if e.code.as_ref() == "models_required" {
      // Convert validation error to ValidationErrors for consistency
      let mut errors = validator::ValidationErrors::new();
      errors.add("models", e);
      ApiError::from(ObjValidationError::ValidationErrors(errors))
    } else {
      ApiError::from(ObjValidationError::ForwardAllRequiresPrefix)
    }
  })?;

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Generate a unique ULID for the API model
  let id = Ulid::new().to_string();

  // Create the API model alias
  let now = time_service.utc_now();
  // Reset models to empty if forward_all_with_prefix is true
  let models = if payload.forward_all_with_prefix {
    Vec::new()
  } else {
    payload.models
  };

  let api_alias = ApiAlias::new(
    id,
    payload.api_format,
    payload.base_url.trim_end_matches('/').to_string(),
    models,
    payload.prefix,
    payload.forward_all_with_prefix,
    now,
  );

  // Convert ApiKey to Option<String> for DB
  let api_key_option = payload.api_key.as_option().map(|s| s.to_string());

  db_service
    .create_api_model_alias(&api_alias, api_key_option)
    .await?;

  // For forward_all models, populate cache asynchronously (fire-and-forget)
  if api_alias.forward_all_with_prefix {
    spawn_cache_refresh(state.app_service(), api_alias.id.clone());
  }

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
  WithRejection(Json(payload), _): WithRejection<Json<UpdateApiModelRequest>, JsonRejectionError>,
) -> Result<Json<ApiModelResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  // Additional validation: forward_all_with_prefix mode validation
  payload.validate_forward_all().map_err(|e| {
    if e.code.as_ref() == "prefix_required" {
      ApiError::from(ObjValidationError::ForwardAllRequiresPrefix)
    } else if e.code.as_ref() == "models_required" {
      // Convert validation error to ValidationErrors for consistency
      let mut errors = validator::ValidationErrors::new();
      errors.add("models", e);
      ApiError::from(ObjValidationError::ValidationErrors(errors))
    } else {
      ApiError::from(ObjValidationError::ForwardAllRequiresPrefix)
    }
  })?;

  let db_service = state.app_service().db_service();
  let time_service = state.app_service().time_service();

  // Get existing API model
  let mut api_alias = db_service.get_api_model_alias(&id).await?.ok_or_else(|| {
    ApiError::from(services::EntityError::NotFound(format!(
      "API model '{}' not found",
      id
    )))
  })?;

  // Update all fields (api_key is handled separately for security)
  api_alias.api_format = payload.api_format;
  api_alias.base_url = payload.base_url.trim_end_matches('/').to_string();
  api_alias.models = payload.models.into();
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
    return Err(ApiError::from(services::EntityError::NotFound(format!(
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
  WithRejection(Json(payload), _): WithRejection<Json<TestPromptRequest>, JsonRejectionError>,
) -> Result<Json<TestPromptResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  let ai_api_service = state.app_service().ai_api_service();
  let db_service = state.app_service().db_service();

  // Resolve credentials using TestCreds enum
  let result = match &payload.creds {
    TestCreds::ApiKey(api_key) => {
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
    TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(services::EntityError::NotFound(format!(
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
  WithRejection(Json(payload), _): WithRejection<Json<FetchModelsRequest>, JsonRejectionError>,
) -> Result<Json<FetchModelsResponse>, ApiError> {
  // Validate the request
  payload
    .validate()
    .map_err(|e| ApiError::from(ObjValidationError::ValidationErrors(e)))?;

  let ai_api_service = state.app_service().ai_api_service();
  let db_service = state.app_service().db_service();

  // Resolve credentials using TestCreds enum
  let models = match &payload.creds {
    TestCreds::ApiKey(api_key) => {
      // Use provided API key directly (or None for no authentication)
      ai_api_service
        .fetch_models(
          api_key.as_option().map(|s| s.to_string()),
          payload.base_url.trim_end_matches('/'),
        )
        .await?
    }
    TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db_service.get_api_model_alias(id).await?.ok_or_else(|| {
        ApiError::from(services::EntityError::NotFound(format!(
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

/// Synchronously populate cache with models for an API model alias
///
/// This endpoint fetches models from the external API and populates the cache synchronously.
/// Useful for testing to ensure cache is populated before proceeding with assertions.
#[utoipa::path(
    post,
    path = ENDPOINT_API_MODELS.to_owned() + "/{id}/sync-models",
    tag = API_TAG_API_MODELS,
    operation_id = "syncModels",
    summary = "Sync Models to Cache",
    description = "Synchronously fetches models from the external API and populates the cache. This ensures the cache is populated before returning. Primarily used for testing to avoid timing issues.",
    params(
        ("id" = String, Path, description = "Unique identifier for the API model alias", example = "openai-gpt4")
    ),
    responses(
        (status = 200, description = "Models synced to cache successfully", body = ApiModelResponse,
         example = json!({
             "id": "openai-gpt4",
             "api_format": "openai",
             "base_url": "https://api.openai.com/v1",
             "api_key_masked": "sk-****1234",
             "models": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"],
             "prefix": null,
             "forward_all_with_prefix": false,
             "created_at": "2024-01-01T00:00:00Z",
             "updated_at": "2024-01-01T00:00:00Z"
         })),
        (status = 404, description = "API model not found"),
        (status = 500, description = "Failed to sync models")
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn sync_models_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<ApiModelResponse>, ApiError> {
  let db_service = state.app_service().db_service();
  let ai_api_service = state.app_service().ai_api_service();
  let time_service = state.app_service().time_service();

  // Get the API alias - if not found, return error
  let Some(api_alias) = db_service.get_api_model_alias(&id).await? else {
    return Err(ApiError::from(services::EntityError::NotFound(format!(
      "API model {} not found",
      id
    ))));
  };

  // Get API key (optional)
  let api_key = db_service.get_api_key_for_alias(&id).await.ok().flatten();

  // Fetch models from remote API synchronously
  let models = ai_api_service
    .fetch_models(api_key, &api_alias.base_url)
    .await?;

  // Update cache in DB
  let now = time_service.utc_now();
  db_service
    .update_api_model_cache(&id, models.clone(), now)
    .await?;

  // Get refreshed alias
  let Some(updated_alias) = db_service.get_api_model_alias(&id).await? else {
    return Err(ApiError::from(services::EntityError::NotFound(format!(
      "API model {} not found",
      id
    ))));
  };

  // Check if API key exists
  let has_api_key = db_service
    .get_api_key_for_alias(&id)
    .await
    .ok()
    .flatten()
    .is_some();

  Ok(Json(ApiModelResponse::from_alias(
    updated_alias,
    has_api_key,
  )))
}

/// Helper function to spawn async cache refresh for forward_all models
fn spawn_cache_refresh(app_service: Arc<dyn services::AppService>, alias_id: String) {
  tokio::spawn(async move {
    let db = app_service.db_service();
    let ai_api = app_service.ai_api_service();
    let time_service = app_service.time_service();

    if let Ok(Some(alias)) = db.get_api_model_alias(&alias_id).await {
      let api_key = db.get_api_key_for_alias(&alias_id).await.ok().flatten();
      if let Ok(models) = ai_api.fetch_models(api_key, &alias.base_url).await {
        let now = time_service.utc_now();
        let _ = db.update_api_model_cache(&alias_id, models, now).await;
      }
    }
  });
}

#[cfg(test)]
#[path = "test_api_models_crud.rs"]
mod test_api_models_crud;

#[cfg(test)]
#[path = "test_api_models_validation.rs"]
mod test_api_models_validation;

#[cfg(test)]
#[path = "test_api_models_prefix.rs"]
mod test_api_models_prefix;

#[cfg(test)]
#[path = "test_api_models_sync.rs"]
mod test_api_models_sync;
