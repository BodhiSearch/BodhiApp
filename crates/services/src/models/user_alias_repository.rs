use crate::db::{DbError, DefaultDbService};
use crate::models::user_alias_entity as user_alias;
use crate::models::UserAlias;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

#[async_trait::async_trait]
pub trait UserAliasRepository: Send + Sync {
  async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError>;
  async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError>;
  async fn get_user_alias_by_name(&self, alias: &str) -> Result<Option<UserAlias>, DbError>;
  async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError>;
  async fn delete_user_alias(&self, id: &str) -> Result<(), DbError>;
  async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError>;
}

#[async_trait::async_trait]
impl UserAliasRepository for DefaultDbService {
  async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError> {
    let active = user_alias::ActiveModel {
      id: Set(alias.id.clone()),
      alias: Set(alias.alias.clone()),
      repo: Set(alias.repo.to_string()),
      filename: Set(alias.filename.clone()),
      snapshot: Set(alias.snapshot.clone()),
      request_params: Set(alias.request_params.clone()),
      context_params: Set(alias.context_params.clone()),
      created_at: Set(alias.created_at),
      updated_at: Set(alias.updated_at),
    };
    user_alias::Entity::insert(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError> {
    let result = user_alias::Entity::find_by_id(id)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    match result {
      Some(model) => Ok(Some(UserAlias::try_from(model).map_err(|e| {
        DbError::Conversion(format!("Failed to convert user_alias: {}", e))
      })?)),
      None => Ok(None),
    }
  }

  async fn get_user_alias_by_name(&self, alias_name: &str) -> Result<Option<UserAlias>, DbError> {
    let result = user_alias::Entity::find()
      .filter(user_alias::Column::Alias.eq(alias_name))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    match result {
      Some(model) => Ok(Some(UserAlias::try_from(model).map_err(|e| {
        DbError::Conversion(format!("Failed to convert user_alias: {}", e))
      })?)),
      None => Ok(None),
    }
  }

  async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let active = user_alias::ActiveModel {
      id: Set(id.to_string()),
      alias: Set(alias.alias.clone()),
      repo: Set(alias.repo.to_string()),
      filename: Set(alias.filename.clone()),
      snapshot: Set(alias.snapshot.clone()),
      request_params: Set(alias.request_params.clone()),
      context_params: Set(alias.context_params.clone()),
      updated_at: Set(now),
      ..Default::default()
    };
    active.update(&self.db).await.map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_user_alias(&self, id: &str) -> Result<(), DbError> {
    user_alias::Entity::delete_by_id(id)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError> {
    let results = user_alias::Entity::find()
      .order_by_asc(user_alias::Column::Alias)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    results
      .into_iter()
      .map(|m| {
        UserAlias::try_from(m)
          .map_err(|e| DbError::Conversion(format!("Failed to convert user_alias: {}", e)))
      })
      .collect()
  }
}
