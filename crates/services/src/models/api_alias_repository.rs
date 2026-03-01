use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  ApiKeyUpdate, DbError, DefaultDbService,
};
use crate::models::api_model_alias_entity::{self as api_model_alias, ApiAliasView};
use crate::models::ApiAlias;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, Set};

#[async_trait::async_trait]
pub trait ApiAliasRepository: Send + Sync {
  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError>;

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError>;

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError>;

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError>;

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError>;

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;
}

#[async_trait::async_trait]
impl ApiAliasRepository for DefaultDbService {
  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    if let Some(ref prefix) = alias.prefix {
      if !prefix.is_empty() && self.check_prefix_exists(prefix, None).await? {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let (encrypted_api_key, salt, nonce) = if let Some(ref key) = api_key {
      let (enc, s, n) = encrypt_api_key(&self.encryption_key, key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let model = api_model_alias::ActiveModel {
      id: Set(alias.id.clone()),
      api_format: Set(alias.api_format.clone()),
      base_url: Set(alias.base_url.clone()),
      models: Set(alias.models.clone()),
      prefix: Set(alias.prefix.clone()),
      forward_all_with_prefix: Set(alias.forward_all_with_prefix),
      models_cache: Set(alias.models_cache.clone()),
      cache_fetched_at: Set(alias.cache_fetched_at),
      encrypted_api_key: Set(encrypted_api_key),
      salt: Set(salt),
      nonce: Set(nonce),
      created_at: Set(alias.created_at),
      updated_at: Set(alias.updated_at),
    };
    api_model_alias::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError> {
    let result = api_model_alias::Entity::find_by_id(id.to_string())
      .into_partial_model::<ApiAliasView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result.map(Into::into))
  }

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError> {
    if let Some(ref prefix) = model.prefix {
      if !prefix.is_empty()
        && self
          .check_prefix_exists(prefix, Some(id.to_string()))
          .await?
      {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let now = self.time_service.utc_now();

    let mut active = api_model_alias::ActiveModel {
      id: Set(id.to_string()),
      api_format: Set(model.api_format.clone()),
      base_url: Set(model.base_url.clone()),
      models: Set(model.models.clone()),
      prefix: Set(model.prefix.clone()),
      forward_all_with_prefix: Set(model.forward_all_with_prefix),
      updated_at: Set(now),
      ..Default::default()
    };

    match api_key {
      ApiKeyUpdate::Set(api_key_opt) => match api_key_opt {
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
      ApiKeyUpdate::Keep => {}
    }

    api_model_alias::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    let active = api_model_alias::ActiveModel {
      id: Set(id.to_string()),
      models_cache: Set(models.into()),
      cache_fetched_at: Set(fetched_at),
      ..Default::default()
    };

    api_model_alias::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError> {
    api_model_alias::Entity::delete_by_id(id.to_string())
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError> {
    let results = api_model_alias::Entity::find()
      .order_by_desc(api_model_alias::Column::CreatedAt)
      .into_partial_model::<ApiAliasView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError> {
    let result = api_model_alias::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(m) => match (m.encrypted_api_key, m.salt, m.nonce) {
        (Some(encrypted), Some(salt), Some(nonce)) => {
          let api_key = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          Ok(Some(api_key))
        }
        (None, None, None) => Ok(None),
        _ => Err(DbError::EncryptionError(format!(
          "Data corruption: API key encryption fields are partially NULL for alias '{}'",
          id
        ))),
      },
      None => Ok(None),
    }
  }

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    let mut query =
      api_model_alias::Entity::find().filter(api_model_alias::Column::Prefix.eq(prefix));

    if let Some(id) = exclude_id {
      query = query.filter(api_model_alias::Column::Id.ne(id));
    }

    let count = query.count(&self.db).await.map_err(DbError::from)?;
    Ok(count > 0)
  }
}
