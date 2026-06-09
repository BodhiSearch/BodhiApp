use crate::ObjValidationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Envelope sub-types mirror the llm-liberty JSON contract v1.0.0.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct LlmLibertyAuthSpec {
  #[serde(rename = "in")]
  pub location: String,
  pub key: String,
  pub scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct LlmLibertyOauthEndpoints {
  pub authorize_url: String,
  pub token_url: String,
  pub revoke_url: Option<String>,
  pub client_id: String,
  pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct LlmLibertyApiEndpoints {
  pub base_url: String,
  pub chat_url: String,
  pub models_url: Option<String>,
}

/// The JSON blob emitted by `npx @bodhiapp/llm-liberty@latest login`.
///
/// Version field allows BodhiApp to detect breaking changes in the envelope
/// schema (major-version bump = breaking). Currently only `"1.0.0"` is accepted.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq)]
pub struct LlmLibertyEnvelope {
  /// Envelope schema version — must be "1.0.0".
  pub version: String,
  /// Provider identifier, e.g. "anthropic" or "openai-codex".
  pub provider: String,
  pub access_token: String,
  pub refresh_token: String,
  /// Unix epoch seconds when the access token expires.
  pub expires_at: i64,
  pub auth: LlmLibertyAuthSpec,
  pub oauth: LlmLibertyOauthEndpoints,
  pub api: LlmLibertyApiEndpoints,
  #[serde(default = "empty_json_object")]
  pub headers: serde_json::Value,
  #[serde(default = "empty_json_object")]
  pub body: serde_json::Value,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub extra: Option<serde_json::Value>,
}

fn empty_json_object() -> serde_json::Value {
  serde_json::json!({})
}

impl LlmLibertyEnvelope {
  /// Validate that the envelope is usable: known version and supported provider.
  pub fn validate_supported(&self) -> Result<(), ObjValidationError> {
    let reason = if self.version != "1.0.0" {
      format!(
        "Unsupported llm-liberty envelope version '{}'. Expected '1.0.0'.",
        self.version
      )
    } else if !matches!(self.provider.as_str(), "anthropic" | "openai-codex") {
      format!(
        "Unsupported provider '{}'. Only 'anthropic' and 'openai-codex' are supported in this version.",
        self.provider
      )
    } else if self.access_token.is_empty() {
      "access_token is required.".into()
    } else if self.refresh_token.is_empty() {
      "refresh_token is required.".into()
    } else if self.oauth.token_url.is_empty() {
      "oauth.token_url is required.".into()
    } else if self.oauth.client_id.is_empty() {
      "oauth.client_id is required.".into()
    } else if self.api.chat_url.is_empty() {
      "api.chat_url is required.".into()
    } else if self.auth.location != "header" {
      // v1 only supports header-based auth (Bearer); reject query/body schemes loudly.
      format!(
        "Unsupported auth.in '{}'. Only 'header' is supported in this version.",
        self.auth.location
      )
    } else if self.auth.key != "Authorization" {
      format!(
        "Unsupported auth.key '{}'. Only 'Authorization' is supported in this version.",
        self.auth.key
      )
    } else if self.auth.scheme != "Bearer" {
      format!(
        "Unsupported auth.scheme '{}'. Only 'Bearer' is supported in this version.",
        self.auth.scheme
      )
    } else {
      return Ok(());
    };
    Err(ObjValidationError::LlmLibertyEnvelopeInvalid(reason))
  }
}

/// Tagged envelope update action — either keep the existing credentials or replace them.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(tag = "action", content = "value", rename_all = "lowercase")]
pub enum LlmLibertyEnvelopeUpdate {
  /// Keep the stored credentials unchanged.
  Keep,
  /// Replace with a new envelope (atomic re-paste).
  Set(LlmLibertyEnvelope),
}

impl Default for LlmLibertyEnvelopeUpdate {
  fn default() -> Self {
    LlmLibertyEnvelopeUpdate::Keep
  }
}

/// Non-secret summary of stored LLM Liberty OAuth credentials for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct LlmLibertySummary {
  pub provider: String,
  pub envelope_version: String,
  /// Unix epoch seconds — when the stored access token expires.
  pub expires_at: i64,
  pub has_refresh_token: bool,
}

/// Fully resolved credentials returned after the refresh check.
/// Contains plaintext tokens valid at the time of resolution.
#[derive(Debug, Clone)]
pub struct ResolvedLlmLibertyCredentials {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub tenant_id: String,
  /// The llm-liberty provider id ("anthropic" in v1). Routes that target a
  /// specific upstream MUST verify this matches before forwarding.
  pub provider: String,
  pub auth_scheme: String,
  pub auth_key: String,
  pub oauth_token_url: String,
  pub oauth_client_id: String,
  pub oauth_client_secret: Option<String>,
  pub api_base_url: String,
  pub api_chat_url: String,
  pub api_models_url: Option<String>,
  pub headers_json: serde_json::Value,
  pub body_json: serde_json::Value,
  pub extra_json: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
  use crate::models::llm_liberty_envelope::{
    LlmLibertyApiEndpoints, LlmLibertyAuthSpec, LlmLibertyEnvelope, LlmLibertyOauthEndpoints,
  };
  use crate::ObjValidationError;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  fn valid_envelope() -> LlmLibertyEnvelope {
    LlmLibertyEnvelope {
      version: "1.0.0".into(),
      provider: "anthropic".into(),
      access_token: "access".into(),
      refresh_token: "refresh".into(),
      expires_at: 0,
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
        base_url: "https://api.example".into(),
        chat_url: "https://api.example/messages".into(),
        models_url: None,
      },
      headers: serde_json::json!({}),
      body: serde_json::json!({}),
      extra: None,
    }
  }

  #[rstest]
  fn validate_supported_accepts_well_formed_envelope() {
    assert_eq!(Ok(()), valid_envelope().validate_supported());
  }

  #[rstest]
  fn validate_supported_accepts_codex_provider() {
    let mut env = valid_envelope();
    env.provider = "openai-codex".into();
    assert_eq!(Ok(()), env.validate_supported());
  }

  #[rstest]
  fn validate_supported_rejects_unknown_provider() {
    let mut env = valid_envelope();
    env.provider = "google-gemini".into();
    let err = env
      .validate_supported()
      .expect_err("expected validation error");
    assert!(matches!(
      err,
      ObjValidationError::LlmLibertyEnvelopeInvalid(_)
    ));
  }

  #[rstest]
  #[case::query_in("query", "Authorization", "Bearer")]
  #[case::wrong_key("header", "X-Api-Key", "Bearer")]
  #[case::wrong_scheme("header", "Authorization", "Basic")]
  fn validate_supported_rejects_non_bearer_header_auth(
    #[case] location: &str,
    #[case] key: &str,
    #[case] scheme: &str,
  ) {
    let mut env = valid_envelope();
    env.auth.location = location.into();
    env.auth.key = key.into();
    env.auth.scheme = scheme.into();
    let err = env
      .validate_supported()
      .expect_err("expected validation error");
    assert!(matches!(
      err,
      ObjValidationError::LlmLibertyEnvelopeInvalid(_)
    ));
  }

  #[rstest]
  fn deserializes_with_default_empty_object_for_headers_and_body() {
    let json = serde_json::json!({
      "version": "1.0.0",
      "provider": "anthropic",
      "access_token": "a",
      "refresh_token": "r",
      "expires_at": 0,
      "auth": {"in": "header", "key": "Authorization", "scheme": "Bearer"},
      "oauth": {
        "authorize_url": "https://x", "token_url": "https://x", "client_id": "c"
      },
      "api": {
        "base_url": "https://x", "chat_url": "https://x/m"
      }
    });
    let env: LlmLibertyEnvelope = serde_json::from_value(json).unwrap();
    assert_eq!(serde_json::json!({}), env.headers);
    assert_eq!(serde_json::json!({}), env.body);
  }
}
