use super::{AiApiClientFactory, DefaultAiApiClientFactory, LibertySource};
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::models::{Alias, ApiAlias, ApiFormat};
use crate::test_utils::{
  fixed_dt, test_llm_liberty_envelope, test_llm_liberty_envelope_codex,
  test_resolved_llm_liberty_credentials, test_resolved_llm_liberty_credentials_codex,
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

fn make_codex_creds() -> ResolvedLlmLibertyCredentials {
  let mut creds = test_resolved_llm_liberty_credentials_codex();
  creds.api_models_url = None;
  creds
}

fn resolved_source<'a>(
  creds: &'a ResolvedLlmLibertyCredentials,
  alias: &'a ApiAlias,
) -> LibertySource<'a> {
  LibertySource::Resolved {
    creds,
    alias_id: &alias.id,
    prefix: alias.prefix.clone(),
    tenant_id: "tenant-a",
    user_id: "user-a",
  }
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
  let result = service.for_alias(&Alias::Api(alias.clone()), Some("key".to_string()));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_alias_returns_error_for_llm_liberty_oauth() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let err = match service.for_alias(&Alias::Api(alias.clone()), None) {
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
fn for_liberty_envelope_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let envelope = make_envelope("anthropic");
  let result = service.for_liberty(LibertySource::Envelope(&envelope));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_liberty_envelope_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let envelope = make_envelope("google-gemini");
  let err = match service.for_liberty(LibertySource::Envelope(&envelope)) {
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

#[rstest]
#[anyhow_trace]
fn for_liberty_envelope_succeeds_for_openai_codex_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let mut envelope = test_llm_liberty_envelope_codex();
  envelope.api.models_url = None;
  let result = service.for_liberty(LibertySource::Envelope(&envelope));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_liberty_resolved_succeeds_for_openai_codex_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let creds = make_codex_creds();
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let result = service.for_liberty(resolved_source(&creds, &alias));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_liberty_resolved_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let creds = make_creds("anthropic");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let result = service.for_liberty(resolved_source(&creds, &alias));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_liberty_resolved_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiClientFactory::new()?;
  let creds = make_creds("google-gemini");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let err = match service.for_liberty(resolved_source(&creds, &alias)) {
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
