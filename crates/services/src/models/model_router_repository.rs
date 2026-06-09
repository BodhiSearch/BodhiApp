use crate::db::{DbError, DefaultDbService};
use crate::models::model_router_entity::{self as model_router};
use crate::models::ModelRouterAlias;

use sea_orm::prelude::*;
use sea_orm::{PaginatorTrait, QueryOrder, Set};

#[async_trait::async_trait]
pub trait ModelRouterRepository: Send + Sync {
  async fn create_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &ModelRouterAlias,
  ) -> Result<(), DbError>;

  async fn get_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ModelRouterAlias>, DbError>;

  async fn update_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    alias: &ModelRouterAlias,
  ) -> Result<(), DbError>;

  async fn delete_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError>;

  async fn list_model_router_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ModelRouterAlias>, DbError>;

  /// Whether a router with `alias` name already exists for this (tenant, user),
  /// optionally excluding a given id (for update checks).
  async fn check_router_alias_exists(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;
}

#[async_trait::async_trait]
impl ModelRouterRepository for DefaultDbService {
  async fn create_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &ModelRouterAlias,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let alias = alias.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let model = model_router::ActiveModel {
            id: Set(alias.id.clone()),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(user_id),
            alias: Set(alias.alias.clone()),
            targets: Set(alias.targets.clone().into()),
            strategy: Set(alias.strategy.clone()),
            created_at: Set(alias.created_at),
            updated_at: Set(alias.updated_at),
          };
          model_router::Entity::insert(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ModelRouterAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = model_router::Entity::find_by_id(id_owned)
            .filter(model_router::Column::TenantId.eq(&tenant_id_owned))
            .filter(model_router::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result.map(Into::into))
        })
      })
      .await
  }

  async fn update_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    alias: &ModelRouterAlias,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();
    let alias = alias.clone();
    let now = self.time_service.utc_now();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let exists = model_router::Entity::find()
            .filter(model_router::Column::Id.eq(&id))
            .filter(model_router::Column::TenantId.eq(&tenant_id_owned))
            .filter(model_router::Column::UserId.eq(&user_id))
            .count(txn)
            .await
            .map_err(DbError::from)?
            > 0;
          if !exists {
            return Err(DbError::ItemNotFound {
              id: id.clone(),
              item_type: "model_router".to_string(),
            });
          }

          let active = model_router::ActiveModel {
            id: Set(id.clone()),
            alias: Set(alias.alias.clone()),
            targets: Set(alias.targets.clone().into()),
            strategy: Set(alias.strategy.clone()),
            updated_at: Set(now),
            ..Default::default()
          };
          model_router::Entity::update(active)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn delete_model_router_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let res = model_router::Entity::delete_many()
            .filter(model_router::Column::TenantId.eq(&tenant_id_owned))
            .filter(model_router::Column::UserId.eq(&user_id))
            .filter(model_router::Column::Id.eq(&id))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          if res.rows_affected == 0 {
            return Err(DbError::ItemNotFound {
              id: id.clone(),
              item_type: "model_router".to_string(),
            });
          }
          Ok(())
        })
      })
      .await
  }

  async fn list_model_router_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ModelRouterAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = model_router::Entity::find()
            .filter(model_router::Column::TenantId.eq(&tenant_id_owned))
            .filter(model_router::Column::UserId.eq(&user_id_owned))
            .order_by_desc(model_router::Column::CreatedAt)
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results.into_iter().map(Into::into).collect())
        })
      })
      .await
  }

  async fn check_router_alias_exists(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let alias_owned = alias.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = model_router::Entity::find()
            .filter(model_router::Column::TenantId.eq(&tenant_id_owned))
            .filter(model_router::Column::UserId.eq(&user_id_owned))
            .filter(model_router::Column::Alias.eq(&alias_owned));
          if let Some(id) = exclude_id {
            query = query.filter(model_router::Column::Id.ne(id));
          }
          let count = query.count(txn).await.map_err(DbError::from)?;
          Ok(count > 0)
        })
      })
      .await
  }
}
