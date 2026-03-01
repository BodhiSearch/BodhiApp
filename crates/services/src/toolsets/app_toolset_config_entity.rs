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

// ============================================================================
// AppToolsetConfigRow - Database row for app-level toolset type configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AppToolsetConfigRow {
  pub id: String,
  pub toolset_type: String,
  pub enabled: bool,
  pub updated_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<Model> for AppToolsetConfigRow {
  fn from(m: Model) -> Self {
    AppToolsetConfigRow {
      id: m.id,
      toolset_type: m.toolset_type,
      enabled: m.enabled,
      updated_by: m.updated_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
