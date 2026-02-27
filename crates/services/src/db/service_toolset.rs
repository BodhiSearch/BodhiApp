use crate::db::{
  encryption::decrypt_api_key,
  entities::{app_toolset_config, toolset},
  ApiKeyUpdate, AppToolsetConfigRow, DbError, DefaultDbService, ToolsetRepository, ToolsetRow,
};
use sea_orm::prelude::*;
use sea_orm::sea_query::Alias;
use sea_orm::{Condition, Set};

#[async_trait::async_trait]
impl ToolsetRepository for DefaultDbService {
  async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError> {
    let result = toolset::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result.map(ToolsetRow::from))
  }

  async fn get_toolset_by_slug(
    &self,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetRow>, DbError> {
    let result = toolset::Entity::find()
      .filter(toolset::Column::UserId.eq(user_id))
      .filter(
        Expr::expr(Expr::col(toolset::Column::Slug).cast_as(Alias::new("TEXT")))
          .eq(Expr::val(slug).cast_as(Alias::new("TEXT"))),
      )
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result.map(ToolsetRow::from))
  }

  async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError> {
    let model = toolset::ActiveModel {
      id: Set(row.id.clone()),
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

    let inserted = model.insert(&self.db).await.map_err(DbError::from)?;
    Ok(ToolsetRow::from(inserted))
  }

  async fn update_toolset(
    &self,
    row: &ToolsetRow,
    api_key_update: ApiKeyUpdate,
  ) -> Result<ToolsetRow, DbError> {
    let mut active: toolset::ActiveModel = Default::default();
    active.id = Set(row.id.clone());
    active.slug = Set(row.slug.clone());
    active.description = Set(row.description.clone());
    active.enabled = Set(row.enabled);
    active.updated_at = Set(row.updated_at);

    match api_key_update {
      ApiKeyUpdate::Keep => {}
      ApiKeyUpdate::Set(api_key) => {
        active.encrypted_api_key = Set(api_key);
        active.salt = Set(row.salt.clone());
        active.nonce = Set(row.nonce.clone());
      }
    }

    toolset::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(row.clone())
  }

  async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError> {
    let results = toolset::Entity::find()
      .filter(toolset::Column::UserId.eq(user_id))
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results.into_iter().map(ToolsetRow::from).collect())
  }

  async fn list_toolsets_by_toolset_type(
    &self,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetRow>, DbError> {
    let results = toolset::Entity::find()
      .filter(
        Condition::all()
          .add(toolset::Column::UserId.eq(user_id))
          .add(toolset::Column::ToolsetType.eq(toolset_type)),
      )
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results.into_iter().map(ToolsetRow::from).collect())
  }

  async fn delete_toolset(&self, id: &str) -> Result<(), DbError> {
    toolset::Entity::delete_by_id(id.to_string())
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError> {
    let result = self.get_toolset(id).await?;

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
    toolset_type: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<AppToolsetConfigRow, DbError> {
    let now = self.time_service.utc_now();

    let existing = app_toolset_config::Entity::find()
      .filter(app_toolset_config::Column::ToolsetType.eq(toolset_type))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match existing {
      Some(existing_model) => {
        let mut active: app_toolset_config::ActiveModel = Default::default();
        active.id = Set(existing_model.id.clone());
        active.enabled = Set(enabled);
        active.updated_by = Set(updated_by.to_string());
        active.updated_at = Set(now);

        app_toolset_config::Entity::update(active)
          .exec(&self.db)
          .await
          .map_err(DbError::from)?;

        Ok(AppToolsetConfigRow {
          id: existing_model.id,
          toolset_type: toolset_type.to_string(),
          enabled,
          updated_by: updated_by.to_string(),
          created_at: existing_model.created_at,
          updated_at: now,
        })
      }
      None => {
        let id = ulid::Ulid::new().to_string();
        let model = app_toolset_config::ActiveModel {
          id: Set(id.clone()),
          toolset_type: Set(toolset_type.to_string()),
          enabled: Set(enabled),
          updated_by: Set(updated_by.to_string()),
          created_at: Set(now),
          updated_at: Set(now),
        };

        app_toolset_config::Entity::insert(model)
          .exec(&self.db)
          .await
          .map_err(DbError::from)?;

        Ok(AppToolsetConfigRow {
          id,
          toolset_type: toolset_type.to_string(),
          enabled,
          updated_by: updated_by.to_string(),
          created_at: now,
          updated_at: now,
        })
      }
    }
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    let results = app_toolset_config::Entity::find()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results.into_iter().map(AppToolsetConfigRow::from).collect())
  }

  async fn get_app_toolset_config(
    &self,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    let result = app_toolset_config::Entity::find()
      .filter(app_toolset_config::Column::ToolsetType.eq(toolset_type))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result.map(AppToolsetConfigRow::from))
  }
}
