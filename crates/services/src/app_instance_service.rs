use crate::db::{AppInstanceRow, DbError, DbService};
use crate::{AppInstance, AppStatus};
use chrono::{TimeZone, Utc};
use objs::{AppError, ErrorType};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppInstanceError {
  #[error("Application instance not found.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  NotFound,
  #[error("Invalid application status: '{0}'.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidStatus(String),
  #[error("Invalid timestamp value: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidTimestamp(i64),
  #[error(transparent)]
  Db(#[from] DbError),
}

type Result<T> = std::result::Result<T, AppInstanceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait AppInstanceService: Send + Sync + std::fmt::Debug {
  async fn get_instance(&self) -> Result<Option<AppInstance>>;
  async fn get_status(&self) -> Result<AppStatus>;
  async fn create_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: AppStatus,
  ) -> Result<AppInstance>;
  async fn update_status(&self, status: &AppStatus) -> Result<()>;
}

fn row_to_instance(row: AppInstanceRow) -> Result<AppInstance> {
  let status = row
    .app_status
    .parse::<AppStatus>()
    .map_err(|_| AppInstanceError::InvalidStatus(row.app_status.clone()))?;
  let created_at = Utc
    .timestamp_opt(row.created_at, 0)
    .single()
    .ok_or(AppInstanceError::InvalidTimestamp(row.created_at))?;
  let updated_at = Utc
    .timestamp_opt(row.updated_at, 0)
    .single()
    .ok_or(AppInstanceError::InvalidTimestamp(row.updated_at))?;
  Ok(AppInstance {
    client_id: row.client_id,
    client_secret: row.client_secret,
    status,
    created_at,
    updated_at,
  })
}

#[derive(Debug, derive_new::new)]
pub struct DefaultAppInstanceService {
  db_service: Arc<dyn DbService>,
}

#[async_trait::async_trait]
impl AppInstanceService for DefaultAppInstanceService {
  async fn get_instance(&self) -> Result<Option<AppInstance>> {
    let row = self.db_service.get_app_instance().await?;
    row.map(row_to_instance).transpose()
  }

  async fn get_status(&self) -> Result<AppStatus> {
    let row = self.db_service.get_app_instance().await?;
    match row {
      None => Ok(AppStatus::default()),
      Some(r) => r
        .app_status
        .parse::<AppStatus>()
        .map_err(|_| AppInstanceError::InvalidStatus(r.app_status)),
    }
  }

  async fn create_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    status: AppStatus,
  ) -> Result<AppInstance> {
    self
      .db_service
      .upsert_app_instance(client_id, client_secret, &status.to_string())
      .await?;
    self.get_instance().await?.ok_or(AppInstanceError::NotFound)
  }

  async fn update_status(&self, status: &AppStatus) -> Result<()> {
    let instance = self
      .get_instance()
      .await?
      .ok_or(AppInstanceError::NotFound)?;
    self
      .db_service
      .update_app_instance_status(&instance.client_id, &status.to_string())
      .await?;
    Ok(())
  }
}

#[cfg(test)]
#[path = "test_app_instance_service.rs"]
mod test_app_instance_service;
