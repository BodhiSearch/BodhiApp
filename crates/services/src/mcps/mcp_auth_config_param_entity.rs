use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_auth_config_params")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub auth_config_id: String,
  pub param_type: String,
  pub param_key: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_auth_config_entity::Entity",
    from = "Column::AuthConfigId",
    to = "super::mcp_auth_config_entity::Column::Id"
  )]
  McpAuthConfig,
}

impl Related<super::mcp_auth_config_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Type alias — McpAuthConfigParamEntity is the entity Model.
pub type McpAuthConfigParamEntity = Model;
