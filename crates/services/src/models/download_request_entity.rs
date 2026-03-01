use crate::models::DownloadStatus;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema, DeriveEntityModel)]
#[sea_orm(table_name = "download_requests")]
#[schema(as = DownloadRequest)]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub repo: String,
  pub filename: String,
  pub status: DownloadStatus,
  pub error: Option<String>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub updated_at: DateTime<Utc>,
  pub total_bytes: Option<i64>,
  #[serde(default)]
  #[sea_orm(default_value = "0")]
  pub downloaded_bytes: i64,
  #[schema(value_type = String, format = "date-time", example = "2024-11-10T04:52:06.786Z")]
  pub started_at: Option<DateTime<Utc>>,
}

pub type DownloadRequest = Model;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub fn new_pending(repo: &str, filename: &str, now: DateTime<Utc>) -> Self {
    Model {
      id: ulid::Ulid::new().to_string(),
      repo: repo.to_string(),
      filename: filename.to_string(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: now,
      updated_at: now,
      total_bytes: None,
      downloaded_bytes: 0,
      started_at: None,
    }
  }
}
