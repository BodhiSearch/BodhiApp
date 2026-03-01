use crate::mcps::McpAuthType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcps")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub created_by: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub tools_cache: Option<String>,
  pub tools_filter: Option<String>,
  pub auth_type: McpAuthType,
  pub auth_uuid: Option<String>,
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
}

impl Related<super::mcp_server_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpServer.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

// ============================================================================
// McpRow - Database row for user-owned MCP instances
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpRow {
  pub id: String,
  pub created_by: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub tools_cache: Option<String>,
  pub tools_filter: Option<String>,
  pub auth_type: McpAuthType,
  pub auth_uuid: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Joined MCP instance + server info from SQL JOIN query
#[derive(Debug, Clone, PartialEq)]
pub struct McpWithServerRow {
  pub id: String,
  pub created_by: String,
  pub mcp_server_id: String,
  pub name: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub tools_cache: Option<String>,
  pub tools_filter: Option<String>,
  pub auth_type: McpAuthType,
  pub auth_uuid: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  // Server info from JOIN
  pub server_url: String,
  pub server_name: String,
  pub server_enabled: bool,
}

impl From<Model> for McpRow {
  fn from(m: Model) -> Self {
    McpRow {
      id: m.id,
      created_by: m.created_by,
      mcp_server_id: m.mcp_server_id,
      name: m.name,
      slug: m.slug,
      description: m.description,
      enabled: m.enabled,
      tools_cache: m.tools_cache,
      tools_filter: m.tools_filter,
      auth_type: m.auth_type,
      auth_uuid: m.auth_uuid,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
