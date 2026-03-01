use crate::AppStatus;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "apps")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub client_id: String,
  pub encrypted_client_secret: String,
  pub salt_client_secret: String,
  pub nonce_client_secret: String,
  pub app_status: AppStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone)]
pub struct AppInstanceRow {
  pub client_id: String,
  pub client_secret: String,
  pub app_status: AppStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<Model> for AppInstanceRow {
  fn from(model: Model) -> Self {
    AppInstanceRow {
      client_id: model.client_id,
      client_secret: String::new(),
      app_status: model.app_status,
      created_at: model.created_at,
      updated_at: model.updated_at,
    }
  }
}
