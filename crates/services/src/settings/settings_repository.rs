use super::setting_entity::{self, DbSetting};
use crate::db::{DbError, DefaultDbService};
use sea_orm::{sea_query::OnConflict, EntityTrait, QueryOrder, Set};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SettingsRepository: Send + Sync {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError>;
  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError>;
  async fn delete_setting(&self, key: &str) -> Result<(), DbError>;
  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError>;
}

#[async_trait::async_trait]
impl SettingsRepository for DefaultDbService {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    let result = setting_entity::Entity::find_by_id(key)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(DbSetting::from))
  }

  async fn upsert_setting(&self, setting_input: &DbSetting) -> Result<DbSetting, DbError> {
    let now = self.time_service.utc_now();
    let active = setting_entity::ActiveModel {
      key: Set(setting_input.key.clone()),
      value: Set(setting_input.value.clone()),
      value_type: Set(setting_input.value_type.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    setting_entity::Entity::insert(active)
      .on_conflict(
        OnConflict::column(setting_entity::Column::Key)
          .update_columns([
            setting_entity::Column::Value,
            setting_entity::Column::ValueType,
            setting_entity::Column::UpdatedAt,
          ])
          .to_owned(),
      )
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    // Re-fetch to get the actual stored values (created_at may differ on update)
    let stored = self.get_setting(&setting_input.key).await?;
    Ok(stored.expect("setting should exist after upsert"))
  }

  async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
    setting_entity::Entity::delete_by_id(key)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    let results = setting_entity::Entity::find()
      .order_by_asc(setting_entity::Column::Key)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(DbSetting::from).collect())
  }
}
