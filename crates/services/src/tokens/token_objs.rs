use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// TokenStatus - API token active/inactive status
// ============================================================================

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

// ============================================================================
// ApiTokenRow - Plain Rust struct representing an API token record
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiTokenRow {
  pub id: String,
  pub user_id: String,
  pub name: String,
  pub token_prefix: String,
  pub token_hash: String,
  pub scopes: String,
  pub status: TokenStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
