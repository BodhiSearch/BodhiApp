use crate::models::{JsonVec, OAIRequestParams};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "user_aliases")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  #[sea_orm(unique)]
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  #[sea_orm(column_type = "JsonBinary")]
  pub request_params: OAIRequestParams,
  #[sea_orm(column_type = "JsonBinary")]
  pub context_params: JsonVec,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<Model> for crate::models::UserAlias {
  type Error = crate::models::ModelValidationError;

  fn try_from(m: Model) -> Result<Self, Self::Error> {
    Ok(crate::models::UserAlias {
      id: m.id,
      alias: m.alias,
      repo: m.repo.parse()?,
      filename: m.filename,
      snapshot: m.snapshot,
      request_params: m.request_params,
      context_params: m.context_params,
      created_at: m.created_at,
      updated_at: m.updated_at,
    })
  }
}
