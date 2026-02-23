use crate::db::{DbError, DbSetting, SettingsRepository, SqliteDbService};

#[async_trait::async_trait]
impl SettingsRepository for SqliteDbService {
  async fn get_setting(&self, key: &str) -> Result<Option<DbSetting>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, i64, i64)>(
      "SELECT key, value, value_type, created_at, updated_at FROM settings WHERE key = ?",
    )
    .bind(key)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(key, value, value_type, created_at, updated_at)| DbSetting {
        key,
        value,
        value_type,
        created_at,
        updated_at,
      },
    ))
  }

  async fn upsert_setting(&self, setting: &DbSetting) -> Result<DbSetting, DbError> {
    let now = self.time_service.utc_now().timestamp();
    sqlx::query(
      "INSERT INTO settings (key, value, value_type, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?)
       ON CONFLICT(key) DO UPDATE SET
         value = excluded.value,
         value_type = excluded.value_type,
         updated_at = excluded.updated_at",
    )
    .bind(&setting.key)
    .bind(&setting.value)
    .bind(&setting.value_type)
    .bind(now)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(DbSetting {
      key: setting.key.clone(),
      value: setting.value.clone(),
      value_type: setting.value_type.clone(),
      created_at: now,
      updated_at: now,
    })
  }

  async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM settings WHERE key = ?")
      .bind(key)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn list_settings(&self) -> Result<Vec<DbSetting>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, i64, i64)>(
      "SELECT key, value, value_type, created_at, updated_at FROM settings ORDER BY key",
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(key, value, value_type, created_at, updated_at)| DbSetting {
            key,
            value,
            value_type,
            created_at,
            updated_at,
          },
        )
        .collect(),
    )
  }
}
