use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_auth_configs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub mcp_server_id: String,
  pub config_type: String,
  pub name: String,
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
  #[sea_orm(has_many = "super::mcp_auth_config_param_entity::Entity")]
  McpAuthConfigParam,
  #[sea_orm(has_one = "super::mcp_oauth_config_detail_entity::Entity")]
  McpOAuthConfigDetail,
}

impl Related<super::mcp_server_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpServer.def()
  }
}

impl Related<super::mcp_auth_config_param_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthConfigParam.def()
  }
}

impl Related<super::mcp_oauth_config_detail_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpOAuthConfigDetail.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Type alias — McpAuthConfigEntity is the entity Model.
pub type McpAuthConfigEntity = Model;
