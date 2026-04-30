use super::{AiApiClientFactory, DefaultAiApiClientFactory};
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::{
  fixed_dt, test_llm_liberty_envelope, test_resolved_llm_liberty_credentials,
};
use anyhow_trace::anyhow_trace;
use errmeta::AppError;
use rstest::rstest;

fn make_alias(format: ApiFormat) -> ApiAlias {
  ApiAlias::new(
    "test-alias".to_string(),
    format,
    "https://api.example.com/v1",
    vec![],
    None,
    false,
    fixed_dt(),
    None,
    None,
  )
}

fn make_envelope(provider: &str) -> LlmLibertyEnvelope {
  let mut env = test_llm_liberty_envelope();
  env.provider = provider.to_string();
  env.api.models_url = None;
  env
}

fn make_creds(provider: &str) -> ResolvedLlmLibertyCredentials {
  let mut creds = test_resolved_llm_liberty_credentials();
  creds.provider = provider.to_string();
  creds.api_base_url = "https://api.example.com/v1".to_string();
  creds.api_models_url = None;
  creds
}

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::openai_responses(ApiFormat::OpenAIResponses)]
#[case::anthropic(ApiFormat::Anthropic)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
#[anyhow_trace]
fn for_alias_succeeds_for_non_liberty_formats(#[case] format: ApiFormat) -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let alias = make_alias(format);
  let result = service.for_alias(&alias, Some("key".to_string()));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_alias_returns_error_for_llm_liberty_oauth() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let err = match service.for_alias(&alias, None) {
    Err(e) => e,
    Ok(_) => panic!("expected error for LlmLibertyOauth"),
  };
  assert_eq!(
    "ai_api_client_factory_error-liberty_requires_credentials",
    err.code()
  );
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_envelope_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let envelope = make_envelope("anthropic");
  let result = service.for_envelope(&envelope);
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_envelope_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let envelope = make_envelope("openai-codex");
  let err = match service.for_envelope(&envelope) {
    Err(e) => e,
    Ok(_) => panic!("expected error for unsupported provider"),
  };
  assert_eq!(
    "ai_api_client_factory_error-liberty_provider_unsupported",
    err.code()
  );
  let msg = err.to_string();
  assert!(
    msg.contains("openai-codex"),
    "expected provider name in error: {msg}"
  );
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_resolved_credentials_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let creds = make_creds("anthropic");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let result = service.for_resolved_credentials(&creds, &alias, "tenant-a", "user-a");
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_resolved_credentials_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let creds = make_creds("google-gemini");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let err = match service.for_resolved_credentials(&creds, &alias, "tenant-a", "user-a") {
    Err(e) => e,
    Ok(_) => panic!("expected error for unsupported provider"),
  };
  assert_eq!(
    "ai_api_client_factory_error-liberty_provider_unsupported",
    err.code()
  );
  let msg = err.to_string();
  assert!(
    msg.contains("google-gemini"),
    "expected provider name in error: {msg}"
  );
  Ok(())
}
