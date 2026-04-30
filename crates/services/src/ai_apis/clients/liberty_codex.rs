use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::clients::liberty_anthropic::NoOpRefresh;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::llm_liberty::LlmLibertyRefresh;
use crate::ai_apis::provider_shared::{forward_to_upstream, merge_extra_body};
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_openai::types::models::Model as OpenAIModel;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;
use std::sync::{Arc, Mutex};

const CODEX_CLIENT_VERSION: &str = "0.0.1";

pub(crate) struct LibertyCodexClient {
  client: SafeReqwest,
  access_token: Mutex<String>,
  chat_url: String,
  models_url: Option<String>,
  prefix: Option<String>,
  extra_headers: Option<Value>,
  extra_body: Option<Value>,
  alias_id: String,
  tenant_id: String,
  user_id: String,
  refresh: Arc<dyn LlmLibertyRefresh>,
}

impl LibertyCodexClient {
  /// Inline-envelope flow (test/fetch-models before save): no alias yet, no
  /// prefix, `NoOpRefresh` so 401-retry is skipped via the empty-alias_id guard.
  pub(crate) fn from_envelope(envelope: &LlmLibertyEnvelope, client: SafeReqwest) -> Self {
    Self {
      client,
      access_token: Mutex::new(envelope.access_token.clone()),
      chat_url: envelope.api.chat_url.clone(),
      models_url: envelope.api.models_url.clone(),
      prefix: None,
      extra_headers: value_to_opt(&envelope.headers),
      extra_body: value_to_opt(&envelope.body),
      alias_id: String::new(),
      tenant_id: String::new(),
      user_id: String::new(),
      refresh: Arc::new(NoOpRefresh),
    }
  }

  /// Saved-alias flow: owns 401-retry-with-force-refresh via the injected
  /// `LlmLibertyRefresh`; on UNAUTHORIZED rotates `access_token` in-place and retries once.
  pub(crate) fn from_credentials(
    creds: &ResolvedLlmLibertyCredentials,
    alias_id: &str,
    prefix: Option<String>,
    tenant_id: &str,
    user_id: &str,
    refresh: Arc<dyn LlmLibertyRefresh>,
    client: SafeReqwest,
  ) -> Self {
    Self {
      client,
      access_token: Mutex::new(creds.access_token.clone()),
      chat_url: creds.api_chat_url.clone(),
      models_url: creds.api_models_url.clone(),
      prefix,
      extra_headers: value_to_opt(&creds.headers_json),
      extra_body: value_to_opt(&creds.body_json),
      alias_id: alias_id.to_string(),
      tenant_id: tenant_id.to_string(),
      user_id: user_id.to_string(),
      refresh,
    }
  }

  fn apply_auth(
    &self,
    mut request: reqwest::RequestBuilder,
    _client_headers: Option<&[(String, String)]>,
  ) -> reqwest::RequestBuilder {
    let token = self
      .access_token
      .lock()
      .expect("liberty codex client token lock");
    request = request.header("Authorization", format!("Bearer {}", token.as_str()));
    if let Some(Value::Object(ref map)) = self.extra_headers {
      for (k, v) in map {
        if let Some(v_str) = v.as_str() {
          request = request.header(k.as_str(), v_str);
        }
      }
    }
    request
  }

  async fn forward_once(
    &self,
    method: &Method,
    api_path: &str,
    request: Option<Value>,
    query_params: Option<&[(String, String)]>,
    client_headers: Option<&[(String, String)]>,
  ) -> Result<Response> {
    let target_url = resolve_url(api_path, &self.chat_url, self.models_url.as_deref())?;
    let merged_request = match (request, &self.extra_body) {
      (Some(body), Some(extra)) => Some(merge_extra_body(body, extra)),
      (req, _) => req,
    };
    forward_to_upstream(
      &self.client,
      &target_url,
      method,
      "",
      self.prefix.as_deref(),
      merged_request,
      query_params,
      |rb| self.apply_auth(rb, client_headers),
      client_headers,
    )
    .await
  }
}

#[async_trait]
impl AiApiClient for LibertyCodexClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
    let base_body = serde_json::json!({
      "model": model,
      "input": [{"role": "user", "content": prompt}],
    });
    let request_body = match &self.extra_body {
      Some(extra) => merge_extra_body(base_body, extra),
      None => base_body,
    };
    let mut request = self
      .client
      .post(&self.chat_url)?
      .header("Content-Type", "application/json")
      .json(&request_body);
    request = self.apply_auth(request, None);
    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiClientFactoryError::status_to_error(status, body));
    }
    let sse_body = response.text().await.unwrap_or_default();
    Ok(extract_codex_completion_text(&sse_body))
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
    let Some(ref models_url) = self.models_url else {
      return Ok(Vec::new());
    };
    let url = format!("{}?client_version={}", models_url, CODEX_CLIENT_VERSION);
    let mut request = self.client.get(&url)?;
    request = self.apply_auth(request, None);
    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiClientFactoryError::status_to_error(status, body));
    }
    let body: Value = response.json().await?;
    Ok(parse_codex_models(&body))
  }

  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response> {
    let response = self
      .forward_once(
        method,
        api_path,
        request.clone(),
        query_params.as_deref(),
        client_headers.as_deref(),
      )
      .await?;

    if response.status() != axum::http::StatusCode::UNAUTHORIZED || self.alias_id.is_empty() {
      return Ok(response);
    }

    let new_creds = self
      .refresh
      .force_refresh(&self.tenant_id, &self.user_id, &self.alias_id)
      .await
      .map_err(|e| AiApiClientFactoryError::ApiError(e.to_string()))?;

    {
      let mut token = self
        .access_token
        .lock()
        .expect("liberty codex client token lock");
      *token = new_creds.access_token.clone();
    }

    self
      .forward_once(
        method,
        api_path,
        request,
        query_params.as_deref(),
        client_headers.as_deref(),
      )
      .await
  }
}

fn value_to_opt(v: &Value) -> Option<Value> {
  if v.is_null() {
    None
  } else {
    Some(v.clone())
  }
}

// Suffix after `/responses` appended to chat_url; `/models*` → models_url. Other paths → NotFound.
fn resolve_url(api_path: &str, chat_url: &str, models_url: Option<&str>) -> Result<String> {
  if let Some(suffix) = api_path.strip_prefix("/responses") {
    if suffix.is_empty() || suffix.starts_with('?') || suffix.starts_with('/') {
      return Ok(format!("{}{}", chat_url, suffix));
    }
  }
  if api_path.starts_with("/models") {
    if let Some(mu) = models_url {
      return Ok(mu.to_string());
    }
    return Err(AiApiClientFactoryError::NotFound(api_path.to_string()));
  }
  Err(AiApiClientFactoryError::NotFound(api_path.to_string()))
}

/// Parse SSE events from a Codex `/responses` streaming response and extract
/// the final assistant text from the `response.output_text.done` event.
fn extract_codex_completion_text(sse_body: &str) -> String {
  for line in sse_body.lines() {
    let Some(data) = line.strip_prefix("data: ") else {
      continue;
    };
    let Ok(json) = serde_json::from_str::<Value>(data) else {
      continue;
    };
    if json.get("type").and_then(|t| t.as_str()) == Some("response.output_text.done") {
      if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
        return text.to_string();
      }
    }
  }
  "No response".to_string()
}

/// Parse `{"models": [{"slug": "...", ...}]}` into `ApiModel::OpenAI` entries.
fn parse_codex_models(body: &Value) -> Vec<ApiModel> {
  body
    .get("models")
    .and_then(|m| m.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| v.get("slug").and_then(|s| s.as_str()))
        .map(|slug| {
          ApiModel::OpenAI(OpenAIModel {
            id: slug.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "openai".to_string(),
          })
        })
        .collect()
    })
    .unwrap_or_default()
}

#[cfg(test)]
#[path = "test_liberty_codex.rs"]
mod test_liberty_codex;
