use crate::db::{DbError, SqliteDbService, UserAliasRepository};
use chrono::Utc;
use objs::UserAlias;
use sqlx::query_as;

fn parse_user_alias_row(
  id: String,
  alias: String,
  repo: String,
  filename: String,
  snapshot: String,
  request_params_json: String,
  context_params_json: String,
  created_at: i64,
  updated_at: i64,
) -> Result<UserAlias, DbError> {
  let repo = repo
    .parse::<objs::Repo>()
    .map_err(|e| DbError::EncryptionError(format!("Failed to parse repo: {}", e)))?;
  let request_params: objs::OAIRequestParams =
    serde_json::from_str(&request_params_json).map_err(|e| {
      DbError::EncryptionError(format!("Failed to deserialize request_params: {}", e))
    })?;
  let context_params: Vec<String> = serde_json::from_str(&context_params_json).map_err(|e| {
    DbError::EncryptionError(format!("Failed to deserialize context_params: {}", e))
  })?;
  let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();
  let updated_at = chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default();

  Ok(UserAlias {
    id,
    alias,
    repo,
    filename,
    snapshot,
    request_params,
    context_params,
    created_at,
    updated_at,
  })
}

#[async_trait::async_trait]
impl UserAliasRepository for SqliteDbService {
  async fn create_user_alias(&self, alias: &UserAlias) -> Result<(), DbError> {
    let request_params_json = serde_json::to_string(&alias.request_params).map_err(|e| {
      DbError::EncryptionError(format!("Failed to serialize request_params: {}", e))
    })?;
    let context_params_json = serde_json::to_string(&alias.context_params).map_err(|e| {
      DbError::EncryptionError(format!("Failed to serialize context_params: {}", e))
    })?;

    sqlx::query(
      r#"INSERT INTO user_aliases (id, alias, repo, filename, snapshot, request_params_json, context_params_json, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&alias.id)
    .bind(&alias.alias)
    .bind(alias.repo.to_string())
    .bind(&alias.filename)
    .bind(&alias.snapshot)
    .bind(&request_params_json)
    .bind(&context_params_json)
    .bind(alias.created_at.timestamp())
    .bind(alias.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn get_user_alias_by_id(&self, id: &str) -> Result<Option<UserAlias>, DbError> {
    let result = query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
      "SELECT id, alias, repo, filename, snapshot, request_params_json, context_params_json, created_at, updated_at FROM user_aliases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params_json,
        context_params_json,
        created_at,
        updated_at,
      )) => Ok(Some(parse_user_alias_row(
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params_json,
        context_params_json,
        created_at,
        updated_at,
      )?)),
      None => Ok(None),
    }
  }

  async fn get_user_alias_by_name(&self, alias_name: &str) -> Result<Option<UserAlias>, DbError> {
    let result = query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
      "SELECT id, alias, repo, filename, snapshot, request_params_json, context_params_json, created_at, updated_at FROM user_aliases WHERE alias = ?",
    )
    .bind(alias_name)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params_json,
        context_params_json,
        created_at,
        updated_at,
      )) => Ok(Some(parse_user_alias_row(
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params_json,
        context_params_json,
        created_at,
        updated_at,
      )?)),
      None => Ok(None),
    }
  }

  async fn update_user_alias(&self, id: &str, alias: &UserAlias) -> Result<(), DbError> {
    let request_params_json = serde_json::to_string(&alias.request_params).map_err(|e| {
      DbError::EncryptionError(format!("Failed to serialize request_params: {}", e))
    })?;
    let context_params_json = serde_json::to_string(&alias.context_params).map_err(|e| {
      DbError::EncryptionError(format!("Failed to serialize context_params: {}", e))
    })?;
    let now = self.time_service.utc_now();

    sqlx::query(
      r#"UPDATE user_aliases SET alias = ?, repo = ?, filename = ?, snapshot = ?, request_params_json = ?, context_params_json = ?, updated_at = ? WHERE id = ?"#,
    )
    .bind(&alias.alias)
    .bind(alias.repo.to_string())
    .bind(&alias.filename)
    .bind(&alias.snapshot)
    .bind(&request_params_json)
    .bind(&context_params_json)
    .bind(now.timestamp())
    .bind(id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_user_alias(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM user_aliases WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn list_user_aliases(&self) -> Result<Vec<UserAlias>, DbError> {
    let results = query_as::<_, (String, String, String, String, String, String, String, i64, i64)>(
      "SELECT id, alias, repo, filename, snapshot, request_params_json, context_params_json, created_at, updated_at FROM user_aliases ORDER BY alias",
    )
    .fetch_all(&self.pool)
    .await?;

    let mut aliases = Vec::new();
    for (
      id,
      alias,
      repo,
      filename,
      snapshot,
      request_params_json,
      context_params_json,
      created_at,
      updated_at,
    ) in results
    {
      aliases.push(parse_user_alias_row(
        id,
        alias,
        repo,
        filename,
        snapshot,
        request_params_json,
        context_params_json,
        created_at,
        updated_at,
      )?);
    }

    Ok(aliases)
  }
}
