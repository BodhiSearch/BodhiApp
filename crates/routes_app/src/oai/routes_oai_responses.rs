use super::error::OAIRouteError;
use super::ENDPOINT_OAI_RESPONSES;
use crate::shared::AuthScope;
use crate::API_TAG_RESPONSES;
use crate::{ApiError, JsonRejectionError, OaiApiError};
use async_openai::types::responses::{
  CreateResponse, DeleteResponse as OaiDeleteResponse, Response as OaiResponse,
};
use axum::extract::{Path, Query};
use axum::response::Response;
use axum::Json;
use axum_extra::extract::WithRejection;
use services::inference::LlmEndpoint;
use services::{Alias, ApiFormat};
use std::collections::HashMap;

fn validate_responses_request(request: &serde_json::Value) -> Result<(), OAIRouteError> {
  if request.get("model").and_then(|v| v.as_str()).is_none() {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'model' is required and must be a string.".to_string(),
    ));
  }

  if request.get("input").is_none() {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'input' is required.".to_string(),
    ));
  }

  if let Some(stream) = request.get("stream") {
    if !stream.is_boolean() {
      return Err(OAIRouteError::InvalidRequest(
        "Field 'stream' must be a boolean.".to_string(),
      ));
    }
  }

  Ok(())
}

fn validate_response_id(id: &str) -> Result<(), OAIRouteError> {
  if id.is_empty()
    || id.len() > 256
    || !id
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
  {
    return Err(OAIRouteError::InvalidRequest(
      "Invalid response_id format.".to_string(),
    ));
  }
  Ok(())
}

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

  let api_key = crate::providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
  Ok((api_alias, api_key))
}

fn extract_model_param(params: &HashMap<String, String>) -> Result<String, OAIRouteError> {
  params
    .get("model")
    .filter(|m| !m.is_empty())
    .cloned()
    .ok_or_else(|| {
      OAIRouteError::InvalidRequest("Query parameter 'model' is required for routing.".to_string())
    })
}

fn upstream_query_params(params: &HashMap<String, String>) -> Option<Vec<(String, String)>> {
  let filtered: Vec<_> = params
    .iter()
    .filter(|(k, _)| k.as_str() != "model")
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
  if filtered.is_empty() {
    None
  } else {
    Some(filtered)
  }
}

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
) -> Result<Response, OaiApiError> {
  validate_responses_request(&request)?;

  let model = request
    .get("model")
    .and_then(|v| v.as_str())
    .expect("validated by validate_responses_request")
    .to_string();

  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let response = auth_scope
    .inference()
    .forward_remote(LlmEndpoint::Responses, request, &api_alias, api_key)
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

#[utoipa::path(
    get,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}",
    tag = API_TAG_RESPONSES,
    operation_id = "getResponse",
    summary = "Retrieve Response",
    description = "Retrieves a previously created response by ID. Note: `model` query parameter is required for multi-provider routing (not part of upstream OpenAI API).",
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
) -> Result<Response, OaiApiError> {
  validate_response_id(&response_id)?;
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let response = auth_scope
    .inference()
    .forward_remote_with_params(
      LlmEndpoint::ResponsesGet(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
      upstream_query_params(&params),
      None,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

#[utoipa::path(
    delete,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}",
    tag = API_TAG_RESPONSES,
    operation_id = "deleteResponse",
    summary = "Delete Response",
    description = "Deletes a stored response by ID. Note: `model` query parameter is required for multi-provider routing (not part of upstream OpenAI API).",
    params(
        ("response_id" = String, Path, description = "The response ID"),
        ("model" = String, Query, description = "Model name for routing to the correct upstream provider"),
    ),
    responses(
        (status = 200, description = "Response deleted", body = OaiDeleteResponse),
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
) -> Result<Response, OaiApiError> {
  validate_response_id(&response_id)?;
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let response = auth_scope
    .inference()
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

#[utoipa::path(
    get,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}/input_items",
    tag = API_TAG_RESPONSES,
    operation_id = "listResponseInputItems",
    summary = "List Response Input Items",
    description = "Lists input items for a given response. Note: `model` query parameter is required for multi-provider routing (not part of upstream OpenAI API).",
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
) -> Result<Response, OaiApiError> {
  validate_response_id(&response_id)?;
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let response = auth_scope
    .inference()
    .forward_remote_with_params(
      LlmEndpoint::ResponsesInputItems(response_id),
      serde_json::Value::Null,
      &api_alias,
      api_key,
      upstream_query_params(&params),
      None,
    )
    .await
    .map_err(ApiError::from)?;

  Ok(response)
}

#[utoipa::path(
    post,
    path = ENDPOINT_OAI_RESPONSES.to_owned() + "/{response_id}/cancel",
    tag = API_TAG_RESPONSES,
    operation_id = "cancelResponse",
    summary = "Cancel Response",
    description = "Cancels a background response. Note: `model` query parameter is required for multi-provider routing (not part of upstream OpenAI API).",
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
) -> Result<Response, OaiApiError> {
  validate_response_id(&response_id)?;
  let model = extract_model_param(&params)?;
  let (api_alias, api_key) = resolve_responses_alias(&auth_scope, &model).await?;

  let response = auth_scope
    .inference()
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
