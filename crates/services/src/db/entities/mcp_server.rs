use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_servers")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
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
  #[sea_orm(has_many = "super::mcp::Entity")]
  Mcp,
  #[sea_orm(has_many = "super::mcp_auth_header::Entity")]
  McpAuthHeader,
  #[sea_orm(has_many = "super::mcp_oauth_config::Entity")]
  McpOAuthConfig,
}

impl Related<super::mcp::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Mcp.def()
  }
}

impl Related<super::mcp_auth_header::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthHeader.def()
  }
}

impl Related<super::mcp_oauth_config::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpOAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::db::McpServerRow {
  fn from(m: Model) -> Self {
    crate::db::McpServerRow {
      id: m.id,
      url: m.url,
      name: m.name,
      description: m.description,
      enabled: m.enabled,
      created_by: m.created_by,
      updated_by: m.updated_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
