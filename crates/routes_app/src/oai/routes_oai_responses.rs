use super::error::OAIRouteError;
use super::ENDPOINT_OAI_RESPONSES;
use crate::shared::AuthScope;
use crate::API_TAG_RESPONSES;
use crate::{ApiError, JsonRejectionError};
use async_openai::types::responses::{CreateResponse, Response as OaiResponse};
use axum::extract::{Path, Query};
use axum::response::Response;
use axum::Json;
use axum_extra::extract::WithRejection;
use services::inference::LlmEndpoint;
use services::{Alias, ApiFormat};
use std::collections::HashMap;

/// Validates basic structure of a Responses API create request
fn validate_responses_request(request: &serde_json::Value) -> Result<(), OAIRouteError> {
  // Validate model field exists and is a string
  if request.get("model").and_then(|v| v.as_str()).is_none() {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'model' is required and must be a string.".to_string(),
    ));
  }

  // Validate input field exists
  if request.get("input").is_none() {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'input' is required.".to_string(),
    ));
  }

  // Validate stream field is boolean if present
  if let Some(stream) = request.get("stream") {
    if !stream.is_boolean() {
      return Err(OAIRouteError::InvalidRequest(
        "Field 'stream' must be a boolean.".to_string(),
      ));
    }
  }

  Ok(())
}

/// Resolves an API alias for the Responses API, ensuring it has openai_responses format.
async fn resolve_responses_alias(
  auth_scope: &AuthScope,
  model: &str,
) -> Result<(services::ApiAlias, Option<String>), ApiError> {
  let alias =
    auth_scope.data().find_alias(model).await.ok_or_else(|| {
      ApiError::from(services::DataServiceError::AliasNotFound(model.to_string()))
    })?;

  let api_alias = match alias {
    Alias::Api(api_alias) if api_alias.api_format == ApiFormat::OpenAIResponses => api_alias,
    _ => {
      return Err(
        OAIRouteError::InvalidRequest(format!(
          "Model '{}' is not configured for Responses API format. Configure an alias with 'openai_responses' format.",
          model
        ))
        .into(),
      );
    }
  };

  let tenant_id = auth_scope.tenant_id().unwrap_or("").to_string();
  let user_id = auth_scope
    .auth_context()
    .user_id()
    .unwrap_or("")
    .to_string();
  let api_key = auth_scope
    .db_service()
    .get_api_key_for_alias(&tenant_id, &user_id, &api_alias.id)
    .await
    .ok()
    .flatten();

  Ok((api_alias, api_key))
}

/// Extracts and validates the required `model` query parameter for ID-based endpoints.
fn extract_model_param(params: &HashMap<String, String>) -> Result<String, OAIRouteError> {
  params
    .get("model")
    .filter(|m| !m.is_empty())
    .cloned()
    .ok_or_else(|| {
      OAIRouteError::InvalidRequest("Query parameter 'model' is required for routing.".to_string())
    })
}

/// Filters query params, removing `model` (consumed for routing) before forwarding upstream.
fn upstream_query_params(params: &HashMap<String, String>) -> Vec<(String, String)> {
  params
    .iter()
    .filter(|(k, _)| k.as_str() != "model")
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect()
}

/// Create a response (OpenAI Responses API)
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_RESPONSES,
    tag = API_TAG_RESPONSES,
    operation_id = "createResponse",
    summary = "Create Response (OpenAI Responses API)",
    description = "Creates a model response using the Responses API format. Proxied to the upstream provider. Supports both streaming and non-streaming responses.",
    request_body(content = CreateResponse),
    responses(
        (status = 200, description = "Response created",
         content_type = "application/json",
         body = OaiResponse),
        (status = 201, description = "Response stream (actual status is 200, using 201 to avoid OpenAPI limitation).",
         content_type = "text/event-stream",
         body = serde_json::Value),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn responses_create_handler(
  auth_scope: AuthScope,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
) -> Result<Response, ApiError> {
  validate_responses_request(&request)?;

  let model = request
    .get("model")
    .and_then(|v| v.as_str())
    .unwrap()
    .to_string();

  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let inference = auth_scope.inference();
  let response = inference
    .forward_remote(LlmEndpoint::Responses, request, &api_alias, api_key)
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

/// Retrieve a response by ID
#[utoipa::path(
    get,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}",
    tag = API_TAG_RESPONSES,
    operation_id = "getResponse",
    summary = "Retrieve Response",
    description = "Retrieves a previously created response by ID. Requires `model` query parameter for routing.",
    params(
        ("response_id" = String, Path, description = "The response ID"),
        ("model" = String, Query, description = "Model name for routing to the correct upstream provider"),
    ),
    responses(
        (status = 200, description = "Response retrieved", body = OaiResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn responses_get_handler(
  auth_scope: AuthScope,
  Path(response_id): Path<String>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_params = upstream_query_params(&params);

  let inference = auth_scope.inference();
  let query_params = if upstream_params.is_empty() {
    None
  } else {
    Some(upstream_params)
  };
  let response = inference
    .forward_remote_with_params(
      LlmEndpoint::ResponsesGet(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
      query_params,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

/// Delete a response by ID
#[utoipa::path(
    delete,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}",
    tag = API_TAG_RESPONSES,
    operation_id = "deleteResponse",
    summary = "Delete Response",
    description = "Deletes a stored response by ID. Requires `model` query parameter for routing.",
    params(
        ("response_id" = String, Path, description = "The response ID"),
        ("model" = String, Query, description = "Model name for routing to the correct upstream provider"),
    ),
    responses(
        (status = 200, description = "Response deleted"),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn responses_delete_handler(
  auth_scope: AuthScope,
  Path(response_id): Path<String>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let inference = auth_scope.inference();
  let response = inference
    .forward_remote(
      LlmEndpoint::ResponsesDelete(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

/// List input items for a response
#[utoipa::path(
    get,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}/input_items",
    tag = API_TAG_RESPONSES,
    operation_id = "listResponseInputItems",
    summary = "List Response Input Items",
    description = "Lists input items for a given response. Requires `model` query parameter for routing.",
    params(
        ("response_id" = String, Path, description = "The response ID"),
        ("model" = String, Query, description = "Model name for routing to the correct upstream provider"),
    ),
    responses(
        (status = 200, description = "Input items retrieved"),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn responses_input_items_handler(
  auth_scope: AuthScope,
  Path(response_id): Path<String>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_params = upstream_query_params(&params);

  let inference = auth_scope.inference();
  let query_params = if upstream_params.is_empty() {
    None
  } else {
    Some(upstream_params)
  };
  let response = inference
    .forward_remote_with_params(
      LlmEndpoint::ResponsesInputItems(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
      query_params,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

/// Cancel a background response
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}/cancel",
    tag = API_TAG_RESPONSES,
    operation_id = "cancelResponse",
    summary = "Cancel Response",
    description = "Cancels a background response. Requires `model` query parameter for routing.",
    params(
        ("response_id" = String, Path, description = "The response ID"),
        ("model" = String, Query, description = "Model name for routing to the correct upstream provider"),
    ),
    responses(
        (status = 200, description = "Response cancelled", body = OaiResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn responses_cancel_handler(
  auth_scope: AuthScope,
  Path(response_id): Path<String>,
  Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let inference = auth_scope.inference();
  let response = inference
    .forward_remote(
      LlmEndpoint::ResponsesCancel(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

#[cfg(test)]
#[path = "test_oai_responses.rs"]
mod test_oai_responses;
