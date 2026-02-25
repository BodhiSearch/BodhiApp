use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  AppInstanceRepository, AppInstanceRow, DbError, SqliteDbService,
};

#[async_trait::async_trait]
impl AppInstanceRepository for SqliteDbService {
  async fn get_app_instance(&self) -> Result<Option<AppInstanceRow>, DbError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM apps")
      .fetch_one(&self.pool)
      .await?;

    if count > 1 {
      return Err(DbError::MultipleAppInstance);
    }

    if count == 0 {
      return Ok(None);
    }

    let result = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64)>(
      "SELECT client_id, encrypted_client_secret, salt_client_secret, nonce_client_secret,
              scope, app_status, created_at, updated_at
       FROM apps",
    )
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((client_id, encrypted, salt, nonce, scope, app_status, created_at, updated_at)) => {
        let client_secret = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some(AppInstanceRow {
          client_id,
          client_secret,
          salt_client_secret: salt,
          nonce_client_secret: nonce,
          scope,
          app_status,
          created_at,
          updated_at,
        }))
      }
      None => Ok(None),
    }
  }

  async fn upsert_app_instance(
    &self,
    client_id: &str,
    client_secret: &str,
    scope: &str,
    status: &str,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now().timestamp();
    let (encrypted_secret, salt_secret, nonce_secret) =
      encrypt_api_key(&self.encryption_key, client_secret)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    sqlx::query(
      "INSERT INTO apps (client_id, encrypted_client_secret, salt_client_secret, nonce_client_secret,
                         scope, app_status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(client_id) DO UPDATE SET
         encrypted_client_secret = excluded.encrypted_client_secret,
         salt_client_secret = excluded.salt_client_secret,
         nonce_client_secret = excluded.nonce_client_secret,
         scope = excluded.scope,
         app_status = excluded.app_status,
         updated_at = excluded.updated_at",
    )
    .bind(client_id)
    .bind(&encrypted_secret)
    .bind(&salt_secret)
    .bind(&nonce_secret)
    .bind(scope)
    .bind(status)
    .bind(now)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn update_app_instance_status(&self, client_id: &str, status: &str) -> Result<(), DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = sqlx::query("UPDATE apps SET app_status = ?, updated_at = ? WHERE client_id = ?")
      .bind(status)
      .bind(now)
      .bind(client_id)
      .execute(&self.pool)
      .await?;
    if result.rows_affected() == 0 {
      return Err(DbError::ItemNotFound {
        id: client_id.to_string(),
        item_type: "app_instance".to_string(),
      });
    }
    Ok(())
  }

  async fn delete_app_instance(&self, client_id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM apps WHERE client_id = ?")
      .bind(client_id)
      .execute(&self.pool)
      .await?;
    Ok(())
  }
}
