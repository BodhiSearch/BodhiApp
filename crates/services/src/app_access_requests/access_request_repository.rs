use super::access_request_objs::AppAccessRequest;
use super::app_access_request_entity;
use crate::db::{DbError, DefaultDbService};
use crate::AppAccessRequestStatus;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[async_trait]
pub trait AccessRequestRepository: Send + Sync {
  async fn create(&self, row: &AppAccessRequest) -> Result<AppAccessRequest, DbError>;

  async fn get(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>, DbError>;

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str, // JSON string
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequest, DbError>;

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequest, DbError>;

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequest, DbError>;

  async fn get_by_access_request_scope(
    &self,
    tenant_id: &str,
    scope: &str,
  ) -> Result<Option<AppAccessRequest>, DbError>;
}

#[async_trait::async_trait]
impl AccessRequestRepository for DefaultDbService {
  async fn create(&self, row: &AppAccessRequest) -> Result<AppAccessRequest, DbError> {
    let active = app_access_request_entity::ActiveModel {
      id: Set(row.id.clone()),
      tenant_id: Set(row.tenant_id.clone()),
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
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          let model = active.insert(txn).await.map_err(DbError::from)?;
          Ok(AppAccessRequest::from(model))
        })
      })
      .await
  }

  async fn get(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id_owned = id.to_string();
    let now = self.time_service.utc_now();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = app_access_request_entity::Entity::find_by_id(&id_owned);
          if !tenant_id_owned.is_empty() {
            query = query.filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned));
          }
          let result = query.one(txn).await.map_err(DbError::from)?;
          match result {
            Some(model) => {
              let row = AppAccessRequest::from(model);
              // expire_if_draft_and_expired inline
              if row.status == AppAccessRequestStatus::Draft && row.expires_at < now {
                let active = app_access_request_entity::ActiveModel {
                  id: Set(row.id.clone()),
                  status: Set(AppAccessRequestStatus::Expired),
                  updated_at: Set(now),
                  ..Default::default()
                };
                let model = active.update(txn).await.map_err(DbError::from)?;
                Ok(Some(AppAccessRequest::from(model)))
              } else {
                Ok(Some(row))
              }
            }
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let user_id_owned = user_id.to_string();
    let approved_owned = approved.to_string();
    let approved_role_owned = approved_role.to_string();
    let access_request_scope_owned = access_request_scope.to_string();

    // Look up tenant_id from the record first
    let row = app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          // validate_draft_for_update inline
          let row = app_access_request_entity::Entity::find_by_id(&id_owned)
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(AppAccessRequest::from)
            .ok_or_else(|| DbError::ItemNotFound {
              id: id_owned.clone(),
              item_type: "app_access_request".to_string(),
            })?;
          if row.status == AppAccessRequestStatus::Draft {
            if row.expires_at < now {
              let active = app_access_request_entity::ActiveModel {
                id: Set(id_owned.clone()),
                status: Set(AppAccessRequestStatus::Expired),
                updated_at: Set(now),
                ..Default::default()
              };
              active.update(txn).await.map_err(DbError::from)?;
              return Err(DbError::AccessRequestExpired(id_owned));
            }
          } else {
            return Err(DbError::AccessRequestNotDraft {
              id: id_owned,
              status: row.status.to_string(),
            });
          }

          let active = app_access_request_entity::ActiveModel {
            id: Set(row.id.clone()),
            status: Set(AppAccessRequestStatus::Approved),
            user_id: Set(Some(user_id_owned)),
            approved: Set(Some(approved_owned)),
            approved_role: Set(Some(approved_role_owned)),
            access_request_scope: Set(Some(access_request_scope_owned)),
            updated_at: Set(now),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(AppAccessRequest::from(model))
        })
      })
      .await
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let user_id_owned = user_id.to_string();

    // Look up tenant_id from the record first
    let row = app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          // validate_draft_for_update inline
          let row = app_access_request_entity::Entity::find_by_id(&id_owned)
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(AppAccessRequest::from)
            .ok_or_else(|| DbError::ItemNotFound {
              id: id_owned.clone(),
              item_type: "app_access_request".to_string(),
            })?;
          if row.status == AppAccessRequestStatus::Draft {
            if row.expires_at < now {
              let active = app_access_request_entity::ActiveModel {
                id: Set(id_owned.clone()),
                status: Set(AppAccessRequestStatus::Expired),
                updated_at: Set(now),
                ..Default::default()
              };
              active.update(txn).await.map_err(DbError::from)?;
              return Err(DbError::AccessRequestExpired(id_owned));
            }
          } else {
            return Err(DbError::AccessRequestNotDraft {
              id: id_owned,
              status: row.status.to_string(),
            });
          }

          let active = app_access_request_entity::ActiveModel {
            id: Set(row.id.clone()),
            status: Set(AppAccessRequestStatus::Denied),
            user_id: Set(Some(user_id_owned)),
            updated_at: Set(now),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(AppAccessRequest::from(model))
        })
      })
      .await
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let error_message_owned = error_message.to_string();

    // Look up tenant_id from the record first
    let row = app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;
    let tenant_id = row.tenant_id.clone();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
          // validate_draft_for_update inline
          let row = app_access_request_entity::Entity::find_by_id(&id_owned)
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(AppAccessRequest::from)
            .ok_or_else(|| DbError::ItemNotFound {
              id: id_owned.clone(),
              item_type: "app_access_request".to_string(),
            })?;
          if row.status == AppAccessRequestStatus::Draft {
            if row.expires_at < now {
              let active = app_access_request_entity::ActiveModel {
                id: Set(id_owned.clone()),
                status: Set(AppAccessRequestStatus::Expired),
                updated_at: Set(now),
                ..Default::default()
              };
              active.update(txn).await.map_err(DbError::from)?;
              return Err(DbError::AccessRequestExpired(id_owned));
            }
          } else {
            return Err(DbError::AccessRequestNotDraft {
              id: id_owned,
              status: row.status.to_string(),
            });
          }

          let active = app_access_request_entity::ActiveModel {
            id: Set(row.id.clone()),
            status: Set(AppAccessRequestStatus::Failed),
            error_message: Set(Some(error_message_owned)),
            updated_at: Set(now),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(AppAccessRequest::from(model))
        })
      })
      .await
  }

  async fn get_by_access_request_scope(
    &self,
    tenant_id: &str,
    scope: &str,
  ) -> Result<Option<AppAccessRequest>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let scope_owned = scope.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = app_access_request_entity::Entity::find()
            .filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(app_access_request_entity::Column::AccessRequestScope.eq(&scope_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.map(AppAccessRequest::from))
        })
      })
      .await
  }
}
