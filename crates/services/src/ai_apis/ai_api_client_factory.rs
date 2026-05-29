use super::ai_api_client::AiApiClient;
use super::clients::anthropic::AnthropicClient;
use super::clients::anthropic_oauth::AnthropicOauthClient;
use super::clients::gemini::GeminiClient;
use super::clients::liberty_anthropic::LibertyAnthropicClient;
use super::clients::liberty_codex::LibertyCodexClient;
use super::clients::local_llama::LocalLlamaClient;
use super::clients::openai::OpenAiClient;
use super::clients::openai_responses::OpenAiResponsesClient;
use super::error::{AiApiClientFactoryError, Result};
use super::llm_liberty::{DefaultLlmLibertyRefresh, LlmLibertyRefresh};
use crate::inference::LocalLlama;
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::{Alias, ApiFormat};
use crate::SafeReqwest;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Discriminator for `AiApiClientFactory::for_liberty` — the two variants
/// reflect Liberty's distinct credential lifecycle stages and produce clients
/// with different 401-retry behavior.
pub enum LibertySource<'a> {
  /// Pre-persistence validation: stateless client, `NoOpRefresh`, 401 surfaces
  /// directly (no DB call). Used by API-model creation/test flows before the
  /// envelope is encrypted and persisted.
  Envelope(&'a LlmLibertyEnvelope),
  /// Per-request: stateful client, `DefaultLlmLibertyRefresh`. On UNAUTHORIZED
  /// the client calls `force_refresh(tenant_id, user_id, alias_id)` and retries
  /// the request once with the rotated token.
  Resolved {
    creds: &'a ResolvedLlmLibertyCredentials,
    alias_id: &'a str,
    prefix: Option<String>,
    tenant_id: &'a str,
    user_id: &'a str,
  },
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiClientFactory: Send + Sync + std::fmt::Debug {
  /// Per-request client for non-Liberty alias flows.
  /// - `Alias::User`/`Alias::Model` → `LocalLlamaClient` (or `LocalNotSupportedInCluster` when no local runtime is wired)
  /// - `Alias::Api` with key-based formats → corresponding remote client with `api_key`
  /// - `Alias::Api` with `LlmLibertyOauth` format → `LibertyRequiresCredentials` (caller must use `for_liberty`)
  fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>>;

  /// Per-request client for an `LlmLibertyOauth` source. See `LibertySource` doc for the two lifecycle stages.
  fn for_liberty<'a>(&self, source: LibertySource<'a>) -> Result<Box<dyn AiApiClient>>;

  /// Shared `SafeReqwest` for OAuth refresh paths that reuse the same connection
  /// pool as model traffic. Justified by Arc-shared pooling; not strictly an
  /// AI-client concern but the cheapest place to expose it.
  fn safe_http_client(&self) -> SafeReqwest;
}

#[derive(Debug, Clone)]
pub struct DefaultAiApiClientFactory {
  client: SafeReqwest,
  refresh: Arc<dyn LlmLibertyRefresh>,
  local_llama: Option<Arc<dyn LocalLlama>>,
}

impl DefaultAiApiClientFactory {
  pub fn with_db(
    db: Arc<dyn crate::db::DbService>,
    local_llama: Option<Arc<dyn LocalLlama>>,
  ) -> Result<Self> {
    let client = SafeReqwest::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .allow_private_ips()
      .build()?;
    let refresh = Arc::new(DefaultLlmLibertyRefresh::new(db, client.clone()));
    Ok(Self {
      client,
      refresh,
      local_llama,
    })
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
      local_llama: None,
    })
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn new_with_refresh(client: SafeReqwest, refresh: Arc<dyn LlmLibertyRefresh>) -> Self {
    Self {
      client,
      refresh,
      local_llama: None,
    }
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn with_local_llama(mut self, local_llama: Arc<dyn LocalLlama>) -> Self {
    self.local_llama = Some(local_llama);
    self
  }
}

#[async_trait]
impl AiApiClientFactory for DefaultAiApiClientFactory {
  fn safe_http_client(&self) -> SafeReqwest {
    self.client.clone()
  }

  fn for_alias(&self, alias: &Alias, api_key: Option<String>) -> Result<Box<dyn AiApiClient>> {
    match alias {
      Alias::User(_) | Alias::Model(_) => match &self.local_llama {
        Some(rt) => Ok(Box::new(LocalLlamaClient::new(rt.clone(), alias.clone()))),
        None => Err(AiApiClientFactoryError::LocalNotSupportedInCluster),
      },
      Alias::Api(api_alias) => {
        let prefix = api_alias.prefix.clone();
        let client: Box<dyn AiApiClient> = match api_alias.api_format {
          ApiFormat::OpenAI => Box::new(OpenAiClient::new(
            api_key,
            api_alias.base_url.clone(),
            prefix,
            self.client.clone(),
          )),
          ApiFormat::OpenAIResponses => Box::new(OpenAiResponsesClient::new(
            api_key,
            api_alias.base_url.clone(),
            prefix,
            self.client.clone(),
          )),
          ApiFormat::Anthropic => Box::new(AnthropicClient::new(
            api_key,
            api_alias.base_url.clone(),
            prefix,
            self.client.clone(),
          )),
          ApiFormat::AnthropicOAuth => Box::new(AnthropicOauthClient::new(
            api_key,
            api_alias.base_url.clone(),
            prefix,
            api_alias.extra_headers.clone(),
            api_alias.extra_body.clone(),
            self.client.clone(),
          )),
          ApiFormat::Gemini => Box::new(GeminiClient::new(
            api_key,
            api_alias.base_url.clone(),
            prefix,
            self.client.clone(),
          )),
          ApiFormat::LlmLibertyOauth => {
            return Err(AiApiClientFactoryError::LibertyRequiresCredentials)
          }
        };
        Ok(client)
      }
      Alias::ModelRouter(_) => Err(AiApiClientFactoryError::ModelRouterNotForwardable),
    }
  }

  fn for_liberty<'a>(&self, source: LibertySource<'a>) -> Result<Box<dyn AiApiClient>> {
    match source {
      LibertySource::Envelope(envelope) => match envelope.provider.as_str() {
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
      },
      LibertySource::Resolved {
        creds,
        alias_id,
        prefix,
        tenant_id,
        user_id,
      } => match creds.provider.as_str() {
        "anthropic" => Ok(Box::new(LibertyAnthropicClient::from_credentials(
          creds,
          alias_id,
          prefix,
          tenant_id,
          user_id,
          self.refresh.clone(),
          self.client.clone(),
        ))),
        "openai-codex" => Ok(Box::new(LibertyCodexClient::from_credentials(
          creds,
          alias_id,
          prefix,
          tenant_id,
          user_id,
          self.refresh.clone(),
          self.client.clone(),
        ))),
        other => Err(AiApiClientFactoryError::LibertyProviderUnsupported(
          other.to_string(),
        )),
      },
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
