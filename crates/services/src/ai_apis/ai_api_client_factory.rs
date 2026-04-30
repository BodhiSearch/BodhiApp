use super::ai_api_client::AiApiClient;
use super::clients::anthropic::AnthropicClient;
use super::clients::anthropic_oauth::AnthropicOauthClient;
use super::clients::gemini::GeminiClient;
use super::clients::liberty_anthropic::LibertyAnthropicClient;
use super::clients::liberty_codex::LibertyCodexClient;
use super::clients::openai::OpenAiClient;
use super::clients::openai_responses::OpenAiResponsesClient;
use super::error::{AiApiClientFactoryError, Result};
use super::llm_liberty::{DefaultLlmLibertyRefresh, LlmLibertyRefresh};
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::{ApiAlias, ApiFormat};
use crate::SafeReqwest;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiClientFactory: Send + Sync + std::fmt::Debug {
  fn for_alias(&self, alias: &ApiAlias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>>;

  fn for_envelope(&self, envelope: &LlmLibertyEnvelope) -> Result<Box<dyn AiApiClient>>;

  fn for_resolved_credentials(
    &self,
    creds: &ResolvedLlmLibertyCredentials,
    alias: &ApiAlias,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Box<dyn AiApiClient>>;

  fn safe_http_client(&self) -> SafeReqwest;
}

#[derive(Debug, Clone)]
pub struct DefaultAiApiClientFactory {
  client: SafeReqwest,
  refresh: Arc<dyn LlmLibertyRefresh>,
}

impl DefaultAiApiClientFactory {
  pub fn with_db(db: Arc<dyn crate::db::DbService>) -> Result<Self> {
    let client = SafeReqwest::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .allow_private_ips()
      .build()?;
    let refresh = Arc::new(DefaultLlmLibertyRefresh::new(db, client.clone()));
    Ok(Self { client, refresh })
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn new() -> Result<Self> {
    use crate::ai_apis::clients::liberty_anthropic::NoOpRefresh;
    let client = SafeReqwest::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .allow_private_ips()
      .build()?;
    Ok(Self {
      client,
      refresh: Arc::new(NoOpRefresh),
    })
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn new_with_refresh(client: SafeReqwest, refresh: Arc<dyn LlmLibertyRefresh>) -> Self {
    Self { client, refresh }
  }
}

#[async_trait]
impl AiApiClientFactory for DefaultAiApiClientFactory {
  fn safe_http_client(&self) -> SafeReqwest {
    self.client.clone()
  }

  fn for_alias(&self, alias: &ApiAlias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>> {
    let prefix = alias.prefix.clone();
    let client: Box<dyn AiApiClient> = match alias.api_format {
      ApiFormat::OpenAI => Box::new(OpenAiClient::new(
        api_key,
        alias.base_url.clone(),
        prefix,
        self.client.clone(),
      )),
      ApiFormat::OpenAIResponses => Box::new(OpenAiResponsesClient::new(
        api_key,
        alias.base_url.clone(),
        prefix,
        self.client.clone(),
      )),
      ApiFormat::Anthropic => Box::new(AnthropicClient::new(
        api_key,
        alias.base_url.clone(),
        prefix,
        self.client.clone(),
      )),
      ApiFormat::AnthropicOAuth => Box::new(AnthropicOauthClient::new(
        api_key,
        alias.base_url.clone(),
        prefix,
        alias.extra_headers.clone(),
        alias.extra_body.clone(),
        self.client.clone(),
      )),
      ApiFormat::Gemini => Box::new(GeminiClient::new(
        api_key,
        alias.base_url.clone(),
        prefix,
        self.client.clone(),
      )),
      ApiFormat::LlmLibertyOauth => {
        return Err(AiApiClientFactoryError::LibertyRequiresCredentials)
      }
    };
    Ok(client)
  }

  fn for_envelope(&self, envelope: &LlmLibertyEnvelope) -> Result<Box<dyn AiApiClient>> {
    match envelope.provider.as_str() {
      "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_envelope(
        envelope,
        self.client.clone(),
      ))),
      "openai-codex" => Ok(Box::new(LibertyCodexClient::from_envelope(
        envelope,
        self.client.clone(),
      ))),
      other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(
        other.to_string(),
      )),
    }
  }

  fn for_resolved_credentials(
    &self,
    creds: &ResolvedLlmLibertyCredentials,
    alias: &ApiAlias,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Box<dyn AiApiClient>> {
    match creds.provider.as_str() {
      "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_credentials(
        creds,
        &alias.id,
        alias.prefix.clone(),
        tenant_id,
        user_id,
        self.refresh.clone(),
        self.client.clone(),
      ))),
      "openai-codex" => Ok(Box::new(LibertyCodexClient::from_credentials(
        creds,
        &alias.id,
        alias.prefix.clone(),
        tenant_id,
        user_id,
        self.refresh.clone(),
        self.client.clone(),
      ))),
      other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(
        other.to_string(),
      )),
    }
  }
}

#[cfg(test)]
#[path = "test_ai_api_anthropic.rs"]
mod test_ai_api_anthropic;
#[cfg(test)]
#[path = "test_ai_api_anthropic_oauth.rs"]
mod test_ai_api_anthropic_oauth;
#[cfg(test)]
#[path = "test_ai_api_forward.rs"]
mod test_ai_api_forward;
#[cfg(test)]
#[path = "test_ai_api_gemini.rs"]
mod test_ai_api_gemini;
#[cfg(test)]
#[path = "test_ai_api_openai.rs"]
mod test_ai_api_openai;
#[cfg(test)]
#[path = "test_ai_api_provider_matrix.rs"]
mod test_ai_api_provider_matrix;
#[cfg(test)]
#[path = "test_factory.rs"]
mod test_factory;
#[cfg(test)]
#[path = "test_merge_extra_body.rs"]
mod test_merge_extra_body;
