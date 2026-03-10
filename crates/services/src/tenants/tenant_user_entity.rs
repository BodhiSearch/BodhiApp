use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tenants_users")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub tenant_id: String,
  #[sea_orm(primary_key, auto_increment = false)]
  pub user_id: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::tenant_entity::Entity",
    from = "Column::TenantId",
    to = "super::tenant_entity::Column::Id"
  )]
  Tenant,
}

impl Related<super::tenant_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tenant.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
