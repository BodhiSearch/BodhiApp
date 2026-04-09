use crate::shared::AuthScope;
use crate::{ApiError, BodhiApiError, ValidatedJson};
use crate::{
  API_TAG_MODELS_API, ENDPOINT_MODELS_API, ENDPOINT_MODELS_API_FETCH_MODELS,
  ENDPOINT_MODELS_API_FORMATS, ENDPOINT_MODELS_API_TEST,
};
use axum::{extract::Path, http::StatusCode, Json};
use services::{
  ApiAliasResponse, ApiFormat, ApiFormatsResponse, ApiModelRequest, FetchModelsRequest,
  FetchModelsResponse, TestCreds, TestPromptRequest, TestPromptResponse,
};

/// Get a specific API model configuration
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS_API.to_owned() + "/{id}",
    tag = API_TAG_MODELS_API,
    operation_id = "getApiModel",
    summary = "Get API Model Configuration",
    description = "Retrieves detailed configuration for a specific API model alias by ID. API keys are masked for security unless explicitly requested.",
    params(
        ("id" = String, Path, description = "Unique identifier for the API model alias", example = "openai-gpt4")
    ),
    responses(
        (status = 200, description = "API model configuration retrieved successfully", body = ApiAliasResponse,
         example = json!({
             "id": "openai-gpt4",
             "api_format": "openai",
             "base_url": "https://api.openai.com/v1",
             "has_api_key": true,
             "models": ["gpt-4"]
         })),
        (status = 404, description = "API model with specified ID not found", body = BodhiApiError,
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
pub async fn api_models_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<ApiAliasResponse>, ApiError> {
  let result = auth_scope.api_models().get(&id).await?;
  Ok(Json(result))
}

/// Create a new API model configuration
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_API,
    tag = API_TAG_MODELS_API,
    operation_id = "createApiModel",
    request_body = ApiModelRequest,
    responses(
        (status = 201, description = "API model created", body = ApiAliasResponse),
        (status = 409, description = "Alias already exists", body = BodhiApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_create(
  auth_scope: AuthScope,
  ValidatedJson(form): ValidatedJson<ApiModelRequest>,
) -> Result<(StatusCode, Json<ApiAliasResponse>), ApiError> {
  let result = auth_scope.api_models().create(form).await?;
  Ok((StatusCode::CREATED, Json(result)))
}

/// Update an existing API model configuration
#[utoipa::path(
    put,
    path = ENDPOINT_MODELS_API.to_owned() + "/{id}",
    tag = API_TAG_MODELS_API,
    operation_id = "updateApiModel",
    params(
        ("id" = String, Path, description = "API model ID")
    ),
    request_body = ApiModelRequest,
    responses(
        (status = 200, description = "API model updated", body = ApiAliasResponse),
        (status = 404, description = "API model not found", body = BodhiApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(form): ValidatedJson<ApiModelRequest>,
) -> Result<Json<ApiAliasResponse>, ApiError> {
  let result = auth_scope.api_models().update(&id, form).await?;
  Ok(Json(result))
}

/// Delete an API model configuration
#[utoipa::path(
    delete,
    path = ENDPOINT_MODELS_API.to_owned() + "/{id}",
    tag = API_TAG_MODELS_API,
    operation_id = "deleteApiModel",
    params(
        ("id" = String, Path, description = "API model ID")
    ),
    responses(
        (status = 204, description = "API model deleted"),
        (status = 404, description = "API model not found", body = BodhiApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_destroy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  auth_scope.api_models().delete(&id).await?;
  Ok(StatusCode::NO_CONTENT)
}

/// Test API connectivity with a prompt.
//
// CRUD uniformity exception: This is a utility endpoint, not CRUD. It uses
// `require_tenant_id()` / `require_user_id()` directly to resolve stored
// credentials when `TestCreds::Id` is provided, since this operation crosses
// the CRUD boundary (reading from an existing API model to perform a test).
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_API_TEST.to_owned(),
    tag = API_TAG_MODELS_API,
    operation_id = "testApiModel",
    request_body = TestPromptRequest,
    responses(
        (status = 200, description = "Test result", body = TestPromptResponse),
        (status = 400, description = "Invalid request", body = BodhiApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_test(
  auth_scope: AuthScope,
  ValidatedJson(payload): ValidatedJson<TestPromptRequest>,
) -> Result<Json<TestPromptResponse>, ApiError> {
  let ai_api = auth_scope.ai_api();
  let db = auth_scope.db();
  let tenant_id = auth_scope.require_tenant_id()?;
  let user_id = auth_scope.require_user_id()?;

  // Resolve credentials using TestCreds enum
  let result = match &payload.creds {
    TestCreds::ApiKey(api_key) => {
      // Use provided API key directly (or None for no authentication)
      ai_api
        .test_prompt(
          api_key.as_option().map(|s| s.to_string()),
          payload.base_url.trim_end_matches('/'),
          &payload.model,
          &payload.prompt,
          &payload.api_format,
        )
        .await
    }
    TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db
        .get_api_model_alias(tenant_id, user_id, id)
        .await?
        .ok_or_else(|| {
          ApiError::from(services::EntityError::NotFound(format!(
            "API model '{}' not found",
            id
          )))
        })?;

      // Get stored key (may be None if no key configured)
      let stored_key = db.get_api_key_for_alias(tenant_id, user_id, id).await?;

      ai_api
        .test_prompt(
          stored_key,
          api_model.base_url.trim_end_matches('/'),
          &payload.model,
          &payload.prompt,
          &payload.api_format,
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

/// Fetch available models from the API.
//
// CRUD uniformity exception: This is a utility endpoint, not CRUD. It uses
// `require_tenant_id()` / `require_user_id()` directly to resolve stored
// credentials when `TestCreds::Id` is provided, since this operation crosses
// the CRUD boundary (reading from an existing API model to perform a fetch).
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_API_FETCH_MODELS.to_owned(),
    tag = API_TAG_MODELS_API,
    operation_id = "fetchApiModels",
    request_body = FetchModelsRequest,
    responses(
        (status = 200, description = "Available models", body = FetchModelsResponse),
        (status = 400, description = "Invalid request", body = BodhiApiError),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_fetch_models(
  auth_scope: AuthScope,
  ValidatedJson(payload): ValidatedJson<FetchModelsRequest>,
) -> Result<Json<FetchModelsResponse>, ApiError> {
  let ai_api = auth_scope.ai_api();
  let db = auth_scope.db();
  let tenant_id = auth_scope.require_tenant_id()?;
  let user_id = auth_scope.require_user_id()?;

  // Resolve credentials using TestCreds enum
  let models = match &payload.creds {
    TestCreds::ApiKey(api_key) => {
      // Use provided API key directly (or None for no authentication)
      ai_api
        .fetch_models(
          api_key.as_option().map(|s| s.to_string()),
          payload.base_url.trim_end_matches('/'),
        )
        .await?
    }
    TestCreds::Id(id) => {
      // Look up stored model configuration by ID
      let api_model = db
        .get_api_model_alias(tenant_id, user_id, id)
        .await?
        .ok_or_else(|| {
          ApiError::from(services::EntityError::NotFound(format!(
            "API model '{}' not found",
            id
          )))
        })?;

      // Get stored key (may be None if no key configured)
      let stored_key = db.get_api_key_for_alias(tenant_id, user_id, id).await?;

      ai_api
        .fetch_models(stored_key, api_model.base_url.trim_end_matches('/'))
        .await?
    }
  };

  Ok(Json(FetchModelsResponse { models }))
}

/// Get available API formats
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS_API_FORMATS.to_owned(),
    tag = API_TAG_MODELS_API,
    operation_id = "getApiFormats",
    summary = "Get Available API Formats",
    description = "Retrieves list of supported API formats/protocols: 'openai' (Chat Completions) and 'openai_responses' (Responses API).",
    responses(
        (status = 200, description = "API formats retrieved successfully", body = ApiFormatsResponse,
         example = json!({
             "data": ["openai", "openai_responses"]
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
pub async fn api_models_formats() -> Result<Json<ApiFormatsResponse>, ApiError> {
  Ok(Json(ApiFormatsResponse {
    data: vec![ApiFormat::OpenAI, ApiFormat::OpenAIResponses],
  }))
}

/// Synchronously populate cache with models for an API model alias
///
/// This endpoint fetches models from the external API and populates the cache synchronously.
/// Useful for testing to ensure cache is populated before proceeding with assertions.
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_API.to_owned() + "/{id}/sync-models",
    tag = API_TAG_MODELS_API,
    operation_id = "syncModels",
    summary = "Sync Models to Cache",
    description = "Synchronously fetches models from the external API and populates the cache. This ensures the cache is populated before returning. Primarily used for testing to avoid timing issues.",
    params(
        ("id" = String, Path, description = "Unique identifier for the API model alias", example = "openai-gpt4")
    ),
    responses(
        (status = 200, description = "Models synced to cache successfully", body = ApiAliasResponse,
         example = json!({
             "id": "openai-gpt4",
             "api_format": "openai",
             "base_url": "https://api.openai.com/v1",
             "has_api_key": true,
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
pub async fn api_models_sync(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<ApiAliasResponse>, ApiError> {
  let result = auth_scope.api_models().sync_cache(&id).await?;
  Ok(Json(result))
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
