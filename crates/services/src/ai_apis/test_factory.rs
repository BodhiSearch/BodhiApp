use super::{AiApiService, DefaultAiApiService};
use crate::models::llm_liberty_envelope::{
  LlmLibertyApiEndpoints, LlmLibertyAuthSpec, LlmLibertyEnvelope, LlmLibertyOauthEndpoints,
  ResolvedLlmLibertyCredentials,
};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::fixed_dt;
use anyhow_trace::anyhow_trace;
use chrono::{Duration, Utc};
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
  LlmLibertyEnvelope {
    version: "1.0.0".into(),
    provider: provider.to_string(),
    access_token: "access".into(),
    refresh_token: "refresh".into(),
    expires_at: (Utc::now() + Duration::hours(1)).timestamp(),
    auth: LlmLibertyAuthSpec {
      location: "header".into(),
      key: "Authorization".into(),
      scheme: "Bearer".into(),
    },
    oauth: LlmLibertyOauthEndpoints {
      authorize_url: "https://oauth.example/authorize".into(),
      token_url: "https://oauth.example/token".into(),
      revoke_url: None,
      client_id: "client".into(),
      client_secret: None,
    },
    api: LlmLibertyApiEndpoints {
      base_url: "https://api.example.com".into(),
      chat_url: "https://api.example.com/v1/messages".into(),
      models_url: None,
    },
    headers: serde_json::json!({}),
    body: serde_json::json!({}),
    extra: None,
  }
}

fn make_creds(provider: &str) -> ResolvedLlmLibertyCredentials {
  ResolvedLlmLibertyCredentials {
    access_token: "access".to_string(),
    refresh_token: "refresh".to_string(),
    expires_at: Utc::now() + Duration::hours(1),
    tenant_id: "tenant-a".to_string(),
    provider: provider.to_string(),
    auth_scheme: "Bearer".to_string(),
    auth_key: "Authorization".to_string(),
    oauth_token_url: "https://oauth.example/token".to_string(),
    oauth_client_id: "client".to_string(),
    oauth_client_secret: None,
    api_base_url: "https://api.example.com/v1".to_string(),
    api_chat_url: "https://api.example.com/v1/messages".to_string(),
    api_models_url: None,
    headers_json: serde_json::json!({}),
    body_json: serde_json::json!({}),
    extra_json: None,
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
  let service = DefaultAiApiService::new()?;
  let alias = make_alias(format);
  let result = service.for_alias(&alias, Some("key".to_string()));
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_alias_returns_error_for_llm_liberty_oauth() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  match service.for_alias(&alias, None) {
    Err(e) => {
      let msg = e.to_string();
      assert!(
        msg.contains("LlmLibertyOauth"),
        "expected LlmLibertyOauth in error: {msg}"
      );
    }
    Ok(_) => panic!("expected error for LlmLibertyOauth"),
  }
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_envelope_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let envelope = make_envelope("anthropic");
  let result = service.for_envelope(&envelope);
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_envelope_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let envelope = make_envelope("openai-codex");
  match service.for_envelope(&envelope) {
    Err(e) => {
      let msg = e.to_string();
      assert!(
        msg.contains("openai-codex"),
        "expected provider name in error: {msg}"
      );
    }
    Ok(_) => panic!("expected error for unsupported provider"),
  }
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_resolved_credentials_succeeds_for_anthropic_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let creds = make_creds("anthropic");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  let result = service.for_resolved_credentials(&creds, &alias, "tenant-a", "user-a");
  assert!(result.is_ok());
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn for_resolved_credentials_returns_error_for_unsupported_provider() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let creds = make_creds("google-gemini");
  let alias = make_alias(ApiFormat::LlmLibertyOauth);
  match service.for_resolved_credentials(&creds, &alias, "tenant-a", "user-a") {
    Err(e) => {
      let msg = e.to_string();
      assert!(
        msg.contains("google-gemini"),
        "expected provider name in error: {msg}"
      );
    }
    Ok(_) => panic!("expected error for unsupported provider"),
  }
  Ok(())
}
