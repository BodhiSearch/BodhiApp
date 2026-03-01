use crate::tokens::token_objs::{ApiTokenRow, TokenStatus};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "api_tokens")]
#[schema(as = ApiToken)]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub user_id: String,
  pub name: String,
  #[sea_orm(unique)]
  pub token_prefix: String,
  pub token_hash: String,
  pub scopes: String,
  pub status: TokenStatus,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
}

pub type ApiToken = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ApiTokenRow {
  fn from(m: Model) -> Self {
    ApiTokenRow {
      id: m.id,
      user_id: m.user_id,
      name: m.name,
      token_prefix: m.token_prefix,
      token_hash: m.token_hash,
      scopes: m.scopes,
      status: m.status,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
