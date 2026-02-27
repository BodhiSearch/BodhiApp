use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "settings")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub key: String,
  pub value: String,
  pub value_type: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::db::DbSetting {
  fn from(m: Model) -> Self {
    crate::db::DbSetting {
      key: m.key,
      value: m.value,
      value_type: m.value_type,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
