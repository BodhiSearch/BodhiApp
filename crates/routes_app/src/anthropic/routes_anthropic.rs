use crate::oai::OAIRouteError;
use crate::shared::AuthScope;
use crate::{AnthropicApiError, BodhiErrorResponse, JsonRejectionError};
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::Json;
use axum_extra::extract::WithRejection;
use services::inference::LlmEndpoint;
use services::models::LlmLibertyRequestParts;
use services::{Alias, ApiAlias, ApiFormat, ApiModel, DataServiceError};
use std::collections::{HashMap, HashSet};

/// Path-parameter safety: rejects non-ASCII and special chars that could cause
/// URL-injection when forwarded. Allows alphanumeric, `-`, `_`, `.` (e.g. `claude-3.5-sonnet`).
fn validate_model_id(id: &str) -> Result<(), AnthropicApiError> {
  if id.is_empty()
    || id.len() > 128
    || !id
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
  {
    return Err(AnthropicApiError::invalid_request(
      "Invalid model_id format.",
    ));
  }
  Ok(())
}

fn extract_anthropic_headers(headers: &HeaderMap) -> Option<Vec<(String, String)>> {
  let forwarded: Vec<(String, String)> = headers
    .iter()
    .filter(|(name, _)| name.as_str().to_ascii_lowercase().starts_with("anthropic-"))
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

async fn resolve_anthropic_alias(
  auth_scope: &AuthScope,
  model: &str,
) -> Result<(ApiAlias, Option<String>), BodhiErrorResponse> {
  let alias =
    auth_scope.data().find_alias(model).await.ok_or_else(|| {
      BodhiErrorResponse::from(DataServiceError::AliasNotFound(model.to_string()))
    })?;

  let mut api_alias = match alias {
    Alias::Api(api_alias)
      if matches!(
        api_alias.api_format,
        ApiFormat::Anthropic | ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth
      ) =>
    {
      api_alias
    }
    _ => {
      return Err(
        OAIRouteError::InvalidRequest(format!(
          "Model '{}' is not configured for Anthropic Messages API format. Configure an alias with 'anthropic', 'anthropic_oauth', or 'llm_liberty_oauth' format.",
          model
        ))
        .into(),
      );
    }
  };

  if api_alias.api_format == ApiFormat::LlmLibertyOauth {
    let creds = crate::providers::resolve_llm_liberty_credentials(auth_scope, &api_alias.id)
      .await
      .map_err(BodhiErrorResponse::from)?;
    // Forward-compat guard: an envelope from a future llm-liberty provider
    // (openai-codex, google-gemini, …) must not be routed through Anthropic's
    // upstream. Reject loudly rather than send a malformed request.
    if creds.provider != "anthropic" {
      return Err(
        OAIRouteError::InvalidRequest(format!(
          "Alias '{}' uses llm_liberty provider '{}'; only 'anthropic' is supported on this route.",
          api_alias.id, creds.provider
        ))
        .into(),
      );
    }
    let LlmLibertyRequestParts {
      access_token,
      base_url,
      extra_headers,
      extra_body,
    } = creds.into_request_parts();
    api_alias.base_url = base_url;
    api_alias.extra_headers = extra_headers;
    api_alias.extra_body = extra_body;
    return Ok((api_alias, access_token));
  }

  let api_key = crate::providers::resolve_api_key_for_alias(auth_scope, &api_alias.id).await;
  Ok((api_alias, api_key))
}

async fn list_user_anthropic_aliases(
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
        Alias::Api(api_alias)
          if matches!(
            api_alias.api_format,
            ApiFormat::Anthropic | ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth
          ) =>
        {
          Some(api_alias)
        }
        _ => None,
      })
      .collect(),
  )
}

/// Opaque pass-through proxy — only `model` is read; all other fields forwarded verbatim.
/// OpenAPI docs live in `resources/openapi-anthropic.json`, not utoipa annotations.
pub async fn anthropic_messages_create_handler(
  auth_scope: AuthScope,
  headers: HeaderMap,
  Query(query_params): Query<HashMap<String, String>>,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
) -> Result<Response, AnthropicApiError> {
  let model = request
    .get("model")
    .and_then(|v| v.as_str())
    .ok_or_else(AnthropicApiError::missing_model)?
    .to_string();

  let (api_alias, api_key) = resolve_anthropic_alias(&auth_scope, &model).await?;
  let client_headers = extract_anthropic_headers(&headers);
  let params: Vec<(String, String)> = query_params.into_iter().collect();
  let params_opt = if params.is_empty() {
    None
  } else {
    Some(params)
  };

  // For LlmLibertyOauth aliases the upstream may invalidate access tokens before
  // expires_at (third-party usage flagging). Retry once with a force-refreshed
  // token on a 401. Clone the request body+headers so the retry has fresh inputs.
  let is_llm_liberty = api_alias.api_format == ApiFormat::LlmLibertyOauth;
  let retry_inputs =
    is_llm_liberty.then(|| (request.clone(), params_opt.clone(), client_headers.clone()));

  let response = auth_scope
    .inference()
    .forward_remote_with_params(
      LlmEndpoint::AnthropicMessages,
      request,
      &api_alias,
      api_key,
      params_opt,
      client_headers,
    )
    .await
    .map_err(BodhiErrorResponse::from)?;

  if response.status() == StatusCode::UNAUTHORIZED {
    if let Some((retry_request, retry_params, retry_headers)) = retry_inputs {
      let creds = crate::providers::resolve_llm_liberty_credentials_with_force_refresh(
        &auth_scope,
        &api_alias.id,
      )
      .await
      .map_err(BodhiErrorResponse::from)?;
      let LlmLibertyRequestParts {
        access_token,
        base_url,
        extra_headers,
        extra_body,
      } = creds.into_request_parts();
      let mut retry_alias = api_alias.clone();
      retry_alias.base_url = base_url;
      retry_alias.extra_headers = extra_headers;
      retry_alias.extra_body = extra_body;
      return auth_scope
        .inference()
        .forward_remote_with_params(
          LlmEndpoint::AnthropicMessages,
          retry_request,
          &retry_alias,
          access_token,
          retry_params,
          retry_headers,
        )
        .await
        .map_err(|e| BodhiErrorResponse::from(e).into());
    }
  }

  Ok(response)
}

/// Aggregates models from all Anthropic-format aliases, returning full metadata.
/// Models are served from the locally cached metadata — no upstream calls.
pub async fn anthropic_models_list_handler(
  auth_scope: AuthScope,
) -> Result<Json<serde_json::Value>, AnthropicApiError> {
  let aliases = list_user_anthropic_aliases(&auth_scope).await?;

  let mut seen: HashSet<String> = HashSet::new();
  let mut ordered: Vec<serde_json::Value> = Vec::new();

  for alias in &aliases {
    let prefix = alias.prefix.as_deref().unwrap_or("");
    for model in alias.models.iter() {
      match model {
        ApiModel::Anthropic(m) => {
          let aliased_id = format!("{}{}", prefix, m.id);
          if seen.insert(aliased_id.clone()) {
            let mut entry = serde_json::to_value(m).unwrap_or_default();
            entry["id"] = serde_json::json!(aliased_id);
            ordered.push(entry);
          }
        }
        ApiModel::OpenAI(_) | ApiModel::Gemini(_) => {} // skip — wrong format for anthropic endpoint
      }
    }
  }

  let first_id = ordered
    .first()
    .and_then(|v| v.get("id"))
    .and_then(|v| v.as_str())
    .map(String::from);
  let last_id = ordered
    .last()
    .and_then(|v| v.get("id"))
    .and_then(|v| v.as_str())
    .map(String::from);

  Ok(Json(serde_json::json!({
    "data": ordered,
    "first_id": first_id,
    "last_id": last_id,
    "has_more": false,
  })))
}

/// Returns a single model's metadata from locally cached data, consistent with the list handler.
pub async fn anthropic_models_get_handler(
  auth_scope: AuthScope,
  Path(model_id): Path<String>,
) -> Result<Json<serde_json::Value>, AnthropicApiError> {
  validate_model_id(&model_id)?;

  let aliases = list_user_anthropic_aliases(&auth_scope).await?;

  for alias in &aliases {
    let prefix = alias.prefix.as_deref().unwrap_or("");
    for model in alias.models.iter() {
      if let ApiModel::Anthropic(m) = model {
        let aliased_id = format!("{}{}", prefix, m.id);
        if aliased_id == model_id {
          let mut entry = serde_json::to_value(m).unwrap_or_default();
          entry["id"] = serde_json::json!(aliased_id);
          return Ok(Json(entry));
        }
      }
    }
  }

  Err(AnthropicApiError::not_found(&model_id))
}

#[cfg(test)]
#[path = "test_anthropic_messages.rs"]
mod test_anthropic_messages;

#[cfg(test)]
#[path = "test_anthropic_models.rs"]
mod test_anthropic_models;

#[cfg(test)]
#[path = "test_anthropic_oauth_routing.rs"]
mod test_anthropic_oauth_routing;
