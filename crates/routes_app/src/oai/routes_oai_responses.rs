use super::error::OAIRouteError;
use super::ENDPOINT_OAI_RESPONSES;
use crate::shared::AuthScope;
use crate::API_TAG_RESPONSES;
use crate::{JsonRejectionError, OaiApiError};
use async_openai::types::responses::{
  CreateResponse, DeleteResponse as OaiDeleteResponse, Response as OaiResponse,
};
use axum::extract::{Path, Query};
use axum::http::Method;
use axum::response::Response;
use axum::Json;
use axum_extra::extract::WithRejection;
use services::models::llm_liberty_envelope::ResolvedLlmLibertyCredentials;
use services::{Alias, ApiAlias, ApiFormat};
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

// Stack-only value used once per request; boxing both arms is overkill.
#[allow(clippy::large_enum_variant)]
enum ResponsesAliasResolution {
  Native {
    alias: ApiAlias,
    api_key: Option<String>,
  },
  Liberty {
    alias: ApiAlias,
    creds: ResolvedLlmLibertyCredentials,
  },
}

async fn resolve_responses_alias(
  auth_scope: &AuthScope,
  model: &str,
) -> Result<ResponsesAliasResolution, OaiApiError> {
  let alias = auth_scope.data().find_alias(model).await.ok_or_else(|| {
    OaiApiError::from(services::DataServiceError::AliasNotFound(model.to_string()))
  })?;

  let api_alias = match alias {
    Alias::Api(api_alias)
      if matches!(
        api_alias.api_format,
        ApiFormat::OpenAIResponses | ApiFormat::LlmLibertyOauth
      ) =>
    {
      api_alias
    }
    _ => {
      return Err(
        OAIRouteError::InvalidRequest(format!(
          "Model '{}' is not configured for Responses API format. Configure an alias with 'openai_responses' or 'llm_liberty_oauth' format.",
          model
        ))
        .into(),
      );
    }
  };

  if api_alias.api_format == ApiFormat::LlmLibertyOauth {
    let creds = crate::providers::resolve_llm_liberty_credentials(auth_scope, &api_alias.id)
      .await
      .map_err(OaiApiError::from)?;
    // Provider verification (creds.provider == "openai-codex") lives in the
    // factory's for_liberty -> LibertyProviderUnsupported (BadRequest).
    return Ok(ResponsesAliasResolution::Liberty {
      alias: api_alias,
      creds,
    });
  }

  let api_key = crate::providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
  Ok(ResponsesAliasResolution::Native {
    alias: api_alias,
    api_key,
  })
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

// Native and Liberty both forwarded via the unified ai_api() factory using the
// same (method, upstream_path) pair — no per-arm asymmetry.
async fn dispatch_responses_op(
  auth_scope: &AuthScope,
  resolution: ResponsesAliasResolution,
  method: Method,
  upstream_path: String,
  query_params: Option<Vec<(String, String)>>,
) -> Result<Response, OaiApiError> {
  let client = match resolution {
    ResponsesAliasResolution::Native { alias, api_key } => auth_scope
      .ai_api()
      .for_alias(&Alias::Api(alias), api_key)
      .map_err(OaiApiError::from)?,
    ResponsesAliasResolution::Liberty { alias, creds } => auth_scope
      .ai_api()
      .for_resolved(&creds, &alias)
      .map_err(OaiApiError::from)?,
  };
  client
    .forward_request_with_method(&method, &upstream_path, None, query_params, None)
    .await
    .map_err(OaiApiError::from)
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
  Query(query_params): Query<HashMap<String, String>>,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
) -> Result<Response, OaiApiError> {
  validate_responses_request(&request)?;

  let model = request
    .get("model")
    .and_then(|v| v.as_str())
    .expect("validated by validate_responses_request")
    .to_string();

  let resolution = resolve_responses_alias(&auth_scope, &model).await?;
  let params: Vec<(String, String)> = query_params.into_iter().collect();
  let params_opt = if params.is_empty() {
    None
  } else {
    Some(params)
  };

  let client = match resolution {
    ResponsesAliasResolution::Native {
      alias: api_alias,
      api_key,
    } => auth_scope
      .ai_api()
      .for_alias(&Alias::Api(api_alias), api_key)
      .map_err(OaiApiError::from)?,
    ResponsesAliasResolution::Liberty { alias, creds } => auth_scope
      .ai_api()
      .for_resolved(&creds, &alias)
      .map_err(OaiApiError::from)?,
  };
  client
    .forward_request_with_method(&Method::POST, "/responses", Some(request), params_opt, None)
    .await
    .map_err(OaiApiError::from)
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
  let resolution = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_path = format!("/responses/{}", response_id);
  dispatch_responses_op(
    &auth_scope,
    resolution,
    Method::GET,
    upstream_path,
    upstream_query_params(&params),
  )
  .await
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
  let resolution = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_path = format!("/responses/{}", response_id);
  dispatch_responses_op(
    &auth_scope,
    resolution,
    Method::DELETE,
    upstream_path,
    upstream_query_params(&params),
  )
  .await
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
  let resolution = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_path = format!("/responses/{}/input_items", response_id);
  dispatch_responses_op(
    &auth_scope,
    resolution,
    Method::GET,
    upstream_path,
    upstream_query_params(&params),
  )
  .await
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
  let resolution = resolve_responses_alias(&auth_scope, &model).await?;
  let upstream_path = format!("/responses/{}/cancel", response_id);
  dispatch_responses_op(
    &auth_scope,
    resolution,
    Method::POST,
    upstream_path,
    upstream_query_params(&params),
  )
  .await
}

#[cfg(test)]
#[path = "test_oai_responses_errors.rs"]
mod test_oai_responses_errors;

#[cfg(test)]
#[path = "test_oai_responses_success.rs"]
mod test_oai_responses_success;
