use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_servers")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub url: String,
  pub name: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub created_by: String,
  pub updated_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::mcp_entity::Entity")]
  Mcp,
  #[sea_orm(has_many = "super::mcp_auth_header_entity::Entity")]
  McpAuthHeader,
  #[sea_orm(has_many = "super::mcp_oauth_config_entity::Entity")]
  McpOAuthConfig,
}

impl Related<super::mcp_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Mcp.def()
  }
}

impl Related<super::mcp_auth_header_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthHeader.def()
  }
}

impl Related<super::mcp_oauth_config_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpOAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Type alias — McpServerEntity is the entity Model.
pub type McpServerEntity = Model;

/// Backward-compatible alias (deprecated — use McpServerEntity).
pub type McpServerRow = Model;
