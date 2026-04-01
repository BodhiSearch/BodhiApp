use crate::mcps::McpAuthType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcps")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub user_id: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub auth_type: McpAuthType,
  pub auth_config_id: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_server_entity::Entity",
    from = "Column::McpServerId",
    to = "super::mcp_server_entity::Column::Id"
  )]
  McpServer,
  #[sea_orm(
    belongs_to = "super::mcp_auth_config_entity::Entity",
    from = "Column::AuthConfigId",
    to = "super::mcp_auth_config_entity::Column::Id"
  )]
  McpAuthConfig,
}

impl Related<super::mcp_server_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpServer.def()
  }
}

impl Related<super::mcp_auth_config_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Type alias — McpEntity is the entity Model.
pub type McpEntity = Model;

/// Joined MCP instance + server info from SQL JOIN query
#[derive(Debug, Clone, PartialEq)]
pub struct McpWithServerEntity {
  pub id: String,
  pub user_id: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub auth_type: McpAuthType,
  pub auth_config_id: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  // Server info from JOIN
  pub server_url: String,
  pub server_name: String,
  pub server_enabled: bool,
}
