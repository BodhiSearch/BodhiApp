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

  /// Newest approved access request for `(tenant, app, user)`.
  async fn latest_approved_for_app_user(
    &self,
    tenant_id: &str,
    app_client_id: &str,
    user_id: &str,
  ) -> Result<Option<AppAccessRequest>, DbError>;

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

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = app_access_request_entity::Entity::find_by_id(&id_owned)
            .filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.map(AppAccessRequest::from))
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

  async fn latest_approved_for_app_user(
    &self,
    tenant_id: &str,
    app_client_id: &str,
    user_id: &str,
  ) -> Result<Option<AppAccessRequest>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let app_client_id_owned = app_client_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let row = app_access_request_entity::Entity::find()
            .filter(app_access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(app_access_request_entity::Column::AppClientId.eq(&app_client_id_owned))
            .filter(app_access_request_entity::Column::UserId.eq(&user_id_owned))
            .filter(app_access_request_entity::Column::Status.eq(AppAccessRequestStatus::Approved))
            .order_by_desc(app_access_request_entity::Column::CreatedAt)
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(row.map(AppAccessRequest::from))
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
