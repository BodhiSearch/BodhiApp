use super::anthropic_shared::{
  apply_bearer_auth_and_version, extract_anthropic_completion_text, parse_anthropic_models_page,
};
use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::llm_liberty::{LlmLibertyRefresh, LlmLibertyRefreshError};
use crate::ai_apis::provider_shared::{forward_to_upstream, merge_extra_body};
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub(crate) struct LibertyAnthropicClient {
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

impl LibertyAnthropicClient {
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
    request: reqwest::RequestBuilder,
    client_headers: Option<&[(String, String)]>,
  ) -> reqwest::RequestBuilder {
    let token = self.access_token.lock().expect("liberty client token lock");
    apply_bearer_auth_and_version(
      request,
      Some(token.as_str()),
      self.extra_headers.as_ref(),
      client_headers,
    )
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
    let is_messages = api_path == "/messages";
    let merged_request = match (request, &self.extra_body) {
      (Some(body), Some(extra)) if is_messages => Some(merge_extra_body(body, extra)),
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
impl AiApiClient for LibertyAnthropicClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
    let base_body = serde_json::json!({
      "model": model,
      "max_tokens": 50,
      "messages": [{"role": "user", "content": prompt}]
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
    let body: Value = response.json().await?;
    Ok(extract_anthropic_completion_text(&body))
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
    let Some(ref models_url) = self.models_url else {
      return Ok(Vec::new());
    };
    let mut all_models: Vec<ApiModel> = Vec::new();
    let mut before_id: Option<String> = None;
    loop {
      let url = match &before_id {
        Some(bid) => format!("{}?before_id={}", models_url, bid),
        None => models_url.clone(),
      };
      let mut request = self.client.get(&url)?;
      request = self.apply_auth(request, None);
      let response = request.send().await?;
      let status = response.status();
      if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AiApiClientFactoryError::status_to_error(status, body));
      }
      let body: Value = response.json().await?;
      let page_models = parse_anthropic_models_page(&body);
      before_id = page_models.last().map(|m| m.id().to_string());
      all_models.extend(page_models);
      if !body
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
      {
        break;
      }
    }
    Ok(all_models)
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
      let mut token = self.access_token.lock().expect("liberty client token lock");
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

/// LibertyAnthropic only proxies `/messages` and `/models*`; any other path is
/// rejected with `NotFound` to fail fast on misrouted callers.
fn resolve_url(api_path: &str, chat_url: &str, models_url: Option<&str>) -> Result<String> {
  if api_path == "/messages" || api_path.starts_with("/messages?") {
    return Ok(chat_url.to_string());
  }
  if api_path.starts_with("/models") {
    if let Some(mu) = models_url {
      return Ok(mu.to_string());
    }
    return Err(AiApiClientFactoryError::NotFound(api_path.to_string()));
  }
  Err(AiApiClientFactoryError::NotFound(api_path.to_string()))
}

#[derive(Debug)]
pub(crate) struct NoOpRefresh;

#[async_trait]
impl LlmLibertyRefresh for NoOpRefresh {
  async fn force_refresh(
    &self,
    _tenant_id: &str,
    _user_id: &str,
    _alias_id: &str,
  ) -> std::result::Result<ResolvedLlmLibertyCredentials, LlmLibertyRefreshError> {
    Err(LlmLibertyRefreshError::NotFound("no-op".to_string()))
  }
}
