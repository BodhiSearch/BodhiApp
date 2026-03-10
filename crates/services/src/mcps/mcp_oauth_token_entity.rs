use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_oauth_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub mcp_oauth_config_id: String,
  pub encrypted_access_token: String,
  pub access_token_salt: String,
  pub access_token_nonce: String,
  pub encrypted_refresh_token: Option<String>,
  pub refresh_token_salt: Option<String>,
  pub refresh_token_nonce: Option<String>,
  pub scopes_granted: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
  pub user_id: String,
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
  pub user_id: String,
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
      user_id: v.user_id,
      created_at: v.created_at,
      updated_at: v.updated_at,
    }
  }
}

/// Type alias — McpOAuthTokenEntity is the entity Model.
pub type McpOAuthTokenEntity = Model;
