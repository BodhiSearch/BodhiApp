use crate::models::llm_liberty_envelope::{
  LlmLibertyApiEndpoints, LlmLibertyAuthSpec, LlmLibertyEnvelope, LlmLibertyOauthEndpoints,
  ResolvedLlmLibertyCredentials,
};
use chrono::{Duration, Utc};

/// Builds a well-formed `LlmLibertyEnvelope` for the `anthropic` provider with
/// `expires_at` 1 hour in the future. Tests mutate fields after construction
/// to override URLs, tokens, or provider as needed.
pub fn test_llm_liberty_envelope() -> LlmLibertyEnvelope {
  LlmLibertyEnvelope {
    version: "1.0.0".into(),
    provider: "anthropic".into(),
    access_token: "test-access-token".into(),
    refresh_token: "test-refresh-token".into(),
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
      client_id: "client-id".into(),
      client_secret: None,
    },
    api: LlmLibertyApiEndpoints {
      base_url: "https://api.example.com".into(),
      chat_url: "https://api.example.com/v1/messages".into(),
      models_url: Some("https://api.example.com/v1/models".into()),
    },
    headers: serde_json::json!({}),
    body: serde_json::json!({}),
    extra: None,
  }
}

pub fn test_llm_liberty_envelope_codex() -> LlmLibertyEnvelope {
  LlmLibertyEnvelope {
    version: "1.0.0".into(),
    provider: "openai-codex".into(),
    access_token: "test-access-token".into(),
    refresh_token: "test-refresh-token".into(),
    expires_at: (Utc::now() + Duration::hours(1)).timestamp(),
    auth: LlmLibertyAuthSpec {
      location: "header".into(),
      key: "Authorization".into(),
      scheme: "Bearer".into(),
    },
    oauth: LlmLibertyOauthEndpoints {
      authorize_url: "https://auth.example/authorize".into(),
      token_url: "https://auth.example/token".into(),
      revoke_url: None,
      client_id: "client-id".into(),
      client_secret: None,
    },
    api: LlmLibertyApiEndpoints {
      base_url: "https://api.example.com/codex".into(),
      chat_url: "https://api.example.com/codex/responses".into(),
      models_url: Some("https://api.example.com/codex/models".into()),
    },
    headers: serde_json::json!({
      "ChatGPT-Account-ID": "test-account-id",
      "originator": "codex_cli_rs",
      "User-Agent": "codex_cli_rs/0.0.1",
      "OpenAI-Beta": "responses=experimental"
    }),
    body: serde_json::json!({
      "instructions": "You are Codex, OpenAI's coding agent.",
      "store": false,
      "stream": true
    }),
    extra: None,
  }
}

/// Builds a `ResolvedLlmLibertyCredentials` for the `anthropic` provider.
/// Tests mutate fields after construction to override tokens, URLs, or provider.
pub fn test_resolved_llm_liberty_credentials() -> ResolvedLlmLibertyCredentials {
  ResolvedLlmLibertyCredentials {
    access_token: "test-access-token".to_string(),
    refresh_token: "test-refresh-token".to_string(),
    expires_at: Utc::now() + Duration::hours(1),
    tenant_id: "tenant-a".to_string(),
    provider: "anthropic".to_string(),
    auth_scheme: "Bearer".to_string(),
    auth_key: "Authorization".to_string(),
    oauth_token_url: "https://oauth.example/token".to_string(),
    oauth_client_id: "client-id".to_string(),
    oauth_client_secret: None,
    api_base_url: "https://api.example.com".to_string(),
    api_chat_url: "https://api.example.com/v1/messages".to_string(),
    api_models_url: Some("https://api.example.com/v1/models".to_string()),
    headers_json: serde_json::json!({}),
    body_json: serde_json::json!({}),
    extra_json: None,
  }
}

pub fn test_resolved_llm_liberty_credentials_codex() -> ResolvedLlmLibertyCredentials {
  ResolvedLlmLibertyCredentials {
    access_token: "test-access-token".to_string(),
    refresh_token: "test-refresh-token".to_string(),
    expires_at: Utc::now() + Duration::hours(1),
    tenant_id: "tenant-a".to_string(),
    provider: "openai-codex".to_string(),
    auth_scheme: "Bearer".to_string(),
    auth_key: "Authorization".to_string(),
    oauth_token_url: "https://auth.example/token".to_string(),
    oauth_client_id: "client-id".to_string(),
    oauth_client_secret: None,
    api_base_url: "https://api.example.com/codex".to_string(),
    api_chat_url: "https://api.example.com/codex/responses".to_string(),
    api_models_url: Some("https://api.example.com/codex/models".to_string()),
    headers_json: serde_json::json!({
      "ChatGPT-Account-ID": "test-account-id",
      "originator": "codex_cli_rs",
      "User-Agent": "codex_cli_rs/0.0.1",
      "OpenAI-Beta": "responses=experimental"
    }),
    body_json: serde_json::json!({
      "instructions": "You are Codex, OpenAI's coding agent.",
      "store": false,
      "stream": true
    }),
    extra_json: None,
  }
}
