use super::api_token_entity::{self, TokenEntity};
use crate::db::{DbError, DefaultDbService};
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, Set};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TokenRepository: Send + Sync {
  async fn create_api_token(&self, tenant_id: &str, token: &mut TokenEntity)
    -> Result<(), DbError>;

  async fn list_api_tokens(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<TokenEntity>, usize), DbError>;

  async fn get_api_token_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<TokenEntity>, DbError>;

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<TokenEntity>, DbError>;

  async fn update_api_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), DbError>;
}

#[async_trait::async_trait]
impl TokenRepository for DefaultDbService {
  async fn create_api_token(
    &self,
    tenant_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    token.created_at = now;
    token.updated_at = now;
    token.tenant_id = tenant_id.to_string();

    let model = api_token_entity::ActiveModel {
      id: Set(token.id.clone()),
      tenant_id: Set(tenant_id.to_string()),
      user_id: Set(token.user_id.clone()),
      name: Set(token.name.clone()),
      token_prefix: Set(token.token_prefix.clone()),
      token_hash: Set(token.token_hash.clone()),
      scopes: Set(token.scopes.clone()),
      status: Set(token.status.clone()),
      created_at: Set(token.created_at),
      updated_at: Set(token.updated_at),
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          api_token_entity::Entity::insert(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn list_api_tokens(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<TokenEntity>, usize), DbError> {
    let page = page.max(1);
    let offset = ((page - 1) * per_page) as u64;
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let total = api_token_entity::Entity::find()
            .filter(api_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_token_entity::Column::UserId.eq(&user_id_owned))
            .count(txn)
            .await
            .map_err(DbError::from)? as usize;

          let results = api_token_entity::Entity::find()
            .filter(api_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_token_entity::Column::UserId.eq(&user_id_owned))
            .order_by_desc(api_token_entity::Column::CreatedAt)
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

  async fn get_api_token_by_id(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<TokenEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = api_token_entity::Entity::find_by_id(id_owned)
            .filter(api_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_token_entity::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<TokenEntity>, DbError> {
    let result = api_token_entity::Entity::find()
      .filter(api_token_entity::Column::TokenPrefix.eq(prefix))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }

  async fn update_api_token(
    &self,
    tenant_id: &str,
    user_id: &str,
    token: &mut TokenEntity,
  ) -> Result<(), DbError> {
    token.updated_at = self.time_service.utc_now();
    let token_id = token.id.clone();
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let active = api_token_entity::ActiveModel {
      id: Set(token.id.clone()),
      name: Set(token.name.clone()),
      status: Set(token.status.clone()),
      updated_at: Set(token.updated_at),
      ..Default::default()
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let existing = api_token_entity::Entity::find_by_id(token_id.clone())
            .filter(api_token_entity::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_token_entity::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          if existing.is_none() {
            return Err(DbError::ItemNotFound {
              id: token_id,
              item_type: "api_token".to_string(),
            });
          }

          api_token_entity::Entity::update(active)
            .exec(txn)
            .await
            .map_err(DbError::from)?;

          Ok(())
        })
      })
      .await
  }
}
