use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  ApiKeyUpdate, DbError, DownloadRequest, DownloadStatus, ModelMetadataRow, ModelRepository,
  SqliteDbService,
};
use chrono::{DateTime, Utc};
use objs::{AliasSource, ApiAlias, ApiFormat};
use sqlx::query_as;
use std::str::FromStr;

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
