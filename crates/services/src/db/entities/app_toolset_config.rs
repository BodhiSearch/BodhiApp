use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "app_toolset_configs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  #[sea_orm(unique)]
  pub toolset_type: String,
  pub enabled: bool,
  pub updated_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::db::AppToolsetConfigRow {
  fn from(m: Model) -> Self {
    crate::db::AppToolsetConfigRow {
      id: m.id,
      toolset_type: m.toolset_type,
      enabled: m.enabled,
      updated_by: m.updated_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
