use crate::ObjValidationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// =============================================================================
// Envelope sub-types (mirror llm-liberty JSON contract v1.0.0)
// =============================================================================

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

// =============================================================================
// Top-level envelope
// =============================================================================

/// The JSON blob emitted by `npx @bodhiapp/llm-liberty@latest login`.
///
/// Version field allows BodhiApp to detect breaking changes in the envelope
/// schema (major-version bump = breaking). Currently only `"1.0.0"` is accepted.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq)]
pub struct LlmLibertyEnvelope {
  /// Envelope schema version — must be "1.0.0".
  pub version: String,
  /// Provider identifier, e.g. "anthropic". Only "anthropic" is supported in v1.
  pub provider: String,
  pub access_token: String,
  pub refresh_token: String,
  /// Unix epoch seconds when the access token expires.
  pub expires_at: i64,
  pub auth: LlmLibertyAuthSpec,
  pub oauth: LlmLibertyOauthEndpoints,
  pub api: LlmLibertyApiEndpoints,
  #[serde(default)]
  pub headers: serde_json::Value,
  #[serde(default)]
  pub body: serde_json::Value,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub extra: Option<serde_json::Value>,
}

impl LlmLibertyEnvelope {
  /// Validate that the envelope is usable: known version and supported provider.
  pub fn validate_supported(&self) -> Result<(), ObjValidationError> {
    let reason = if self.version != "1.0.0" {
      format!(
        "Unsupported llm-liberty envelope version '{}'. Expected '1.0.0'.",
        self.version
      )
    } else if self.provider != "anthropic" {
      format!(
        "Unsupported provider '{}'. Only 'anthropic' is supported in this version.",
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
    } else {
      return Ok(());
    };
    Err(ObjValidationError::LlmLibertyEnvelopeInvalid(reason))
  }

  /// Build the request-shape parameters needed to call an upstream provider client
  /// directly from a not-yet-persisted envelope (test/fetch-models flow before save).
  pub fn to_request_parts(&self) -> LlmLibertyRequestParts {
    LlmLibertyRequestParts {
      access_token: Some(self.access_token.clone()),
      base_url: self.api.base_url.clone(),
      extra_headers: value_to_opt(&self.headers),
      extra_body: value_to_opt(&self.body),
    }
  }
}

// =============================================================================
// Update action (parallels ApiKeyUpdate)
// =============================================================================

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

// =============================================================================
// Summary returned in ApiAliasResponse (no secrets)
// =============================================================================

/// Non-secret summary of stored LLM Liberty OAuth credentials for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct LlmLibertySummary {
  pub provider: String,
  pub envelope_version: String,
  /// Unix epoch seconds — when the stored access token expires.
  pub expires_at: i64,
  pub has_refresh_token: bool,
}

// =============================================================================
// Resolved credentials (produced by refresh layer, consumed by provider clients)
// =============================================================================

/// Fully resolved credentials returned after the refresh check.
/// Contains plaintext tokens valid at the time of resolution.
#[derive(Debug, Clone)]
pub struct ResolvedLlmLibertyCredentials {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_at: chrono::DateTime<chrono::Utc>,
  pub tenant_id: String,
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

impl ResolvedLlmLibertyCredentials {
  /// Build the request-shape parameters needed to call an upstream provider client.
  /// Consumes self to avoid cloning the JSON values.
  pub fn into_request_parts(self) -> LlmLibertyRequestParts {
    LlmLibertyRequestParts {
      access_token: Some(self.access_token),
      base_url: self.api_base_url,
      extra_headers: value_to_opt_owned(self.headers_json),
      extra_body: value_to_opt_owned(self.body_json),
    }
  }
}

// =============================================================================
// Common request-parts shape used by api_model_service and route handlers
// =============================================================================

/// The four parameters every provider-client constructor needs:
/// `(api_key, base_url, extra_headers, extra_body)`. Constructed from either an
/// envelope (create/test) or resolved stored credentials (sync/forward).
#[derive(Debug, Clone)]
pub struct LlmLibertyRequestParts {
  pub access_token: Option<String>,
  pub base_url: String,
  pub extra_headers: Option<serde_json::Value>,
  pub extra_body: Option<serde_json::Value>,
}

fn value_to_opt(v: &serde_json::Value) -> Option<serde_json::Value> {
  if v.is_null() {
    None
  } else {
    Some(v.clone())
  }
}

fn value_to_opt_owned(v: serde_json::Value) -> Option<serde_json::Value> {
  if v.is_null() {
    None
  } else {
    Some(v)
  }
}
