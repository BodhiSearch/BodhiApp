use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "toolsets")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub user_id: String,
  pub toolset_type: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub encrypted_api_key: Option<String>,
  pub salt: Option<String>,
  pub nonce: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ============================================================================
// ToolsetRow - Database row for user toolset instances
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ToolsetRow {
  pub id: String,
  pub user_id: String,
  pub toolset_type: String,
  pub slug: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub encrypted_api_key: Option<String>,
  pub salt: Option<String>,
  pub nonce: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<Model> for ToolsetRow {
  fn from(m: Model) -> Self {
    ToolsetRow {
      id: m.id,
      user_id: m.user_id,
      toolset_type: m.toolset_type,
      slug: m.slug,
      description: m.description,
      enabled: m.enabled,
      encrypted_api_key: m.encrypted_api_key,
      salt: m.salt,
      nonce: m.nonce,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
