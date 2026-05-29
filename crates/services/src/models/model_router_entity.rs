use crate::models::{ModelRouterAlias, RouterTargetVec, RoutingStrategyConfig};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model_router_aliases")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub user_id: String,
  pub alias: String,
  #[sea_orm(column_type = "JsonBinary")]
  pub targets: RouterTargetVec,
  #[sea_orm(column_type = "JsonBinary")]
  pub strategy: RoutingStrategyConfig,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type ModelRouterEntity = Model;

impl From<Model> for ModelRouterAlias {
  fn from(m: Model) -> Self {
    ModelRouterAlias {
      id: m.id,
      alias: m.alias,
      targets: m.targets.into(),
      strategy: m.strategy,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
