use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_auth_headers")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
  pub header_key: String,
  pub encrypted_header_value: String,
  pub header_value_salt: String,
  pub header_value_nonce: String,
  pub created_by: String,
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

/// View that excludes encryption fields for domain reads
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct McpAuthHeaderView {
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
  pub header_key: String,
  pub created_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<McpAuthHeaderView> for crate::mcps::McpAuthHeader {
  fn from(v: McpAuthHeaderView) -> Self {
    crate::mcps::McpAuthHeader {
      id: v.id,
      name: v.name,
      mcp_server_id: v.mcp_server_id,
      header_key: v.header_key,
      has_header_value: true,
      created_by: v.created_by,
      created_at: v.created_at,
      updated_at: v.updated_at,
    }
  }
}

// ============================================================================
// McpAuthHeaderRow - Database row for header-based MCP authentication configs
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct McpAuthHeaderRow {
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
  pub header_key: String,
  pub encrypted_header_value: String,
  pub header_value_salt: String,
  pub header_value_nonce: String,
  pub created_by: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Model> for McpAuthHeaderRow {
  fn from(m: Model) -> Self {
    McpAuthHeaderRow {
      id: m.id,
      name: m.name,
      mcp_server_id: m.mcp_server_id,
      header_key: m.header_key,
      encrypted_header_value: m.encrypted_header_value,
      header_value_salt: m.header_value_salt,
      header_value_nonce: m.header_value_nonce,
      created_by: m.created_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
