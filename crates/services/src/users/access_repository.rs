use super::access_request_entity::{self, UserAccessRequestEntity};
use crate::db::{DbError, DefaultDbService};
use crate::UserAccessRequestStatus;
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, Set};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait AccessRepository: Send + Sync {
  async fn insert_pending_request(
    &self,
    tenant_id: &str,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequestEntity, DbError>;

  async fn get_pending_request(
    &self,
    tenant_id: &str,
    user_id: String,
  ) -> Result<Option<UserAccessRequestEntity>, DbError>;

  async fn list_pending_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError>;

  async fn list_all_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError>;

  async fn update_request_status(
    &self,
    tenant_id: &str,
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError>;

  async fn get_request_by_id(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<UserAccessRequestEntity>, DbError>;
}

#[async_trait::async_trait]
impl AccessRepository for DefaultDbService {
  async fn insert_pending_request(
    &self,
    tenant_id: &str,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequestEntity, DbError> {
    let now = self.time_service.utc_now();
    let id = crate::new_ulid();

    let model = access_request_entity::ActiveModel {
      id: Set(id.clone()),
      tenant_id: Set(tenant_id.to_string()),
      username: Set(username),
      user_id: Set(user_id),
      reviewer: Set(None),
      status: Set(UserAccessRequestStatus::Pending),
      created_at: Set(now),
      updated_at: Set(now),
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          access_request_entity::Entity::insert(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;

          access_request_entity::Entity::find_by_id(id)
            .one(txn)
            .await
            .map_err(DbError::from)?
            .ok_or_else(|| DbError::from(sea_orm::DbErr::RecordNotInserted))
        })
      })
      .await
  }

  async fn get_pending_request(
    &self,
    tenant_id: &str,
    user_id: String,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = access_request_entity::Entity::find()
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(access_request_entity::Column::UserId.eq(user_id))
            .filter(
              access_request_entity::Column::Status
                .eq(UserAccessRequestStatus::Pending.to_string()),
            )
            .one(txn)
            .await
            .map_err(DbError::from)?;

          Ok(result)
        })
      })
      .await
  }

  async fn list_pending_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    let offset = ((page - 1) * per_page) as u64;
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let total = access_request_entity::Entity::find()
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(
              access_request_entity::Column::Status
                .eq(UserAccessRequestStatus::Pending.to_string()),
            )
            .count(txn)
            .await
            .map_err(DbError::from)? as usize;

          let results = access_request_entity::Entity::find()
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(
              access_request_entity::Column::Status
                .eq(UserAccessRequestStatus::Pending.to_string()),
            )
            .order_by_asc(access_request_entity::Column::CreatedAt)
            .offset(offset)
            .limit(per_page as u64)
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok((results, total))
        })
      })
      .await
  }

  async fn list_all_requests(
    &self,
    tenant_id: &str,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequestEntity>, usize), DbError> {
    let offset = ((page - 1) * per_page) as u64;
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let total = access_request_entity::Entity::find()
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .count(txn)
            .await
            .map_err(DbError::from)? as usize;

          let results = access_request_entity::Entity::find()
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .order_by_asc(access_request_entity::Column::CreatedAt)
            .offset(offset)
            .limit(per_page as u64)
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok((results, total))
        })
      })
      .await
  }

  async fn update_request_status(
    &self,
    tenant_id: &str,
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let id_owned = id.to_string();
    let tenant_id_owned = tenant_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let existing = access_request_entity::Entity::find_by_id(id_owned.clone())
            .filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          if existing.is_none() {
            return Err(DbError::ItemNotFound {
              id: id_owned,
              item_type: "user_access_request".to_string(),
            });
          }

          let active = access_request_entity::ActiveModel {
            id: Set(id_owned),
            status: Set(status),
            reviewer: Set(Some(reviewer)),
            updated_at: Set(now),
            ..Default::default()
          };

          access_request_entity::Entity::update(active)
            .exec(txn)
            .await
            .map_err(DbError::from)?;

          Ok(())
        })
      })
      .await
  }

  async fn get_request_by_id(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<UserAccessRequestEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = access_request_entity::Entity::find_by_id(id_owned);
          if !tenant_id_owned.is_empty() {
            query = query.filter(access_request_entity::Column::TenantId.eq(&tenant_id_owned));
          }
          let result = query.one(txn).await.map_err(DbError::from)?;

          Ok(result)
        })
      })
      .await
  }
}
