use crate::db::{encryption::decrypt_api_key, DbError, DefaultDbService};
use crate::RawApiKeyUpdate;
use sea_orm::prelude::*;
use sea_orm::sea_query::Alias;
use sea_orm::{Condition, Set};

use super::app_toolset_config_entity as app_toolset_config;
use super::app_toolset_config_entity::AppToolsetConfigEntity;
use super::toolset_entity as toolset;
use super::toolset_entity::ToolsetEntity;

#[async_trait::async_trait]
pub trait ToolsetRepository: Send + Sync {
  // Toolset instances
  async fn get_toolset(&self, tenant_id: &str, id: &str) -> Result<Option<ToolsetEntity>, DbError>;

  async fn get_toolset_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetEntity>, DbError>;

  async fn create_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
  ) -> Result<ToolsetEntity, DbError>;

  async fn update_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
    api_key_update: RawApiKeyUpdate,
  ) -> Result<ToolsetEntity, DbError>;

  async fn list_toolsets(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError>;

  async fn list_toolsets_by_toolset_type(
    &self,
    tenant_id: &str,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError>;

  async fn delete_toolset(&self, tenant_id: &str, id: &str) -> Result<(), DbError>;

  async fn get_toolset_api_key(&self, tenant_id: &str, id: &str)
    -> Result<Option<String>, DbError>;

  // App-level toolset type config
  async fn set_app_toolset_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigEntity, DbError>;

  async fn list_app_toolset_configs(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<AppToolsetConfigEntity>, DbError>;

  async fn get_app_toolset_config(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigEntity>, DbError>;
}

#[async_trait::async_trait]
impl ToolsetRepository for DefaultDbService {
  async fn get_toolset(&self, tenant_id: &str, id: &str) -> Result<Option<ToolsetEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id = id.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = toolset::Entity::find_by_id(id)
            .filter(toolset::Column::TenantId.eq(&*tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn get_toolset_by_slug(
    &self,
    tenant_id: &str,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let slug = slug.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = toolset::Entity::find()
            .filter(toolset::Column::TenantId.eq(&*tenant_id_owned))
            .filter(toolset::Column::UserId.eq(&*user_id))
            .filter(
              Expr::expr(Expr::col(toolset::Column::Slug).cast_as(Alias::new("TEXT")))
                .eq(Expr::val(&*slug).cast_as(Alias::new("TEXT"))),
            )
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }

  async fn create_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
  ) -> Result<ToolsetEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let model = toolset::ActiveModel {
            id: Set(row.id.clone()),
            tenant_id: Set(tenant_id_owned),
            user_id: Set(row.user_id.clone()),
            toolset_type: Set(row.toolset_type.clone()),
            slug: Set(row.slug.clone()),
            description: Set(row.description.clone()),
            enabled: Set(row.enabled),
            encrypted_api_key: Set(row.encrypted_api_key.clone()),
            salt: Set(row.salt.clone()),
            nonce: Set(row.nonce.clone()),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
          };
          let inserted = model.insert(txn).await.map_err(DbError::from)?;
          Ok(inserted)
        })
      })
      .await
  }

  async fn update_toolset(
    &self,
    tenant_id: &str,
    row: &ToolsetEntity,
    api_key_update: RawApiKeyUpdate,
  ) -> Result<ToolsetEntity, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let row = row.clone();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          // Verify tenant ownership before update
          let existing = toolset::Entity::find_by_id(row.id.clone())
            .filter(toolset::Column::TenantId.eq(&*tenant_id_owned))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          if existing.is_none() {
            return Err(DbError::ItemNotFound {
              id: row.id.clone(),
              item_type: "toolset".to_string(),
            });
          }

          let mut active = toolset::ActiveModel {
            id: Set(row.id.clone()),
            slug: Set(row.slug.clone()),
            description: Set(row.description.clone()),
            enabled: Set(row.enabled),
            updated_at: Set(row.updated_at),
            ..Default::default()
          };

          match api_key_update {
            RawApiKeyUpdate::Keep => {}
            RawApiKeyUpdate::Set(api_key) => {
              active.encrypted_api_key = Set(api_key);
              active.salt = Set(row.salt.clone());
              active.nonce = Set(row.nonce.clone());
            }
          }

          let model = active.update(txn).await.map_err(DbError::from)?;
          Ok(model)
        })
      })
      .await
  }

  async fn list_toolsets(
    &self,
    tenant_id: &str,
    user_id: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = toolset::Entity::find()
            .filter(toolset::Column::TenantId.eq(&*tenant_id_owned))
            .filter(toolset::Column::UserId.eq(&*user_id))
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn list_toolsets_by_toolset_type(
    &self,
    tenant_id: &str,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let user_id = user_id.to_string();
    let toolset_type = toolset_type.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = toolset::Entity::find()
            .filter(
              Condition::all()
                .add(toolset::Column::TenantId.eq(&*tenant_id_owned))
                .add(toolset::Column::UserId.eq(&*user_id))
                .add(toolset::Column::ToolsetType.eq(&*toolset_type)),
            )
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn delete_toolset(&self, tenant_id: &str, id: &str) -> Result<(), DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let id = id.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          toolset::Entity::delete_many()
            .filter(toolset::Column::TenantId.eq(&*tenant_id_owned))
            .filter(toolset::Column::Id.eq(&*id))
            .exec(txn)
            .await
            .map_err(DbError::from)?;
          Ok(())
        })
      })
      .await
  }

  async fn get_toolset_api_key(
    &self,
    tenant_id: &str,
    id: &str,
  ) -> Result<Option<String>, DbError> {
    let result = self.get_toolset(tenant_id, id).await?;

    if let Some(row) = result {
      if let (Some(encrypted), Some(salt), Some(nonce)) =
        (row.encrypted_api_key, row.salt, row.nonce)
      {
        let api_key = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        return Ok(Some(api_key));
      }
    }

    Ok(None)
  }

  async fn set_app_toolset_enabled(
    &self,
    tenant_id: &str,
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigEntity, DbError> {
    let now = self.time_service.utc_now();
    let tenant_id_owned = tenant_id.to_string();
    let toolset_type = toolset_type.to_string();
    let updated_by = updated_by.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let existing = app_toolset_config::Entity::find()
            .filter(app_toolset_config::Column::TenantId.eq(&*tenant_id_owned))
            .filter(app_toolset_config::Column::ToolsetType.eq(&*toolset_type))
            .one(txn)
            .await
            .map_err(DbError::from)?;

          match existing {
            Some(existing_model) => {
              let active = app_toolset_config::ActiveModel {
                id: Set(existing_model.id.clone()),
                enabled: Set(enabled),
                updated_by: Set(updated_by.clone()),
                updated_at: Set(now),
                ..Default::default()
              };

              app_toolset_config::Entity::update(active)
                .exec(txn)
                .await
                .map_err(DbError::from)?;

              Ok(AppToolsetConfigEntity {
                id: existing_model.id,
                tenant_id: tenant_id_owned.clone(),
                toolset_type: toolset_type.clone(),
                enabled,
                updated_by,
                created_at: existing_model.created_at,
                updated_at: now,
              })
            }
            None => {
              let id = crate::new_ulid();
              let model = app_toolset_config::ActiveModel {
                id: Set(id.clone()),
                tenant_id: Set(tenant_id_owned.clone()),
                toolset_type: Set(toolset_type.clone()),
                enabled: Set(enabled),
                updated_by: Set(updated_by.clone()),
                created_at: Set(now),
                updated_at: Set(now),
              };

              app_toolset_config::Entity::insert(model)
                .exec(txn)
                .await
                .map_err(DbError::from)?;

              Ok(AppToolsetConfigEntity {
                id,
                tenant_id: tenant_id_owned,
                toolset_type,
                enabled,
                updated_by,
                created_at: now,
                updated_at: now,
              })
            }
          }
        })
      })
      .await
  }

  async fn list_app_toolset_configs(
    &self,
    tenant_id: &str,
  ) -> Result<Vec<AppToolsetConfigEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let results = app_toolset_config::Entity::find()
            .filter(app_toolset_config::Column::TenantId.eq(&*tenant_id_owned))
            .all(txn)
            .await
            .map_err(DbError::from)?;
          Ok(results)
        })
      })
      .await
  }

  async fn get_app_toolset_config(
    &self,
    tenant_id: &str,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigEntity>, DbError> {
    let tenant_id_owned = tenant_id.to_string();
    let toolset_type = toolset_type.to_string();
    self
      .with_tenant_txn(tenant_id, |txn| {
        Box::pin(async move {
          let result = app_toolset_config::Entity::find()
            .filter(app_toolset_config::Column::TenantId.eq(&*tenant_id_owned))
            .filter(app_toolset_config::Column::ToolsetType.eq(&*toolset_type))
            .one(txn)
            .await
            .map_err(DbError::from)?;
          Ok(result)
        })
      })
      .await
  }
}
