use crate::TokenScope;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TokenStatus {
  Active,
  Inactive,
}

/// Model inference grant for an API token. `All` is a wildcard that includes
/// models added in the future; `Specific` lists alias ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelGrant {
  #[default]
  All,
  Specific {
    ids: Vec<String>,
  },
}

/// MCP connect grant for an API token. `All` is a wildcard (incl. future MCPs),
/// `None` grants no MCP access, `Specific` lists the user's own instance ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpGrant {
  #[default]
  All,
  None,
  Specific {
    ids: Vec<String>,
  },
}

/// Per-resource grants carried by an API token. Listing (`list_models` /
/// `list_mcps`) is separate from inference/connect: with listing off the
/// discovery endpoints return an empty set, but inference on an individually
/// granted resource still succeeds.
///
/// Intentionally standalone — NOT shared with the App-access-request envelope
/// (`ApprovedResources`); the two may diverge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct TokenGrantsV1 {
  #[serde(default)]
  pub list_models: bool,
  #[serde(default)]
  pub models: ModelGrant,
  #[serde(default)]
  pub list_mcps: bool,
  #[serde(default)]
  pub mcps: McpGrant,
}

/// Versioned envelope; the `version` tag is mandatory (mirrors `ApprovedResources`).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(tag = "version")]
pub enum TokenGrants {
  #[serde(rename = "1")]
  V1(TokenGrantsV1),
}

impl<'de> Deserialize<'de> for TokenGrants {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = serde_json::Value::deserialize(deserializer)?;
    let version = value
      .get("version")
      .and_then(|v| v.as_str())
      .ok_or_else(|| serde::de::Error::missing_field("version"))?;
    match version {
      "1" => {
        let v1: TokenGrantsV1 = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self::V1(v1))
      }
      unknown => Err(serde::de::Error::custom(format!(
        "Unsupported token grants version '{}'. Supported versions: [1]",
        unknown
      ))),
    }
  }
}

impl TokenGrants {
  pub fn version(&self) -> &str {
    match self {
      Self::V1(_) => "1",
    }
  }

  pub fn v1(&self) -> &TokenGrantsV1 {
    match self {
      Self::V1(v1) => v1,
    }
  }
}

impl Default for TokenGrants {
  /// All-access (parity with the pre-grants behavior): list + use every model and MCP.
  fn default() -> Self {
    Self::V1(TokenGrantsV1 {
      list_models: true,
      models: ModelGrant::All,
      list_mcps: true,
      mcps: McpGrant::All,
    })
  }
}

/// Canonical all-access grants JSON — the `api_tokens.grants` column default and the
/// value stamped on tokens created without explicit grants.
pub fn default_grants_json() -> String {
  serde_json::to_string(&TokenGrants::default()).expect("TokenGrants serializes")
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "My Integration Token",
    "scope": "scope_token_user"
}))]
pub struct CreateTokenRequest {
  /// Descriptive name for the API token
  #[serde(default)]
  #[schema(min_length = 0, max_length = 100, example = "My Integration Token")]
  pub name: Option<String>,
  /// Token scope defining access level
  #[schema(example = "scope_token_user")]
  pub scope: TokenScope,
  /// Per-resource grants for this token. Defaults to all-access when omitted.
  #[serde(default)]
  pub grants: TokenGrants,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "Updated Token Name",
    "status": "inactive"
}))]
pub struct UpdateTokenRequest {
  /// New descriptive name for the token
  #[schema(min_length = 3, max_length = 100, example = "Updated Token Name")]
  pub name: String,
  /// New status for the token (active/inactive)
  #[schema(example = "inactive")]
  pub status: TokenStatus,
}

// Returned only on create; contains the raw token string shown once.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "bodhiapp_1234567890abcdef"
}))]
pub struct TokenCreated {
  /// API token with bodhiapp_ prefix for programmatic access
  #[schema(example = "bodhiapp_1234567890abcdef")]
  pub token: String,
}

// Output type for get/list/update: entity minus tenant_id and token_hash.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenDetail {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub token_prefix: String,
  pub scopes: String,
  pub status: TokenStatus,
  /// Per-resource grants this token carries.
  pub grants: TokenGrants,
  #[schema(value_type = Option<String>, format = "date-time")]
  pub last_used_at: Option<DateTime<Utc>>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<super::TokenEntity> for TokenDetail {
  fn from(t: super::TokenEntity) -> Self {
    // Stored grants are written by us and always valid; fall back to all-access
    // defensively rather than panicking on an unexpected payload.
    let grants = serde_json::from_str(&t.grants).unwrap_or_default();
    Self {
      id: t.id,
      user_id: t.user_id,
      name: t.name,
      token_prefix: t.token_prefix,
      scopes: t.scopes,
      status: t.status,
      grants,
      last_used_at: t.last_used_at,
      created_at: t.created_at,
      updated_at: t.updated_at,
    }
  }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedTokenResponse {
  pub data: Vec<TokenDetail>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[cfg(test)]
mod tests {
  use crate::tokens::token_objs::{
    default_grants_json, McpGrant, ModelGrant, TokenGrants, TokenGrantsV1,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rstest]
  #[case(TokenGrants::default())]
  #[case(TokenGrants::V1(TokenGrantsV1 {
    list_models: false,
    models: ModelGrant::Specific { ids: vec!["m1".into(), "m2".into()] },
    list_mcps: true,
    mcps: McpGrant::None,
  }))]
  #[case(TokenGrants::V1(TokenGrantsV1 {
    list_models: true,
    models: ModelGrant::All,
    list_mcps: false,
    mcps: McpGrant::Specific { ids: vec!["inst-1".into()] },
  }))]
  fn token_grants_round_trip(#[case] grants: TokenGrants) {
    let json = serde_json::to_string(&grants).unwrap();
    let parsed: TokenGrants = serde_json::from_str(&json).unwrap();
    assert_eq!(grants, parsed);
  }

  #[test]
  fn default_grants_json_is_all_access() {
    assert_eq!(
      r#"{"version":"1","list_models":true,"models":{"type":"all"},"list_mcps":true,"mcps":{"type":"all"}}"#,
      default_grants_json()
    );
  }

  #[test]
  fn token_grants_defaults_missing_fields() {
    // Only the mandatory version tag → every field falls back to its serde default.
    let parsed: TokenGrants = serde_json::from_str(r#"{"version":"1"}"#).unwrap();
    assert_eq!(
      TokenGrants::V1(TokenGrantsV1 {
        list_models: false,
        models: ModelGrant::All,
        list_mcps: false,
        mcps: McpGrant::All,
      }),
      parsed
    );
  }

  #[test]
  fn token_grants_missing_version_errors() {
    let err = serde_json::from_str::<TokenGrants>(r#"{"list_models":true}"#).unwrap_err();
    assert!(err.to_string().contains("version"));
  }

  #[test]
  fn token_grants_unknown_version_errors() {
    let err = serde_json::from_str::<TokenGrants>(r#"{"version":"2"}"#).unwrap_err();
    assert!(err.to_string().contains("Unsupported token grants version"));
  }
}
