use super::access_request_objs::AppAccessRequest;
use super::app_access_request_entity;
use crate::db::{DbError, DefaultDbService};
use crate::AppAccessRequestStatus;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

#[async_trait]
pub trait AccessRequestRepository: Send + Sync {
  async fn create(&self, row: &AppAccessRequest) -> Result<AppAccessRequest, DbError>;

  async fn get(&self, tenant_id: &str, id: &str) -> Result<Option<AppAccessRequest>, DbError>;

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    tenant_id: &str,
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

  /// Approved access requests owned by `user_id` within `tenant_id`, newest first.
  async fn list_approved_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<AppAccessRequest>, DbError>;

  /// Transition an Approved request owned by `user_id` to Revoked.
  async fn update_revocation(
    &self,
    tenant_id: &str,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequest, DbError>;
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
      status: Set(row.status),
      requested: Set(row.requested.clone()),
      approved: Set(row.approved.clone()),
      user_id: Set(row.user_id.clone()),
      requested_role: Set(row.requested_role.clone()),
      approved_role: Set(row.approved_role.clone()),
      access_request_scope: Set(row.access_request_scope.clone()),
      source_access_request_id: Set(row.source_access_request_id.clone()),
      error_message: Set(row.error_message.clone()),
      expires_at: Set(row.expires_at),
      created_at: Set(row.created_at),
      updated_at: Set(row.updated_at),
    };
    let tenant_id = row.tenant_id.clone().unwrap_or_default();

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
    tenant_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let user_id_owned = user_id.to_string();
    let tenant_id_owned = tenant_id.to_string();
    let approved_owned = approved.to_string();
    let approved_role_owned = approved_role.to_string();
    let access_request_scope_owned = access_request_scope.to_string();

    // Look up the record directly (bypasses RLS) to confirm it exists
    app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;

    // Use the provided tenant_id for RLS context (binds draft to tenant)
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Re-read within transaction — RLS allows NULL tenant_id rows
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
            tenant_id: Set(Some(tenant_id_owned)),
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

    // Look up tenant_id from the record first (bypasses RLS)
    let row = app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;
    let tenant_id = row.tenant_id.clone().unwrap_or_default();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
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

    // Look up tenant_id from the record first (bypasses RLS)
    let row = app_access_request_entity::Entity::find_by_id(&id_owned)
      .one(&self.db)
      .await
      .map_err(DbError::from)?
      .ok_or_else(|| DbError::ItemNotFound {
        id: id_owned.clone(),
        item_type: "app_access_request".to_string(),
      })?;
    let tenant_id = row.tenant_id.clone().unwrap_or_default();

    self
      .with_tenant_txn(&tenant_id, |txn| {
        Box::pin(async move {
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

  async fn list_approved_for_user(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<AppAccessRequest>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let rows = app_access_request_entity::Entity::find()
            .filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(app_access_request_entity::Column::UserId.eq(&user_id_owned))
            .filter(app_access_request_entity::Column::Status.eq(AppAccessRequestStatus::Approved))
            .order_by_desc(app_access_request_entity::Column::CreatedAt)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(rows.into_iter().map(AppAccessRequest::from).collect())
        })
      })
      .await
  }

  async fn update_revocation(
    &self,
    tenant_id: &str,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let user_id_owned = user_id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Filter by tenant explicitly — defense-in-depth for SQLite (no RLS).
          let row = app_access_request_entity::Entity::find_by_id(&id_owned)
            .filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?
            .map(AppAccessRequest::from)
            .ok_or_else(|| DbError::ItemNotFound {
              id: id_owned.clone(),
              item_type: "app_access_request".to_string(),
            })?;

          // Only the owner may revoke, and only an Approved grant.
          if row.user_id.as_deref() != Some(user_id_owned.as_str()) {
            return Err(DbError::ItemNotFound {
              id: id_owned,
              item_type: "app_access_request".to_string(),
            });
          }
          if row.status != AppAccessRequestStatus::Approved {
            return Err(DbError::AccessRequestNotDraft {
              id: id_owned,
              status: row.status.to_string(),
            });
          }

          let active = app_access_request_entity::ActiveModel {
            id: Set(row.id.clone()),
            status: Set(AppAccessRequestStatus::Revoked),
            updated_at: Set(now),
            ..Default::default()
          };
          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(AppAccessRequest::from(model))
        })
      })
      .await
  }
}
