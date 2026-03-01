use super::access_request_objs::AppAccessRequestRow;
use super::app_access_request_entity;
use crate::db::{DbError, DefaultDbService};
use crate::AppAccessRequestStatus;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[async_trait]
pub trait AccessRequestRepository: Send + Sync {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError>;

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError>;

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str, // JSON string
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError>;

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError>;

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError>;

  async fn get_by_access_request_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppAccessRequestRow>, DbError>;
}

impl DefaultDbService {
  /// If the record is a draft and past its `expires_at`, mark it as `Expired` in DB and return the updated row.
  /// Otherwise return the row unchanged.
  async fn expire_if_draft_and_expired(
    &self,
    row: AppAccessRequestRow,
  ) -> Result<AppAccessRequestRow, DbError> {
    if row.status == AppAccessRequestStatus::Draft {
      let now = self.time_service.utc_now();
      if row.expires_at < now {
        let updated_at = now;
        let active = app_access_request_entity::ActiveModel {
          id: Set(row.id.clone()),
          status: Set(AppAccessRequestStatus::Expired),
          updated_at: Set(updated_at),
          ..Default::default()
        };
        let model = active.update(&self.db).await.map_err(DbError::from)?;
        return Ok(AppAccessRequestRow::from(model));
      }
    }
    Ok(row)
  }

  /// Fetch the record (raw, without auto-expire), and if it's a draft past expiry mark it
  /// expired and return an error. If it's not a draft, return an error. Otherwise return Ok(()).
  async fn validate_draft_for_update(&self, id: &str) -> Result<(), DbError> {
    let row = app_access_request_entity::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .map(AppAccessRequestRow::from)
      .ok_or_else(|| DbError::ItemNotFound {
        id: id.to_string(),
        item_type: "app_access_request".to_string(),
      })?;
    if row.status == AppAccessRequestStatus::Draft {
      let now = self.time_service.utc_now();
      if row.expires_at < now {
        // Mark as expired in DB
        let active = app_access_request_entity::ActiveModel {
          id: Set(id.to_string()),
          status: Set(AppAccessRequestStatus::Expired),
          updated_at: Set(now),
          ..Default::default()
        };
        active.update(&self.db).await.map_err(DbError::from)?;
        return Err(DbError::AccessRequestExpired(id.to_string()));
      }
      Ok(())
    } else {
      Err(DbError::AccessRequestNotDraft {
        id: id.to_string(),
        status: row.status.to_string(),
      })
    }
  }
}

#[async_trait::async_trait]
impl AccessRequestRepository for DefaultDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let active = app_access_request_entity::ActiveModel {
      id: Set(row.id.clone()),
      app_client_id: Set(row.app_client_id.clone()),
      app_name: Set(row.app_name.clone()),
      app_description: Set(row.app_description.clone()),
      flow_type: Set(row.flow_type.clone()),
      redirect_uri: Set(row.redirect_uri.clone()),
      status: Set(row.status),
      requested: Set(row.requested.clone()),
      approved: Set(row.approved.clone()),
      user_id: Set(row.user_id.clone()),
      requested_role: Set(row.requested_role.clone()),
      approved_role: Set(row.approved_role.clone()),
      access_request_scope: Set(row.access_request_scope.clone()),
      error_message: Set(row.error_message.clone()),
      expires_at: Set(row.expires_at),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let model = active.insert(&self.db).await.map_err(DbError::from)?;
    Ok(AppAccessRequestRow::from(model))
  }

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = app_access_request_entity::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    match result {
      Some(model) => {
        let row = AppAccessRequestRow::from(model);
        Ok(Some(self.expire_if_draft_and_expired(row).await?))
      }
      None => Ok(None),
    }
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    self.validate_draft_for_update(id).await?;
    let now = self.time_service.utc_now();
    let active = app_access_request_entity::ActiveModel {
      id: Set(id.to_string()),
      status: Set(AppAccessRequestStatus::Approved),
      user_id: Set(Some(user_id.to_string())),
      approved: Set(Some(approved.to_string())),
      approved_role: Set(Some(approved_role.to_string())),
      access_request_scope: Set(Some(access_request_scope.to_string())),
      updated_at: Set(now),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(AppAccessRequestRow::from(model))
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError> {
    self.validate_draft_for_update(id).await?;
    let now = self.time_service.utc_now();
    let active = app_access_request_entity::ActiveModel {
      id: Set(id.to_string()),
      status: Set(AppAccessRequestStatus::Denied),
      user_id: Set(Some(user_id.to_string())),
      updated_at: Set(now),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(AppAccessRequestRow::from(model))
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    self.validate_draft_for_update(id).await?;
    let now = self.time_service.utc_now();
    let active = app_access_request_entity::ActiveModel {
      id: Set(id.to_string()),
      status: Set(AppAccessRequestStatus::Failed),
      error_message: Set(Some(error_message.to_string())),
      updated_at: Set(now),
      ..Default::default()
    };
    let model = active.update(&self.db).await.map_err(DbError::from)?;
    Ok(AppAccessRequestRow::from(model))
  }

  async fn get_by_access_request_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = app_access_request_entity::Entity::find()
      .filter(app_access_request_entity::Column::AccessRequestScope.eq(scope))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(AppAccessRequestRow::from))
  }
}
