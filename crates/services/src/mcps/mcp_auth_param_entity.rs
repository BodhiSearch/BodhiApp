use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_auth_params")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub mcp_id: String,
  pub param_type: String,
  pub param_key: String,
  pub encrypted_value: String,
  pub value_salt: String,
  pub value_nonce: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_entity::Entity",
    from = "Column::McpId",
    to = "super::mcp_entity::Column::Id"
  )]
  Mcp,
}

impl Related<super::mcp_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Mcp.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Type alias — McpAuthParamEntity is the entity Model.
pub type McpAuthParamEntity = Model;
