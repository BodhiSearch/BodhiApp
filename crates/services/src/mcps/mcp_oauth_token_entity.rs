use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_oauth_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub mcp_oauth_config_id: String,
  pub encrypted_access_token: String,
  pub access_token_salt: String,
  pub access_token_nonce: String,
  pub encrypted_refresh_token: Option<String>,
  pub refresh_token_salt: Option<String>,
  pub refresh_token_nonce: Option<String>,
  pub scopes_granted: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
  pub created_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_oauth_config_entity::Entity",
    from = "Column::McpOauthConfigId",
    to = "super::mcp_oauth_config_entity::Column::Id"
  )]
  McpOAuthConfig,
}

impl Related<super::mcp_oauth_config_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpOAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// View that excludes encryption fields for domain reads.
/// Includes encrypted_refresh_token for is_some() check only.
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct McpOAuthTokenView {
  pub id: String,
  pub mcp_oauth_config_id: String,
  pub encrypted_refresh_token: Option<String>,
  pub scopes_granted: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
  pub created_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<McpOAuthTokenView> for crate::mcps::McpOAuthToken {
  fn from(v: McpOAuthTokenView) -> Self {
    crate::mcps::McpOAuthToken {
      id: v.id,
      mcp_oauth_config_id: v.mcp_oauth_config_id,
      scopes_granted: v.scopes_granted,
      expires_at: v.expires_at.map(|dt| dt.timestamp()),
      has_access_token: true,
      has_refresh_token: v.encrypted_refresh_token.is_some(),
      created_by: v.created_by,
      created_at: v.created_at,
      updated_at: v.updated_at,
    }
  }
}

// ============================================================================
// McpOAuthTokenRow - Database row for OAuth 2.1 stored tokens
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpOAuthTokenRow {
  pub id: String,
  pub mcp_oauth_config_id: String,
  pub encrypted_access_token: String,
  pub access_token_salt: String,
  pub access_token_nonce: String,
  pub encrypted_refresh_token: Option<String>,
  pub refresh_token_salt: Option<String>,
  pub refresh_token_nonce: Option<String>,
  pub scopes_granted: Option<String>,
  pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
  pub created_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Model> for McpOAuthTokenRow {
  fn from(m: Model) -> Self {
    McpOAuthTokenRow {
      id: m.id,
      mcp_oauth_config_id: m.mcp_oauth_config_id,
      encrypted_access_token: m.encrypted_access_token,
      access_token_salt: m.access_token_salt,
      access_token_nonce: m.access_token_nonce,
      encrypted_refresh_token: m.encrypted_refresh_token,
      refresh_token_salt: m.refresh_token_salt,
      refresh_token_nonce: m.refresh_token_nonce,
      scopes_granted: m.scopes_granted,
      expires_at: m.expires_at,
      created_by: m.created_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
