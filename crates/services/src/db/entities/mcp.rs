use chrono::{DateTime, Utc};
use objs::McpAuthType;
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
    belongs_to = "super::mcp_server::Entity",
    from = "Column::McpServerId",
    to = "super::mcp_server::Column::Id"
  )]
  McpServer,
}

impl Related<super::mcp_server::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpServer.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::db::McpRow {
  fn from(m: Model) -> Self {
    crate::db::McpRow {
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
