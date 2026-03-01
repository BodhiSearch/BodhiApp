use crate::UserAccessRequestStatus;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema, DeriveEntityModel)]
#[sea_orm(table_name = "access_requests")]
#[schema(as = UserAccessRequest)]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub username: String,
  pub user_id: String,
  #[serde(default)]
  pub reviewer: Option<String>,
  pub status: UserAccessRequestStatus,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

pub type UserAccessRequest = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
