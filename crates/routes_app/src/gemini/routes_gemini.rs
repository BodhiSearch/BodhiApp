use crate::oai::OAIRouteError;
use crate::shared::AuthScope;
use crate::{BodhiErrorResponse, GeminiApiError, JsonRejectionError};
use axum::extract::{Path, Query};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use axum_extra::extract::WithRejection;
use services::inference::LlmEndpoint;
use services::{Alias, ApiAlias, ApiFormat, ApiModel, DataServiceError};
use std::collections::HashSet;

/// Forward `x-goog-*` request headers to upstream Gemini so SDK telemetry
/// (`x-goog-api-client`, `x-goog-request-params`) reaches Google.
fn extract_gemini_headers(headers: &HeaderMap) -> Option<Vec<(String, String)>> {
  let forwarded: Vec<(String, String)> = headers
    .iter()
    .filter(|(name, _)| name.as_str().to_ascii_lowercase().starts_with("x-goog-"))
    .filter_map(|(name, value)| {
      value
        .to_str()
        .ok()
        .map(|v| (name.as_str().to_string(), v.to_string()))
    })
    .collect();
  if forwarded.is_empty() {
    None
  } else {
    Some(forwarded)
  }
}

/// Path-parameter safety: rejects non-ASCII and special chars that could cause
/// URL-injection when forwarded. Allows alphanumeric, `-`, `_`, `.`, `/`.
fn validate_model_id(id: &str) -> Result<(), GeminiApiError> {
  if id.is_empty()
    || id.len() > 128
    || !id
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
  {
    return Err(GeminiApiError::invalid_request("Invalid model_id format."));
  }
  Ok(())
}

async fn resolve_gemini_alias(
  auth_scope: &AuthScope,
  model: &str,
) -> Result<(ApiAlias, Option<String>), BodhiErrorResponse> {
  // Check if any alias at all supports this model (for helpful format-mismatch errors).
  let alias = auth_scope.data().find_alias(model).await;
  if let Some(Alias::Api(ref api_alias)) = alias {
    if !matches!(api_alias.api_format, ApiFormat::Gemini) {
      return Err(
        OAIRouteError::InvalidRequest(format!(
          "Model '{}' is not configured for Gemini API format. Configure an alias with 'gemini' format.",
          model
        ))
        .into(),
      );
    }
  }

  // Match against the alias's matchable_models() — which prepends the alias prefix
  // (if any) to each bare Gemini `model_id()`.
  let aliases = list_user_gemini_aliases(auth_scope).await?;
  let api_alias = aliases
    .into_iter()
    .find(|alias| alias.matchable_models().iter().any(|m| m == model))
    .ok_or_else(|| BodhiErrorResponse::from(DataServiceError::AliasNotFound(model.to_string())))?;

  let api_key = crate::providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
  Ok((api_alias, api_key))
}

async fn list_user_gemini_aliases(
  auth_scope: &AuthScope,
) -> Result<Vec<ApiAlias>, BodhiErrorResponse> {
  let aliases = auth_scope
    .data()
    .list_aliases()
    .await
    .map_err(BodhiErrorResponse::from)?;
  Ok(
    aliases
      .into_iter()
      .filter_map(|alias| match alias {
        Alias::Api(api_alias) if matches!(api_alias.api_format, ApiFormat::Gemini) => {
          Some(api_alias)
        }
        _ => None,
      })
      .collect(),
  )
}

/// Strip alias prefix — upstream Gemini doesn't know our prefix; strip before forwarding.
fn strip_alias_prefix(model: &str, alias: &ApiAlias) -> String {
  if let Some(prefix) = &alias.prefix {
    if !prefix.is_empty() {
      if let Some(stripped) = model.strip_prefix(prefix.as_str()) {
        return stripped.to_string();
      }
    }
  }
  model.to_string()
}

/// Serializes a GeminiModel and overwrites its `name` field with `models/{prefix}{model_id}`
/// so SDK clients receive a name that round-trips through `:generateContent` paths.
fn gemini_model_to_json(m: &services::GeminiModel, prefix: &str) -> serde_json::Value {
  let mut json = serde_json::to_value(m).unwrap_or_default();
  if let Some(obj) = json.as_object_mut() {
    obj.insert(
      "name".to_string(),
      serde_json::Value::String(format!("models/{}{}", prefix, m.model_id())),
    );
  }
  json
}

/// Aggregates Gemini models from all aliases; served from cached metadata, no upstream calls.
pub async fn gemini_models_list(
  auth_scope: AuthScope,
) -> Result<Json<serde_json::Value>, GeminiApiError> {
  let aliases = list_user_gemini_aliases(&auth_scope).await?;

  let mut seen: HashSet<String> = HashSet::new();
  let mut ordered: Vec<serde_json::Value> = Vec::new();

  for alias in &aliases {
    let prefix = alias.prefix.as_deref().unwrap_or("");
    for model in alias.models.iter() {
      if let ApiModel::Gemini(m) = model {
        let key = format!("{}{}", prefix, m.model_id());
        if seen.insert(key) {
          ordered.push(gemini_model_to_json(m, prefix));
        }
      }
    }
  }

  Ok(Json(serde_json::json!({
    "models": ordered,
  })))
}

/// Returns a single model's metadata from locally cached data.
pub async fn gemini_models_get(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, GeminiApiError> {
  let model_id = id;
  validate_model_id(&model_id)?;

  let aliases = list_user_gemini_aliases(&auth_scope).await?;

  for alias in &aliases {
    let prefix = alias.prefix.as_deref().unwrap_or("");
    for model in alias.models.iter() {
      if let ApiModel::Gemini(m) = model {
        let prefixed = format!("{}{}", prefix, m.model_id());
        if prefixed == model_id {
          return Ok(Json(gemini_model_to_json(m, prefix)));
        }
      }
    }
  }

  Err(GeminiApiError::not_found(format!(
    "Model '{}' not found.",
    model_id
  )))
}

/// Supported Gemini action suffixes (the part after `:` in the path segment).
const GEMINI_ACTIONS: &[&str] = &["generateContent", "streamGenerateContent", "embedContent"];

/// Dispatches `POST /v1beta/models/{model}:{action}` — captures whole segment, splits on last `:`.
pub async fn gemini_action_handler(
  auth_scope: AuthScope,
  headers: HeaderMap,
  Path(id): Path<String>,
  // Forward query params verbatim (e.g. Gemini's ?alt=sse for SSE).
  Query(query_params): Query<std::collections::HashMap<String, String>>,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
) -> Result<Response, GeminiApiError> {
  let client_headers = extract_gemini_headers(&headers);
  let (model, action) = match id.rsplit_once(':') {
    Some((m, a)) => (m, a),
    None => {
      return Err(GeminiApiError::invalid_request(format!(
        "Invalid action path '{}'. Expected format: '{{model}}:{{action}}'.",
        id
      )));
    }
  };

  if !GEMINI_ACTIONS.contains(&action) {
    return Err(GeminiApiError::invalid_request(format!(
      "Unsupported action '{}'. Supported: generateContent, streamGenerateContent, embedContent.",
      action
    )));
  }

  validate_model_id(model)?;

  let (api_alias, api_key) = resolve_gemini_alias(&auth_scope, model).await?;
  let stripped_model = strip_alias_prefix(model, &api_alias);

  let endpoint = match action {
    "generateContent" => LlmEndpoint::GeminiGenerateContent(stripped_model),
    "streamGenerateContent" => LlmEndpoint::GeminiStreamGenerateContent(stripped_model),
    "embedContent" => LlmEndpoint::GeminiEmbedContent(stripped_model),
    _ => unreachable!("action validated above"),
  };

  let forwarded_params: Vec<(String, String)> = query_params.into_iter().collect();
  let forwarded_params_opt = if forwarded_params.is_empty() {
    None
  } else {
    Some(forwarded_params)
  };

  let response = auth_scope
    .inference()
    .forward_remote_with_params(
      endpoint,
      request,
      &api_alias,
      api_key,
      forwarded_params_opt,
      client_headers,
    )
    .await
    .map_err(BodhiErrorResponse::from)?;

  Ok(response)
}

#[cfg(test)]
#[path = "test_gemini_routes.rs"]
mod test_gemini_routes;
