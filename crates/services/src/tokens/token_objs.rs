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
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<super::TokenEntity> for TokenDetail {
  fn from(t: super::TokenEntity) -> Self {
    Self {
      id: t.id,
      user_id: t.user_id,
      name: t.name,
      token_prefix: t.token_prefix,
      scopes: t.scopes,
      status: t.status,
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
