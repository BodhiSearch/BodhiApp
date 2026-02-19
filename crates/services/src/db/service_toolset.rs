use crate::db::{
  encryption::decrypt_api_key, ApiKeyUpdate, AppToolsetConfigRow, DbError, SqliteDbService,
  ToolsetRepository, ToolsetRow,
};

#[async_trait::async_trait]
impl ToolsetRepository for SqliteDbService {
  async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_type, slug, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        user_id,
        toolset_type,
        slug,
        description,
        enabled,
        encrypted_api_key,
        salt,
        nonce,
        created_at,
        updated_at,
      )| {
        ToolsetRow {
          id,
          user_id,
          toolset_type,
          slug,
          description,
          enabled: enabled != 0,
          encrypted_api_key,
          salt,
          nonce,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn get_toolset_by_slug(
    &self,
    user_id: &str,
    slug: &str,
  ) -> Result<Option<ToolsetRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_type, slug, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ? AND slug = ? COLLATE NOCASE",
    )
    .bind(user_id)
    .bind(slug)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        user_id,
        toolset_type,
        slug,
        description,
        enabled,
        encrypted_api_key,
        salt,
        nonce,
        created_at,
        updated_at,
      )| {
        ToolsetRow {
          id,
          user_id,
          toolset_type,
          slug,
          description,
          enabled: enabled != 0,
          encrypted_api_key,
          salt,
          nonce,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError> {
    let enabled = if row.enabled { 1 } else { 0 };

    sqlx::query(
      r#"
      INSERT INTO toolsets (id, user_id, toolset_type, slug, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.user_id)
    .bind(&row.toolset_type)
    .bind(&row.slug)
    .bind(&row.description)
    .bind(enabled)
    .bind(&row.encrypted_api_key)
    .bind(&row.salt)
    .bind(&row.nonce)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn update_toolset(
    &self,
    row: &ToolsetRow,
    api_key_update: ApiKeyUpdate,
  ) -> Result<ToolsetRow, DbError> {
    let enabled = if row.enabled { 1 } else { 0 };

    match api_key_update {
      ApiKeyUpdate::Keep => {
        sqlx::query(
          r#"
          UPDATE toolsets
          SET slug = ?, description = ?, enabled = ?, updated_at = ?
          WHERE id = ?
          "#,
        )
        .bind(&row.slug)
        .bind(&row.description)
        .bind(enabled)
        .bind(row.updated_at)
        .bind(&row.id)
        .execute(&self.pool)
        .await?;
      }
      ApiKeyUpdate::Set(api_key) => {
        sqlx::query(
          r#"
          UPDATE toolsets
          SET slug = ?, description = ?, enabled = ?, encrypted_api_key = ?, salt = ?, nonce = ?, updated_at = ?
          WHERE id = ?
          "#,
        )
        .bind(&row.slug)
        .bind(&row.description)
        .bind(enabled)
        .bind(&api_key)
        .bind(&row.salt)
        .bind(&row.nonce)
        .bind(row.updated_at)
        .bind(&row.id)
        .execute(&self.pool)
        .await?;
      }
    }

    Ok(row.clone())
  }

  async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_type, slug, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(
            id,
            user_id,
            toolset_type,
            slug,
            description,
            enabled,
            encrypted_api_key,
            salt,
            nonce,
            created_at,
            updated_at,
          )| {
            ToolsetRow {
              id,
              user_id,
              toolset_type,
              slug,
              description,
              enabled: enabled != 0,
              encrypted_api_key,
              salt,
              nonce,
              created_at,
              updated_at,
            }
          },
        )
        .collect(),
    )
  }

  async fn list_toolsets_by_toolset_type(
    &self,
    user_id: &str,
    toolset_type: &str,
  ) -> Result<Vec<ToolsetRow>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_type, slug, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ? AND toolset_type = ?",
    )
    .bind(user_id)
    .bind(toolset_type)
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(
            id,
            user_id,
            toolset_type,
            slug,
            description,
            enabled,
            encrypted_api_key,
            salt,
            nonce,
            created_at,
            updated_at,
          )| {
            ToolsetRow {
              id,
              user_id,
              toolset_type,
              slug,
              description,
              enabled: enabled != 0,
              encrypted_api_key,
              salt,
              nonce,
              created_at,
              updated_at,
            }
          },
        )
        .collect(),
    )
  }

  async fn delete_toolset(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM toolsets WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;

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
    let now = self.time_service.utc_now().timestamp();
    let enabled_int = if enabled { 1 } else { 0 };

    let result = sqlx::query_as::<_, (String, i64, String, i64, i64)>(
      r#"
      INSERT INTO app_toolset_configs (toolset_type, enabled, updated_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?)
      ON CONFLICT (toolset_type) DO UPDATE SET
        enabled = excluded.enabled,
        updated_by = excluded.updated_by,
        updated_at = excluded.updated_at
      RETURNING toolset_type, enabled, updated_by, created_at, updated_at
      "#,
    )
    .bind(toolset_type)
    .bind(enabled_int)
    .bind(updated_by)
    .bind(now)
    .bind(now)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppToolsetConfigRow {
      toolset_type: result.0,
      enabled: result.1 != 0,
      updated_by: result.2,
      created_at: result.3,
      updated_at: result.4,
    })
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    let results = sqlx::query_as::<_, (String, i64, String, i64, i64)>(
      "SELECT toolset_type, enabled, updated_by, created_at, updated_at
       FROM app_toolset_configs",
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(toolset_type, enabled, updated_by, created_at, updated_at)| AppToolsetConfigRow {
            toolset_type,
            enabled: enabled != 0,
            updated_by,
            created_at,
            updated_at,
          },
        )
        .collect(),
    )
  }

  async fn get_app_toolset_config(
    &self,
    toolset_type: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (String, i64, String, i64, i64)>(
      "SELECT toolset_type, enabled, updated_by, created_at, updated_at
       FROM app_toolset_configs
       WHERE toolset_type = ?",
    )
    .bind(toolset_type)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(toolset_type, enabled, updated_by, created_at, updated_at)| AppToolsetConfigRow {
        toolset_type,
        enabled: enabled != 0,
        updated_by,
        created_at,
        updated_at,
      },
    ))
  }
}
