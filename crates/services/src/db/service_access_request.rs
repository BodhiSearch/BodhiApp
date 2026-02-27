use crate::db::{
  entities::app_access_request, AccessRequestRepository, AppAccessRequestRow,
  AppAccessRequestStatus, DbError, DefaultDbService,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
impl AccessRequestRepository for DefaultDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let active = app_access_request::ActiveModel {
      id: Set(row.id.clone()),
      app_client_id: Set(row.app_client_id.clone()),
      app_name: Set(row.app_name.clone()),
      app_description: Set(row.app_description.clone()),
      flow_type: Set(row.flow_type.clone()),
      redirect_uri: Set(row.redirect_uri.clone()),
      status: Set(row.status.clone()),
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
    let result = app_access_request::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(AppAccessRequestRow::from))
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now();
    let active = app_access_request::ActiveModel {
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
    let now = self.time_service.utc_now();
    let active = app_access_request::ActiveModel {
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
    let now = self.time_service.utc_now();
    let active = app_access_request::ActiveModel {
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
    let result = app_access_request::Entity::find()
      .filter(app_access_request::Column::AccessRequestScope.eq(scope))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(AppAccessRequestRow::from))
  }
}
