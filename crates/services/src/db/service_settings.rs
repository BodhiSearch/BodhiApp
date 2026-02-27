use crate::db::{entities::setting, DbError, DbSetting, DefaultDbService, SettingsRepository};
use sea_orm::{sea_query::OnConflict, EntityTrait, QueryOrder, Set};

#[async_trait::async_trait]
impl SettingsRepository for DefaultDbService {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    let result = setting::Entity::find_by_id(key)
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result.map(DbSetting::from))
  }

  async fn upsert_setting(&self, setting_input: &DbSetting) -> Result<DbSetting, DbError> {
    let now = self.time_service.utc_now();
    let active = setting::ActiveModel {
      key: Set(setting_input.key.clone()),
      value: Set(setting_input.value.clone()),
      value_type: Set(setting_input.value_type.clone()),
      created_at: Set(now),
      updated_at: Set(now),
    };
    setting::Entity::insert(active)
      .on_conflict(
        OnConflict::column(setting::Column::Key)
          .update_columns([
            setting::Column::Value,
            setting::Column::ValueType,
            setting::Column::UpdatedAt,
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
    setting::Entity::delete_by_id(key)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    let results = setting::Entity::find()
      .order_by_asc(setting::Column::Key)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(results.into_iter().map(DbSetting::from).collect())
  }
}
