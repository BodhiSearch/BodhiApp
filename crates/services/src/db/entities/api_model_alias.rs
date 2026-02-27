use chrono::{DateTime, Utc};
use objs::{ApiFormat, JsonVec};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "api_model_aliases")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  #[sea_orm(column_type = "JsonBinary")]
  pub models: JsonVec,
  pub prefix: Option<String>,
  #[sea_orm(default_value = "0")]
  pub forward_all_with_prefix: bool,
  #[sea_orm(column_type = "JsonBinary")]
  pub models_cache: JsonVec,
  pub cache_fetched_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  // DB-only encryption fields
  pub encrypted_api_key: Option<String>,
  pub salt: Option<String>,
  pub nonce: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// View that excludes encryption fields for domain reads
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct ApiAliasView {
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  pub models: JsonVec,
  pub prefix: Option<String>,
  pub forward_all_with_prefix: bool,
  pub models_cache: JsonVec,
  pub cache_fetched_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<ApiAliasView> for objs::ApiAlias {
  fn from(v: ApiAliasView) -> Self {
    objs::ApiAlias {
      id: v.id,
      api_format: v.api_format,
      base_url: v.base_url,
      models: v.models,
      prefix: v.prefix,
      forward_all_with_prefix: v.forward_all_with_prefix,
      models_cache: v.models_cache,
      cache_fetched_at: v.cache_fetched_at,
      created_at: v.created_at,
      updated_at: v.updated_at,
    }
  }
}
