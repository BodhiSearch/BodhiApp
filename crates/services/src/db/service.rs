use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  AccessRepository, AccessRequestRepository, ApiKeyUpdate, ApiToken, AppAccessRequestRow,
  AppClientToolsetConfigRow, AppToolsetConfigRow, DbCore, DbError, DownloadRequest,
  DownloadStatus, ModelMetadataRow, ModelRepository, SqlxError, TimeService, TokenRepository,
  TokenStatus, ToolsetRepository, ToolsetRow, UserAccessRequest, UserAccessRequestStatus,
  UserAliasRepository,
};
use chrono::{DateTime, Utc};
use derive_new::new;
use objs::{AliasSource, ApiAlias, ApiFormat, UserAlias};
use sqlx::{query_as, SqlitePool};
use std::{str::FromStr, sync::Arc};

/// Super-trait that combines all repository sub-traits.
/// Any type implementing all sub-traits automatically implements DbService
/// via the blanket impl below.
pub trait DbService:
  ModelRepository
  + AccessRepository
  + AccessRequestRepository
  + TokenRepository
  + ToolsetRepository
  + UserAliasRepository
  + DbCore
  + Send
  + Sync
  + std::fmt::Debug
{
}

impl<T> DbService for T where
  T: ModelRepository
    + AccessRepository
    + AccessRequestRepository
    + TokenRepository
    + ToolsetRepository
    + UserAliasRepository
    + DbCore
    + Send
    + Sync
    + std::fmt::Debug
{
}

#[derive(Debug, Clone, new)]
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
  is_production: bool,
}

impl SqliteDbService {
  async fn seed_toolset_configs(&self) -> Result<(), DbError> {
    let scope_uuid = if self.is_production {
      "7a89e236-9d23-4856-aa77-b52823ff9972"
    } else {
      "4ff0e163-36fb-47d6-a5ef-26e396f067d6"
    };

    sqlx::query(
      "INSERT OR IGNORE INTO app_toolset_configs
       (scope, scope_uuid, enabled, updated_by, created_at, updated_at)
       VALUES (?, ?, 0, 'system', strftime('%s', 'now'), strftime('%s', 'now'))",
    )
    .bind("scope_toolset-builtin-exa-web-search")
    .bind(scope_uuid)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn get_by_col(
    &self,
    query: &str,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, DbError> {
    let result = query_as::<
      _,
      (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(query)
    .bind(user_id)
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        user_id,
        name,
        token_prefix,
        token_hash,
        scopes,
        status,
        created_at,
        updated_at,
      )) => {
        let Ok(status) = TokenStatus::from_str(&status) else {
          tracing::warn!("unknown token status: {status} for id: {id}");
          return Ok(None);
        };

        let result = ApiToken {
          id,
          user_id,
          name,
          token_prefix,
          token_hash,
          scopes,
          status,
          created_at,
          updated_at,
        };
        Ok(Some(result))
      }
      None => Ok(None),
    }
  }
}

#[async_trait::async_trait]
impl DbCore for SqliteDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(&self.pool).await?;
    self.seed_toolset_configs().await?;
    Ok(())
  }

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }

  fn encryption_key(&self) -> &[u8] {
    &self.encryption_key
  }
}

#[async_trait::async_trait]
impl ModelRepository for SqliteDbService {
  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query(
      "INSERT INTO download_requests (id, repo, filename, status, error, created_at, updated_at, total_bytes, downloaded_bytes, started_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&request.id)
    .bind(&request.repo)
    .bind(&request.filename)
    .bind(request.status.to_string())
    .bind(&request.error)
    .bind(request.created_at.timestamp())
    .bind(request.updated_at.timestamp())
    .bind(request.total_bytes.map(|b| b as i64))
    .bind(request.downloaded_bytes as i64)
    .bind(request.started_at.map(|t| t.timestamp()))
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = query_as::<_, (String, String, String, String, Option<String>, i64, i64, Option<i64>, i64, Option<i64>)>(
      "SELECT id, repo, filename, status, error, created_at, updated_at, total_bytes, downloaded_bytes, started_at FROM download_requests WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        repo,
        filename,
        status,
        error,
        created_at,
        updated_at,
        total_bytes,
        downloaded_bytes,
        started_at,
      )) => {
        let Ok(status) = DownloadStatus::from_str(&status) else {
          tracing::warn!("unknown download status: {status} for id: {id}");
          return Ok(None);
        };

        Ok(Some(DownloadRequest {
          id,
          repo,
          filename,
          status,
          error,
          created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
          updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
          total_bytes: total_bytes.map(|b| b as u64),
          downloaded_bytes: downloaded_bytes as u64,
          started_at: started_at.and_then(|t| chrono::DateTime::<Utc>::from_timestamp(t, 0)),
        }))
      }
      None => Ok(None),
    }
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    sqlx::query("UPDATE download_requests SET status = ?, error = ?, updated_at = ?, total_bytes = ?, downloaded_bytes = ?, started_at = ? WHERE id = ?")
      .bind(request.status.to_string())
      .bind(&request.error)
      .bind(request.updated_at.timestamp())
      .bind(request.total_bytes.map(|b| b as i64))
      .bind(request.downloaded_bytes as i64)
      .bind(request.started_at.map(|t| t.timestamp()))
      .bind(&request.id)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = ((page - 1) as i64) * (page_size as i64);

    // Get total count
    let total: usize = query_as::<_, (i64,)>("SELECT COUNT(*) FROM download_requests")
      .fetch_one(&self.pool)
      .await?
      .0 as usize;

    // Get paginated results using bind parameters
    let results = query_as::<_, (String, String, String, String, Option<String>, i64, i64, Option<i64>, i64, Option<i64>)>(
      "SELECT id, repo, filename, status, error, created_at, updated_at, total_bytes, downloaded_bytes, started_at
       FROM download_requests
       ORDER BY updated_at DESC
       LIMIT ? OFFSET ?",
    )
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(&self.pool)
    .await?;

    let items = results
      .into_iter()
      .filter_map(
        |(
          id,
          repo,
          filename,
          status,
          error,
          created_at,
          updated_at,
          total_bytes,
          downloaded_bytes,
          started_at,
        )| {
          let status = DownloadStatus::from_str(&status).ok()?;
          Some(DownloadRequest {
            id,
            repo,
            filename,
            status,
            error,
            created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
            updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
            total_bytes: total_bytes.map(|b| b as u64),
            downloaded_bytes: downloaded_bytes as u64,
            started_at: started_at.and_then(|t| chrono::DateTime::<Utc>::from_timestamp(t, 0)),
          })
        },
      )
      .collect::<Vec<DownloadRequest>>();

    Ok((items, total))
  }

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError> {
    let results = query_as::<_, (String, String, String, String, Option<String>, i64, i64, Option<i64>, i64, Option<i64>)>(
      "SELECT id, repo, filename, status, error, created_at, updated_at, total_bytes, downloaded_bytes, started_at
       FROM download_requests
       WHERE repo = ? AND filename = ?
       ORDER BY created_at DESC",
    )
    .bind(repo)
    .bind(filename)
    .fetch_all(&self.pool)
    .await?;

    let items = results
      .into_iter()
      .filter_map(
        |(
          id,
          repo,
          filename,
          status,
          error,
          created_at,
          updated_at,
          total_bytes,
          downloaded_bytes,
          started_at,
        )| {
          let status = DownloadStatus::from_str(&status).ok()?;
          Some(DownloadRequest {
            id,
            repo,
            filename,
            status,
            error,
            created_at: chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default(),
            updated_at: chrono::DateTime::<Utc>::from_timestamp(updated_at, 0).unwrap_or_default(),
            total_bytes: total_bytes.map(|b| b as u64),
            downloaded_bytes: downloaded_bytes as u64,
            started_at: started_at.and_then(|t| chrono::DateTime::<Utc>::from_timestamp(t, 0)),
          })
        },
      )
      .collect();

    Ok(items)
  }

  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    // Check prefix uniqueness if prefix is non-empty
    if let Some(ref prefix) = alias.prefix {
      if !prefix.is_empty() && self.check_prefix_exists(prefix, None).await? {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let models_json = serde_json::to_string(&alias.models)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models: {}", e)))?;

    let (encrypted_api_key, salt, nonce) = if let Some(ref key) = api_key {
      let (enc, s, n) = encrypt_api_key(&self.encryption_key, key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let models_cache_json = serde_json::to_string(&alias.models_cache)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models_cache: {}", e)))?;

    sqlx::query(
      r#"
      INSERT INTO api_model_aliases (id, api_format, base_url, models_json, prefix, forward_all_with_prefix, models_cache_json, cache_fetched_at, encrypted_api_key, salt, nonce, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#
    )
    .bind(&alias.id)
    .bind(alias.api_format.to_string())
    .bind(&alias.base_url)
    .bind(&models_json)
    .bind(&alias.prefix)
    .bind(alias.forward_all_with_prefix)
    .bind(&models_cache_json)
    .bind(alias.cache_fetched_at.timestamp())
    .bind(&encrypted_api_key)
    .bind(&salt)
    .bind(&nonce)
    .bind(alias.created_at.timestamp())
    .bind(alias.created_at.timestamp()) // updated_at = created_at initially
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError> {
    let result = query_as::<_, (String, String, String, String, Option<String>, bool, Option<String>, i64, i64)>(
      "SELECT id, api_format, base_url, models_json, prefix, forward_all_with_prefix, models_cache_json, cache_fetched_at, created_at FROM api_model_aliases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        api_format_str,
        base_url,
        models_json,
        prefix,
        forward_all_with_prefix,
        models_cache_json,
        cache_fetched_at,
        created_at,
      )) => {
        let api_format = api_format_str
          .parse::<ApiFormat>()
          .map_err(|e| DbError::EncryptionError(format!("Failed to parse api_format: {}", e)))?;

        let models: Vec<String> = serde_json::from_str(&models_json)
          .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

        let models_cache: Vec<String> = if let Some(cache_json) = models_cache_json {
          serde_json::from_str(&cache_json).map_err(|e| {
            DbError::EncryptionError(format!("Failed to deserialize models_cache: {}", e))
          })?
        } else {
          Vec::new()
        };

        let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();
        let cache_fetched_at =
          chrono::DateTime::<Utc>::from_timestamp(cache_fetched_at, 0).unwrap_or_default();

        Ok(Some(ApiAlias {
          id,
          api_format,
          base_url,
          models,
          prefix,
          forward_all_with_prefix,
          models_cache,
          cache_fetched_at,
          created_at,
          updated_at: created_at,
        }))
      }
      None => Ok(None),
    }
  }

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError> {
    // Check prefix uniqueness if prefix is non-empty
    if let Some(ref prefix) = model.prefix {
      if !prefix.is_empty()
        && self
          .check_prefix_exists(prefix, Some(id.to_string()))
          .await?
      {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let models_json = serde_json::to_string(&model.models)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models: {}", e)))?;

    let now = self.time_service.utc_now();

    match api_key {
      ApiKeyUpdate::Set(api_key_opt) => {
        // Update with new API key (or clear if None)
        match api_key_opt {
          Some(api_key) => {
            let (encrypted_api_key, salt, nonce) = encrypt_api_key(&self.encryption_key, &api_key)
              .map_err(|e| DbError::EncryptionError(e.to_string()))?;

            sqlx::query(
              r#"
              UPDATE api_model_aliases
              SET api_format = ?, base_url = ?, models_json = ?, prefix = ?, forward_all_with_prefix = ?, encrypted_api_key = ?, salt = ?, nonce = ?, updated_at = ?
              WHERE id = ?
              "#
            )
            .bind(model.api_format.to_string())
            .bind(&model.base_url)
            .bind(&models_json)
            .bind(&model.prefix)
            .bind(model.forward_all_with_prefix)
            .bind(&encrypted_api_key)
            .bind(&salt)
            .bind(&nonce)
            .bind(now.timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;
          }
          None => {
            // Clear the API key
            sqlx::query(
              r#"
              UPDATE api_model_aliases
              SET api_format = ?, base_url = ?, models_json = ?, prefix = ?, forward_all_with_prefix = ?, encrypted_api_key = NULL, salt = NULL, nonce = NULL, updated_at = ?
              WHERE id = ?
              "#
            )
            .bind(model.api_format.to_string())
            .bind(&model.base_url)
            .bind(&models_json)
            .bind(&model.prefix)
            .bind(model.forward_all_with_prefix)
            .bind(now.timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;
          }
        }
      }
      ApiKeyUpdate::Keep => {
        // Update without changing API key
        sqlx::query(
          r#"
          UPDATE api_model_aliases
          SET api_format = ?, base_url = ?, models_json = ?, prefix = ?, forward_all_with_prefix = ?, updated_at = ?
          WHERE id = ?
          "#,
        )
        .bind(model.api_format.to_string())
        .bind(&model.base_url)
        .bind(&models_json)
        .bind(&model.prefix)
        .bind(model.forward_all_with_prefix)
        .bind(now.timestamp())
        .bind(id)
        .execute(&self.pool)
        .await?;
      }
    }

    Ok(())
  }

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    let models_cache_json = serde_json::to_string(&models)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models_cache: {}", e)))?;

    sqlx::query(
      r#"
      UPDATE api_model_aliases
      SET models_cache_json = ?, cache_fetched_at = ?
      WHERE id = ?
      "#,
    )
    .bind(&models_cache_json)
    .bind(fetched_at.timestamp())
    .bind(id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM api_model_aliases WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError> {
    let results = query_as::<_, (String, String, String, String, Option<String>, bool, Option<String>, i64, i64)>(
      "SELECT id, api_format, base_url, models_json, prefix, forward_all_with_prefix, models_cache_json, cache_fetched_at, created_at FROM api_model_aliases ORDER BY created_at DESC"
    )
    .fetch_all(&self.pool)
    .await?;

    let mut aliases = Vec::new();
    for (
      id,
      api_format_str,
      base_url,
      models_json,
      prefix,
      forward_all_with_prefix,
      models_cache_json,
      cache_fetched_at,
      created_at,
    ) in results
    {
      let api_format = api_format_str
        .parse::<ApiFormat>()
        .map_err(|e| DbError::EncryptionError(format!("Failed to parse api_format: {}", e)))?;

      let models: Vec<String> = serde_json::from_str(&models_json)
        .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

      let models_cache: Vec<String> = if let Some(cache_json) = models_cache_json {
        serde_json::from_str(&cache_json).map_err(|e| {
          DbError::EncryptionError(format!("Failed to deserialize models_cache: {}", e))
        })?
      } else {
        Vec::new()
      };

      let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();
      let cache_fetched_at =
        chrono::DateTime::<Utc>::from_timestamp(cache_fetched_at, 0).unwrap_or_default();

      aliases.push(ApiAlias {
        id,
        api_format,
        base_url,
        models,
        prefix,
        forward_all_with_prefix,
        models_cache,
        cache_fetched_at,
        created_at,
        updated_at: created_at,
      });
    }

    Ok(aliases)
  }

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError> {
    let result = query_as::<_, (Option<String>, Option<String>, Option<String>)>(
      "SELECT encrypted_api_key, salt, nonce FROM api_model_aliases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((Some(encrypted_api_key), Some(salt), Some(nonce))) => {
        let api_key = decrypt_api_key(&self.encryption_key, &encrypted_api_key, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some(api_key))
      }
      Some((None, None, None)) => {
        // No API key stored - return None
        Ok(None)
      }
      Some(_) => {
        // Partial NULL - data corruption
        Err(DbError::EncryptionError(format!(
          "Data corruption: API key encryption fields are partially NULL for alias '{}'",
          id
        )))
      }
      None => {
        // Alias doesn't exist
        Ok(None)
      }
    }
  }

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    let count: i64 = match exclude_id {
      Some(id) => {
        sqlx::query_scalar("SELECT COUNT(*) FROM api_model_aliases WHERE prefix = ? AND id != ?")
          .bind(prefix)
          .bind(id)
          .fetch_one(&self.pool)
          .await?
      }
      None => {
        sqlx::query_scalar("SELECT COUNT(*) FROM api_model_aliases WHERE prefix = ?")
          .bind(prefix)
          .fetch_one(&self.pool)
          .await?
      }
    };

    Ok(count > 0)
  }

  async fn upsert_model_metadata(&self, metadata: &ModelMetadataRow) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    // For local models (api_model_id IS NULL), we need to delete existing rows first
    // because the UNIQUE constraint doesn't work with NULL values (NULL != NULL in SQL)
    if metadata.api_model_id.is_none() {
      // Delete any existing metadata for this local model
      sqlx::query(
        r#"
        DELETE FROM model_metadata
        WHERE source = ? AND repo = ? AND filename = ? AND snapshot = ? AND api_model_id IS NULL
        "#,
      )
      .bind(&metadata.source)
      .bind(&metadata.repo)
      .bind(&metadata.filename)
      .bind(&metadata.snapshot)
      .execute(&self.pool)
      .await?;
    }

    // Now insert the new/updated metadata
    sqlx::query(
      r#"
      INSERT INTO model_metadata (
        source, repo, filename, snapshot, api_model_id,
        capabilities_vision, capabilities_audio, capabilities_thinking,
        capabilities_function_calling, capabilities_structured_output,
        context_max_input_tokens, context_max_output_tokens,
        architecture, additional_metadata, chat_template,
        extracted_at, created_at, updated_at
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(source, repo, filename, snapshot, api_model_id) DO UPDATE SET
        capabilities_vision = excluded.capabilities_vision,
        capabilities_audio = excluded.capabilities_audio,
        capabilities_thinking = excluded.capabilities_thinking,
        capabilities_function_calling = excluded.capabilities_function_calling,
        capabilities_structured_output = excluded.capabilities_structured_output,
        context_max_input_tokens = excluded.context_max_input_tokens,
        context_max_output_tokens = excluded.context_max_output_tokens,
        architecture = excluded.architecture,
        additional_metadata = excluded.additional_metadata,
        chat_template = excluded.chat_template,
        extracted_at = excluded.extracted_at,
        updated_at = excluded.updated_at
      "#,
    )
    .bind(&metadata.source)
    .bind(&metadata.repo)
    .bind(&metadata.filename)
    .bind(&metadata.snapshot)
    .bind(&metadata.api_model_id)
    .bind(metadata.capabilities_vision)
    .bind(metadata.capabilities_audio)
    .bind(metadata.capabilities_thinking)
    .bind(metadata.capabilities_function_calling)
    .bind(metadata.capabilities_structured_output)
    .bind(metadata.context_max_input_tokens)
    .bind(metadata.context_max_output_tokens)
    .bind(&metadata.architecture)
    .bind(&metadata.additional_metadata)
    .bind(&metadata.chat_template)
    .bind(metadata.extracted_at)
    .bind(now)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataRow>, DbError> {
    // Metadata is always stored with source='model' since it represents the physical GGUF file
    let result = query_as::<_, ModelMetadataRow>(
      r#"
      SELECT
        id, source, repo, filename, snapshot, api_model_id,
        capabilities_vision, capabilities_audio, capabilities_thinking,
        capabilities_function_calling, capabilities_structured_output,
        context_max_input_tokens, context_max_output_tokens,
        architecture, additional_metadata, chat_template,
        extracted_at, created_at, updated_at
      FROM model_metadata
      WHERE source = ? AND repo = ? AND filename = ? AND snapshot = ?
      "#,
    )
    .bind(AliasSource::Model.to_string())
    .bind(repo)
    .bind(filename)
    .bind(snapshot)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result)
  }

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataRow>, DbError> {
    use std::collections::HashMap;

    if files.is_empty() {
      return Ok(HashMap::new());
    }

    // Debug logging
    tracing::debug!(
      "batch_get_metadata_by_files: querying {} files",
      files.len()
    );
    for (repo, filename, snapshot) in files {
      tracing::debug!(
        "  Query key: repo='{}', filename='{}', snapshot='{}'",
        repo,
        filename,
        snapshot
      );
    }

    // Build placeholders for IN clause: (?, ?, ?), (?, ?, ?), ...
    let placeholders: Vec<String> = files.iter().map(|_| "(?, ?, ?)".to_string()).collect();
    let placeholders_str = placeholders.join(", ");

    // Metadata is always stored with source='model' since it represents the physical GGUF file
    let query_str = format!(
      r#"
      SELECT
        id, source, repo, filename, snapshot, api_model_id,
        capabilities_vision, capabilities_audio, capabilities_thinking,
        capabilities_function_calling, capabilities_structured_output,
        context_max_input_tokens, context_max_output_tokens,
        architecture, additional_metadata, chat_template,
        extracted_at, created_at, updated_at
      FROM model_metadata
      WHERE source = ? AND (repo, filename, snapshot) IN ({})
      "#,
      placeholders_str
    );

    let mut query =
      sqlx::query_as::<_, ModelMetadataRow>(&query_str).bind(AliasSource::Model.to_string());

    for (repo, filename, snapshot) in files {
      query = query.bind(repo).bind(filename).bind(snapshot);
    }

    let results = query.fetch_all(&self.pool).await?;

    tracing::debug!(
      "batch_get_metadata_by_files: found {} results",
      results.len()
    );

    let mut map = HashMap::new();
    for row in results {
      if let (Some(repo), Some(filename), Some(snapshot)) =
        (row.repo.clone(), row.filename.clone(), row.snapshot.clone())
      {
        tracing::debug!(
          "  Result: source='{}', repo='{}', filename='{}', snapshot='{}'",
          row.source,
          repo,
          filename,
          snapshot
        );
        map.insert((repo, filename, snapshot), row);
      }
    }

    tracing::debug!(
      "batch_get_metadata_by_files: returning {} entries in map",
      map.len()
    );

    Ok(map)
  }

  async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError> {
    let results = query_as::<_, ModelMetadataRow>(
      r#"
      SELECT
        id, source, repo, filename, snapshot, api_model_id,
        capabilities_vision, capabilities_audio, capabilities_thinking,
        capabilities_function_calling, capabilities_structured_output,
        context_max_input_tokens, context_max_output_tokens,
        architecture, additional_metadata, chat_template,
        extracted_at, created_at, updated_at
      FROM model_metadata
      ORDER BY source, repo, filename
      "#,
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(results)
  }
}

#[async_trait::async_trait]
impl AccessRepository for SqliteDbService {
  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "INSERT INTO access_requests (username, user_id, created_at, updated_at, status)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, username, user_id, reviewer, status, created_at, updated_at",
    )
    .bind(&username)
    .bind(&user_id)
    .bind(now)
    .bind(now)
    .bind(UserAccessRequestStatus::Pending.to_string())
    .fetch_one(&self.pool)
    .await?;

    Ok(UserAccessRequest {
      id: result.0,
      username: result.1,
      user_id: result.2,
      reviewer: result.3,
      status: UserAccessRequestStatus::from_str(&result.4)?,
      created_at: result.5,
      updated_at: result.6,
    })
  }

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError> {
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE user_id = ? AND status = ?",
    )
    .bind(&user_id)
    .bind(UserAccessRequestStatus::Pending.to_string())
    .fetch_optional(&self.pool)
    .await?;

    let result = result
      .map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let Ok(status) = UserAccessRequestStatus::from_str(&status) else {
            tracing::warn!("unknown request status: {} for id: {}", status, id);
            return None;
          };
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .unwrap_or(None);
    Ok(result)
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = (page - 1) * per_page;
    // Get total count of pending requests
    let total_count: (i64,) = query_as("SELECT COUNT(*) FROM access_requests WHERE status = ?")
      .bind(UserAccessRequestStatus::Pending.to_string())
      .fetch_one(&self.pool)
      .await?;
    let results = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE status = ?
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(UserAccessRequestStatus::Pending.to_string())
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let Ok(status) = UserAccessRequestStatus::from_str(&status) else {
            tracing::warn!("unknown request status: {} for id: {}", status, id);
            return None;
          };
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .collect::<Vec<UserAccessRequest>>();
    Ok((results, total_count.0 as usize))
  }

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = (page - 1) * per_page;
    // Get total count of all requests
    let total_count: (i64,) = query_as("SELECT COUNT(*) FROM access_requests")
      .fetch_one(&self.pool)
      .await?;
    let results = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let status = UserAccessRequestStatus::from_str(&status).ok()?;
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .collect::<Vec<UserAccessRequest>>();
    Ok((results, total_count.0 as usize))
  }

  async fn update_request_status(
    &self,
    id: i64,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    sqlx::query(
      "UPDATE access_requests
         SET status = ?, updated_at = ?, reviewer = ?
         WHERE id = ?",
    )
    .bind(status.to_string())
    .bind(now)
    .bind(&reviewer)
    .bind(id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn get_request_by_id(&self, id: i64) -> Result<Option<UserAccessRequest>, DbError> {
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    if let Some((id, username, user_id, reviewer, status, created_at, updated_at)) = result {
      let status = UserAccessRequestStatus::from_str(&status).map_err(DbError::StrumParse)?;
      Ok(Some(UserAccessRequest {
        id,
        username,
        user_id,
        reviewer,
        status,
        created_at,
        updated_at,
      }))
    } else {
      Ok(None)
    }
  }
}

#[async_trait::async_trait]
impl TokenRepository for SqliteDbService {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    token.created_at = now;
    token.updated_at = now;

    sqlx::query(
      r#"
      INSERT INTO api_tokens (id, user_id, name, token_prefix, token_hash, scopes, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .bind(&token.name)
    .bind(&token.token_prefix)
    .bind(&token.token_hash)
    .bind(&token.scopes)
    .bind(token.status.to_string())
    .bind(token.created_at.timestamp())
    .bind(token.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError> {
    let offset = (page - 1) * per_page;

    let results = query_as::<
      _,
      (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      r#"
      SELECT
        id,
        user_id,
        name,
        token_prefix,
        token_hash,
        scopes,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE user_id = ?
      ORDER BY created_at DESC
      LIMIT ? OFFSET ?
      "#,
    )
    .bind(user_id)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let tokens: Vec<_> = results
      .into_iter()
      .filter_map(
        |(id, user_id, name, token_prefix, token_hash, scopes, status, created_at, updated_at)| {
          let Ok(status) = TokenStatus::from_str(&status) else {
            tracing::warn!("unknown token status: {} for id: {}", status, id);
            return None;
          };

          Some(ApiToken {
            id,
            user_id,
            name,
            token_prefix,
            token_hash,
            scopes,
            status,
            created_at,
            updated_at,
          })
        },
      )
      .collect();

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_tokens WHERE user_id = ?")
      .bind(user_id)
      .fetch_one(&self.pool)
      .await? as usize;
    Ok((tokens, total))
  }

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, DbError> {
    let query = r#"
      SELECT
        id,
        user_id,
        name,
        token_prefix,
        token_hash,
        scopes,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE user_id = ? AND id = ?
      "#;
    self.get_by_col(query, user_id, id).await
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
    let result = query_as::<
      _,
      (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      r#"
      SELECT
        id,
        user_id,
        name,
        token_prefix,
        token_hash,
        scopes,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE token_prefix = ?
      "#,
    )
    .bind(prefix)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((
        id,
        user_id,
        name,
        token_prefix,
        token_hash,
        scopes,
        status,
        created_at,
        updated_at,
      )) => {
        let Ok(status) = TokenStatus::from_str(&status) else {
          tracing::warn!("unknown token status: {status} for id: {id}");
          return Ok(None);
        };

        Ok(Some(ApiToken {
          id,
          user_id,
          name,
          token_prefix,
          token_hash,
          scopes,
          status,
          created_at,
          updated_at,
        }))
      }
      None => Ok(None),
    }
  }

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError> {
    token.updated_at = self.time_service.utc_now();
    let result = sqlx::query(
      r#"
      UPDATE api_tokens
      SET name = ?,
          status = ?,
          updated_at = CURRENT_TIMESTAMP
      WHERE id = ? AND user_id = ?
      "#,
    )
    .bind(&token.name)
    .bind(token.status.to_string())
    .bind(&token.id)
    .bind(user_id)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      return Err(DbError::SqlxError(SqlxError::from(
        sqlx::Error::RowNotFound,
      )));
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl ToolsetRepository for SqliteDbService {
  async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, scope_uuid, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        user_id,
        scope_uuid,
        name,
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
          scope_uuid,
          name,
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

  async fn get_toolset_by_name(
    &self,
    user_id: &str,
    name: &str,
  ) -> Result<Option<ToolsetRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, scope_uuid, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ? AND name = ? COLLATE NOCASE",
    )
    .bind(user_id)
    .bind(name)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        user_id,
        scope_uuid,
        name,
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
          scope_uuid,
          name,
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
      INSERT INTO toolsets (id, user_id, scope_uuid, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.user_id)
    .bind(&row.scope_uuid)
    .bind(&row.name)
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
          SET name = ?, description = ?, enabled = ?, updated_at = ?
          WHERE id = ?
          "#,
        )
        .bind(&row.name)
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
          SET name = ?, description = ?, enabled = ?, encrypted_api_key = ?, salt = ?, nonce = ?, updated_at = ?
          WHERE id = ?
          "#,
        )
        .bind(&row.name)
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
      "SELECT id, user_id, scope_uuid, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ?",
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
            scope_uuid,
            name,
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
              scope_uuid,
              name,
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

  async fn list_toolsets_by_scope_uuid(
    &self,
    user_id: &str,
    scope_uuid: &str,
  ) -> Result<Vec<ToolsetRow>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, scope_uuid, name, description, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM toolsets WHERE user_id = ? AND scope_uuid = ?",
    )
    .bind(user_id)
    .bind(scope_uuid)
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(
            id,
            user_id,
            scope_uuid,
            name,
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
              scope_uuid,
              name,
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

  async fn get_app_toolset_config_by_scope_uuid(
    &self,
    scope_uuid: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (i64, String, String, i64, String, i64, i64)>(
      "SELECT id, scope, scope_uuid, enabled, updated_by, created_at, updated_at FROM app_toolset_configs WHERE scope_uuid = ?",
    )
    .bind(scope_uuid)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(id, scope, scope_uuid, enabled, updated_by, created_at, updated_at)| AppToolsetConfigRow {
        id,
        scope,
        scope_uuid,
        enabled: enabled != 0,
        updated_by,
        created_at,
        updated_at,
      },
    ))
  }

  async fn get_app_toolset_config_by_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (i64, String, String, i64, String, i64, i64)>(
      "SELECT id, scope, scope_uuid, enabled, updated_by, created_at, updated_at FROM app_toolset_configs WHERE scope = ?",
    )
    .bind(scope)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(id, scope, scope_uuid, enabled, updated_by, created_at, updated_at)| AppToolsetConfigRow {
        id,
        scope,
        scope_uuid,
        enabled: enabled != 0,
        updated_by,
        created_at,
        updated_at,
      },
    ))
  }

  async fn upsert_app_toolset_config(
    &self,
    config: &AppToolsetConfigRow,
  ) -> Result<AppToolsetConfigRow, DbError> {
    let enabled = if config.enabled { 1 } else { 0 };

    // Check if config exists
    let existing = self.get_app_toolset_config_by_scope(&config.scope).await?;

    let id = if let Some(existing) = existing {
      // Update existing config
      sqlx::query(
        r#"
        UPDATE app_toolset_configs
        SET scope_uuid = ?, enabled = ?, updated_by = ?, updated_at = ?
        WHERE scope = ?
        "#,
      )
      .bind(&config.scope_uuid)
      .bind(enabled)
      .bind(&config.updated_by)
      .bind(config.updated_at)
      .bind(&config.scope)
      .execute(&self.pool)
      .await?;

      existing.id
    } else {
      // Insert new config
      let result = sqlx::query(
        r#"
        INSERT INTO app_toolset_configs (scope, scope_uuid, enabled, updated_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
      )
      .bind(&config.scope)
      .bind(&config.scope_uuid)
      .bind(enabled)
      .bind(&config.updated_by)
      .bind(config.created_at)
      .bind(config.updated_at)
      .execute(&self.pool)
      .await?;

      result.last_insert_rowid()
    };

    // Return the updated/inserted config
    Ok(AppToolsetConfigRow {
      id,
      scope: config.scope.clone(),
      scope_uuid: config.scope_uuid.clone(),
      enabled: config.enabled,
      updated_by: config.updated_by.clone(),
      created_at: config.created_at,
      updated_at: config.updated_at,
    })
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    let results = sqlx::query_as::<_, (i64, String, String, i64, String, i64, i64)>(
      "SELECT id, scope, scope_uuid, enabled, updated_by, created_at, updated_at FROM app_toolset_configs ORDER BY scope",
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(id, scope, scope_uuid, enabled, updated_by, created_at, updated_at)| {
            AppToolsetConfigRow {
              id,
              scope,
              scope_uuid,
              enabled: enabled != 0,
              updated_by,
              created_at,
              updated_at,
            }
          },
        )
        .collect(),
    )
  }

  async fn list_app_toolset_configs_by_scopes(
    &self,
    scopes: &[String],
  ) -> Result<Vec<AppToolsetConfigRow>, DbError> {
    if scopes.is_empty() {
      return Ok(vec![]);
    }

    let placeholders = scopes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
      "SELECT id, scope, scope_uuid, enabled, updated_by, created_at, updated_at
       FROM app_toolset_configs
       WHERE scope IN ({})
       ORDER BY scope",
      placeholders
    );

    let mut query_builder =
      sqlx::query_as::<_, (i64, String, String, i64, String, i64, i64)>(&query);
    for scope in scopes {
      query_builder = query_builder.bind(scope);
    }

    let results = query_builder.fetch_all(&self.pool).await?;

    Ok(
      results
        .into_iter()
        .map(
          |(id, scope, scope_uuid, enabled, updated_by, created_at, updated_at)| {
            AppToolsetConfigRow {
              id,
              scope,
              scope_uuid,
              enabled: enabled != 0,
              updated_by,
              created_at,
              updated_at,
            }
          },
        )
        .collect(),
    )
  }

  async fn get_app_client_toolset_config(
    &self,
    app_client_id: &str,
  ) -> Result<Option<AppClientToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (i64, String, Option<String>, String, String, i64, i64)>(
      "SELECT id, app_client_id, config_version, toolsets_json, resource_scope, created_at, updated_at
       FROM app_client_toolset_configs WHERE app_client_id = ?",
    )
    .bind(app_client_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        app_client_id,
        config_version,
        toolsets_json,
        resource_scope,
        created_at,
        updated_at,
      )| {
        AppClientToolsetConfigRow {
          id,
          app_client_id,
          config_version,
          toolsets_json,
          resource_scope,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn upsert_app_client_toolset_config(
    &self,
    config: &AppClientToolsetConfigRow,
  ) -> Result<AppClientToolsetConfigRow, DbError> {
    // SQLite upsert - insert or replace based on unique app_client_id
    let result = sqlx::query_as::<_, (i64, String, Option<String>, String, String, i64, i64)>(
      "INSERT INTO app_client_toolset_configs (app_client_id, config_version, toolsets_json, resource_scope, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(app_client_id) DO UPDATE SET
         config_version = excluded.config_version,
         toolsets_json = excluded.toolsets_json,
         resource_scope = excluded.resource_scope,
         updated_at = excluded.updated_at
       RETURNING id, app_client_id, config_version, toolsets_json, resource_scope, created_at, updated_at",
    )
    .bind(&config.app_client_id)
    .bind(&config.config_version)
    .bind(&config.toolsets_json)
    .bind(&config.resource_scope)
    .bind(config.created_at)
    .bind(config.updated_at)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppClientToolsetConfigRow {
      id: result.0,
      app_client_id: result.1,
      config_version: result.2,
      toolsets_json: result.3,
      resource_scope: result.4,
      created_at: result.5,
      updated_at: result.6,
    })
  }
}

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
  let request_params: objs::OAIRequestParams = serde_json::from_str(&request_params_json)
    .map_err(|e| {
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

// ============================================================================
// AccessRequestRepository Implementation
// ============================================================================

#[async_trait::async_trait]
impl AccessRequestRepository for SqliteDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "INSERT INTO app_access_requests
        (id, app_client_id, flow_type, redirect_uri, status, tools_requested,
         tools_approved, user_id, resource_scope, access_request_scope, error_message,
         expires_at, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       RETURNING id, app_client_id, flow_type, redirect_uri, status, tools_requested,
                 tools_approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(&row.id)
    .bind(&row.app_client_id)
    .bind(&row.flow_type)
    .bind(&row.redirect_uri)
    .bind(&row.status)
    .bind(&row.tools_requested)
    .bind(&row.tools_approved)
    .bind(&row.user_id)
    .bind(&row.resource_scope)
    .bind(&row.access_request_scope)
    .bind(&row.error_message)
    .bind(row.expires_at)
    .bind(row.created_at)
    .bind(row.updated_at)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      flow_type: result.2,
      redirect_uri: result.3,
      status: result.4,
      tools_requested: result.5,
      tools_approved: result.6,
      user_id: result.7,
      resource_scope: result.8,
      access_request_scope: result.9,
      error_message: result.10,
      expires_at: result.11,
      created_at: result.12,
      updated_at: result.13,
    })
  }

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "SELECT id, app_client_id, flow_type, redirect_uri, status, tools_requested,
              tools_approved, user_id, resource_scope, access_request_scope, error_message,
              expires_at, created_at, updated_at
       FROM app_access_requests WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(|r| AppAccessRequestRow {
      id: r.0,
      app_client_id: r.1,
      flow_type: r.2,
      redirect_uri: r.3,
      status: r.4,
      tools_requested: r.5,
      tools_approved: r.6,
      user_id: r.7,
      resource_scope: r.8,
      access_request_scope: r.9,
      error_message: r.10,
      expires_at: r.11,
      created_at: r.12,
      updated_at: r.13,
    }))
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    tools_approved: &str,
    resource_scope: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'approved', user_id = ?, tools_approved = ?,
           resource_scope = ?, access_request_scope = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, flow_type, redirect_uri, status, tools_requested,
                 tools_approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(tools_approved)
    .bind(resource_scope)
    .bind(access_request_scope)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      flow_type: result.2,
      redirect_uri: result.3,
      status: result.4,
      tools_requested: result.5,
      tools_approved: result.6,
      user_id: result.7,
      resource_scope: result.8,
      access_request_scope: result.9,
      error_message: result.10,
      expires_at: result.11,
      created_at: result.12,
      updated_at: result.13,
    })
  }

  async fn update_denial(
    &self,
    id: &str,
    user_id: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'denied', user_id = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, flow_type, redirect_uri, status, tools_requested,
                 tools_approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      flow_type: result.2,
      redirect_uri: result.3,
      status: result.4,
      tools_requested: result.5,
      tools_approved: result.6,
      user_id: result.7,
      resource_scope: result.8,
      access_request_scope: result.9,
      error_message: result.10,
      expires_at: result.11,
      created_at: result.12,
      updated_at: result.13,
    })
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'failed', error_message = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, flow_type, redirect_uri, status, tools_requested,
                 tools_approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(error_message)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      flow_type: result.2,
      redirect_uri: result.3,
      status: result.4,
      tools_requested: result.5,
      tools_approved: result.6,
      user_id: result.7,
      resource_scope: result.8,
      access_request_scope: result.9,
      error_message: result.10,
      expires_at: result.11,
      created_at: result.12,
      updated_at: result.13,
    })
  }
}
