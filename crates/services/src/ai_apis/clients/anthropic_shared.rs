/// Shared helpers for Anthropic-protocol clients.
///
/// Used by both `AnthropicOauthClient` (setup-token flow) and
/// `LibertyAnthropicClient` (envelope-explicit-URL flow).
use crate::models::{AnthropicModel, ApiModel};
use serde_json::Value;

/// Inject `Authorization: Bearer <token>` plus any extra headers from the
/// envelope. If neither the client headers nor the extra headers already set
/// `anthropic-version`, add the default `2023-06-01`.
pub(super) fn apply_bearer_auth_and_version(
  mut request: reqwest::RequestBuilder,
  access_token: Option<&str>,
  extra_headers: Option<&Value>,
  client_headers: Option<&[(String, String)]>,
) -> reqwest::RequestBuilder {
  if let Some(token) = access_token {
    request = request.header("Authorization", format!("Bearer {}", token));
  }

  if let Some(Value::Object(ref map)) = extra_headers {
    for (k, v) in map {
      if let Some(v_str) = v.as_str() {
        request = request.header(k.as_str(), v_str);
      }
    }
  }

  let client_has_version = client_headers
    .map(|hdrs| {
      hdrs
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("anthropic-version"))
    })
    .unwrap_or(false);
  let extra_has_version = extra_headers
    .as_ref()
    .and_then(|v| v.as_object())
    .map(|m| {
      m.keys()
        .any(|k| k.eq_ignore_ascii_case("anthropic-version"))
    })
    .unwrap_or(false);
  if !client_has_version && !extra_has_version {
    request = request.header("anthropic-version", "2023-06-01");
  }

  request
}

/// Deserialize `data[]` from an Anthropic models response page into `ApiModel::Anthropic`.
pub(super) fn parse_anthropic_models_page(body: &Value) -> Vec<ApiModel> {
  body
    .get("data")
    .and_then(|d| d.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| serde_json::from_value::<AnthropicModel>(v.clone()).ok())
        .map(ApiModel::Anthropic)
        .collect()
    })
    .unwrap_or_default()
}

/// Extract the text response from an Anthropic `/messages` response.
pub(super) fn extract_anthropic_completion_text(body: &Value) -> String {
  body
    .get("content")
    .and_then(|c| c.as_array())
    .and_then(|arr| arr.first())
    .and_then(|b| b.get("text"))
    .and_then(|t| t.as_str())
    .unwrap_or("No response")
    .to_string()
}
