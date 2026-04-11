use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  DbError, DefaultDbService,
};
use crate::models::api_model_alias_entity::{self as api_model_alias, ApiAliasView};
use crate::models::{ApiAlias, ApiModel};
use crate::RawApiKeyUpdate;

use sea_orm::prelude::*;
use sea_orm::{PaginatorTrait, QueryOrder, Set};

#[async_trait::async_trait]
pub trait ApiAliasRepository: Send + Sync {
  async fn create_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError>;

  async fn get_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiAlias>, DbError>;

  async fn update_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    model: &ApiAlias,
    api_key: RawApiKeyUpdate,
  ) -> Result<(), DbError>;

  async fn update_api_model_models(
    &self,
    tenant_id: &str,
    id: &str,
    models: Vec<ApiModel>,
  ) -> Result<(), DbError>;

  async fn delete_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), DbError>;

  async fn list_api_model_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ApiAlias>, DbError>;

  async fn get_api_key_for_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<String>, DbError>;

  async fn check_prefix_exists(
    &self,
    tenant_id: &str,
    user_id: &str,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;
}

#[async_trait::async_trait]
impl ApiAliasRepository for DefaultDbService {
  async fn create_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let alias = alias.clone();

    let (encrypted_api_key, salt, nonce) = if let Some(ref key) = api_key {
      let (enc, s, n) = encrypt_api_key(&self.encryption_key, key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          if let Some(ref prefix) = alias.prefix {
            if !prefix.is_empty() {
              let count = api_model_alias::Entity::find()
                .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
                .filter(api_model_alias::Column::UserId.eq(&user_id))
                .filter(api_model_alias::Column::Prefix.eq(prefix.as_str()))
                .count(txn)
                .await
                .map_err(DbError::from)?;
              if count > 0 {
                return Err(DbError::PrefixExists(prefix.clone()));
              }
            }
          }

          let model = api_model_alias::ActiveModel {
            id: Set(alias.id.clone()),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(user_id),
            api_format: Set(alias.api_format.clone()),
            base_url: Set(alias.base_url.clone()),
            models: Set(alias.models.clone()),
            prefix: Set(alias.prefix.clone()),
            forward_all_with_prefix: Set(alias.forward_all_with_prefix),
            encrypted_api_key: Set(encrypted_api_key),
            salt: Set(salt),
            nonce: Set(nonce),
            created_at: Set(alias.created_at),
            updated_at: Set(alias.updated_at),
          };
          api_model_alias::Entity::insert(model)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = api_model_alias::Entity::find_by_id(id_owned)
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id_owned))
            .into_partial_model::<ApiAliasView>()
            .one(txn)
            .await
            .map_err(DbError::from)?;

          Ok(result.map(Into::into))
        })
      })
      .await
  }

  async fn update_api_model_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    model: &ApiAlias,
    api_key: RawApiKeyUpdate,
  ) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let id = id.to_string();
    let model = model.clone();

    let now = self.time_service.utc_now();

    let mut active = api_model_alias::ActiveModel {
      id: Set(id.clone()),
      api_format: Set(model.api_format.clone()),
      base_url: Set(model.base_url.clone()),
      models: Set(model.models.clone()),
      prefix: Set(model.prefix.clone()),
      forward_all_with_prefix: Set(model.forward_all_with_prefix),
      updated_at: Set(now),
      ..Default::default()
    };

    match api_key {
      RawApiKeyUpdate::Set(api_key_opt) => match api_key_opt {
        Some(api_key) => {
          let (encrypted, s, n) = encrypt_api_key(&self.encryption_key, &api_key)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          active.encrypted_api_key = Set(Some(encrypted));
          active.salt = Set(Some(s));
          active.nonce = Set(Some(n));
        }
        None => {
          active.encrypted_api_key = Set(None);
          active.salt = Set(None);
          active.nonce = Set(None);
        }
      },
      RawApiKeyUpdate::Keep => {}
    }

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          if let Some(ref prefix) = model.prefix {
            if !prefix.is_empty() {
              let mut query = api_model_alias::Entity::find()
                .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
                .filter(api_model_alias::Column::UserId.eq(&user_id))
                .filter(api_model_alias::Column::Prefix.eq(prefix.as_str()));
              query = query.filter(api_model_alias::Column::Id.ne(&id));
              let count = query.count(txn).await.map_err(DbError::from)?;
              if count > 0 {
                return Err(DbError::PrefixExists(prefix.clone()));
              }
            }
          }

          // Verify ownership before updating: only update if tenant_id and user_id match
          let exists = api_model_alias::Entity::find()
            .filter(api_model_alias::Column::Id.eq(&id))
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id))
            .count(txn)
            .await
            .map_err(DbError::from)?
            > 0;

          if exists {
            api_model_alias::Entity::update(active)
              .exec(txn)
              .await
              .map_err(DbError::from)?;
          }
          Ok(())
        })
      })
      .await
  }

  async fn update_api_model_models(
    &self,
    tenant_id: &str,
    id: &str,
    models: Vec<ApiModel>,
  ) -> Result<(), DbError> {
    let id = id.to_string();
    let now = self.time_service.utc_now();

    let active = api_model_alias::ActiveModel {
      id: Set(id),
      models: Set(models.into()),
      updated_at: Set(now),
      ..Default::default()
    };

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          api_model_alias::Entity::update(active)
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn delete_api_model_alias(
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
          api_model_alias::Entity::delete_many()
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id))
            .filter(api_model_alias::Column::Id.eq(&id))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn list_api_model_aliases(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ApiAlias>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = api_model_alias::Entity::find()
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id_owned))
            .order_by_desc(api_model_alias::Column::CreatedAt)
            .into_partial_model::<ApiAliasView>()
            .all(txn)
            .await
            .map_err(DbError::from)?;

          Ok(results.into_iter().map(Into::into).collect())
        })
      })
      .await
  }

  async fn get_api_key_for_alias(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<String>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let id_owned = id.to_string();
    let encryption_key = self.encryption_key.clone();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = api_model_alias::Entity::find_by_id(id_owned.clone())
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match result {
            Some(m) => match (m.encrypted_api_key, m.salt, m.nonce) {
              (Some(encrypted), Some(salt), Some(nonce)) => {
                let api_key = decrypt_api_key(&encryption_key, &encrypted, &salt, &nonce)
                  .map_err(|e| DbError::EncryptionError(e.to_string()))?;
                Ok(Some(api_key))
              }
              (None, None, None) => Ok(None),
              _ => Err(DbError::EncryptionError(format!(
                "Data corruption: API key encryption fields are partially NULL for alias '{}'",
                id_owned
              ))),
            },
            None => Ok(None),
          }
        })
      })
      .await
  }

  async fn check_prefix_exists(
    &self,
    tenant_id: &str,
    user_id: &str,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id_owned = user_id.to_string();
    let prefix_owned = prefix.to_string();

    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let mut query = api_model_alias::Entity::find()
            .filter(api_model_alias::Column::TenantId.eq(&tenant_id_owned))
            .filter(api_model_alias::Column::UserId.eq(&user_id_owned))
            .filter(api_model_alias::Column::Prefix.eq(&prefix_owned));

          if let Some(id) = exclude_id {
            query = query.filter(api_model_alias::Column::Id.ne(id));
          }

          let count = query.count(txn).await.map_err(DbError::from)?;
          Ok(count > 0)
        })
      })
      .await
  }
}
