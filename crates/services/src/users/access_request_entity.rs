use crate::UserAccessRequestStatus;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "user_access_requests")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub tenant_id: String,
  pub username: String,
  pub user_id: String,
  #[serde(default)]
  pub reviewer: Option<String>,
  pub status: UserAccessRequestStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

pub type UserAccessRequestEntity = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
