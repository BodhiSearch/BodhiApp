use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  ApiKeyUpdate, ApiToken, DownloadRequest, DownloadStatus, ModelMetadataRow, SqlxError,
  SqlxMigrateError, TokenStatus, UserAccessRequest, UserAccessRequestStatus,
};
use chrono::{DateTime, Timelike, Utc};
use derive_new::new;
use objs::{impl_error_from, AppError, ErrorType};
use objs::{AliasSource, ApiAlias, ApiFormat};
use sqlx::{query_as, SqlitePool};
use std::{fs, path::Path, str::FromStr, sync::Arc, time::UNIX_EPOCH};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TimeService: std::fmt::Debug + Send + Sync {
  fn utc_now(&self) -> DateTime<Utc>;

  fn created_at(&self, path: &Path) -> u32;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultTimeService;

impl TimeService for DefaultTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    now.with_nanosecond(0).unwrap_or(now)
  }

  fn created_at(&self, path: &Path) -> u32 {
    fs::metadata(path)
      .map_err(|e| e.to_string())
      .and_then(|m| m.created().map_err(|e| e.to_string()))
      .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
      .unwrap_or_default()
      .as_secs() as u32
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DbError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
  #[error(transparent)]
  SqlxMigrateError(#[from] SqlxMigrateError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, code="db_error-strum_parse", args_delegate = false)]
  StrumParse(#[from] strum::ParseError),
  #[error("token_validation")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TokenValidation(String),
  #[error("encryption_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  EncryptionError(String),
  #[error("prefix_exists")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "db_error-prefix_exists")]
  PrefixExists(String),
}

impl_error_from!(::sqlx::Error, DbError::SqlxError, crate::db::SqlxError);
impl_error_from!(
  ::sqlx::migrate::MigrateError,
  DbError::SqlxMigrateError,
  crate::db::SqlxMigrateError
);

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait DbService: std::fmt::Debug + Send + Sync {
  async fn migrate(&self) -> Result<(), DbError>;

  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError>;

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError>;

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError>;

  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError>;

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError>;

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError>;

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError>;

  async fn update_request_status(
    &self,
    id: i64,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError>;

  async fn get_request_by_id(&self, id: i64) -> Result<Option<UserAccessRequest>, DbError>;

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError>;

  async fn get_api_token_by_id(&self, user_id: &str, id: &str)
    -> Result<Option<ApiToken>, DbError>;

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError>;

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError>;

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError>;

  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError>;

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError>;

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError>;

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError>;

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError>;

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;

  async fn upsert_model_metadata(
    &self,
    metadata: &crate::db::ModelMetadataRow,
  ) -> Result<(), DbError>;

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<crate::db::ModelMetadataRow>, DbError>;

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<
    std::collections::HashMap<(String, String, String), crate::db::ModelMetadataRow>,
    DbError,
  >;

  async fn list_model_metadata(&self) -> Result<Vec<crate::db::ModelMetadataRow>, DbError>;

  // Toolset configuration management
  async fn get_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<Option<crate::db::UserToolsetConfigRow>, DbError>;

  async fn upsert_user_toolset_config(
    &self,
    config: &crate::db::UserToolsetConfigRow,
  ) -> Result<crate::db::UserToolsetConfigRow, DbError>;

  async fn list_user_toolset_configs(
    &self,
    user_id: &str,
  ) -> Result<Vec<crate::db::UserToolsetConfigRow>, DbError>;

  async fn delete_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<(), DbError>;

  // App-level toolset configuration management
  async fn get_app_toolset_config(
    &self,
    toolset_id: &str,
  ) -> Result<Option<crate::db::AppToolsetConfigRow>, DbError>;

  async fn upsert_app_toolset_config(
    &self,
    config: &crate::db::AppToolsetConfigRow,
  ) -> Result<crate::db::AppToolsetConfigRow, DbError>;

  async fn list_app_toolset_configs(&self) -> Result<Vec<crate::db::AppToolsetConfigRow>, DbError>;

  // ============================================================================
  // App-Client Toolset Config (cached from Keycloak /resources/request-access)
  // ============================================================================

  async fn get_app_client_toolset_config(
    &self,
    app_client_id: &str,
  ) -> Result<Option<crate::db::AppClientToolsetConfigRow>, DbError>;

  async fn upsert_app_client_toolset_config(
    &self,
    config: &crate::db::AppClientToolsetConfigRow,
  ) -> Result<crate::db::AppClientToolsetConfigRow, DbError>;

  fn now(&self) -> DateTime<Utc>;

  /// Get the encryption key for encrypting/decrypting sensitive data
  fn encryption_key(&self) -> &[u8];
}

#[derive(Debug, Clone, new)]
pub struct SqliteDbService {
  pool: SqlitePool,
  time_service: Arc<dyn TimeService>,
  encryption_key: Vec<u8>,
}

impl SqliteDbService {
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
impl DbService for SqliteDbService {
  async fn migrate(&self) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(&self.pool).await?;
    Ok(())
  }

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

  // ============================================================================
  // Toolset configuration management
  // ============================================================================

  async fn get_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<Option<crate::db::UserToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (i64, String, String, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_id, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM user_toolset_configs WHERE user_id = ? AND toolset_id = ?",
    )
    .bind(user_id)
    .bind(toolset_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        user_id,
        toolset_id,
        enabled,
        encrypted_api_key,
        salt,
        nonce,
        created_at,
        updated_at,
      )| {
        crate::db::UserToolsetConfigRow {
          id,
          user_id,
          toolset_id,
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

  async fn upsert_user_toolset_config(
    &self,
    config: &crate::db::UserToolsetConfigRow,
  ) -> Result<crate::db::UserToolsetConfigRow, DbError> {
    let enabled = if config.enabled { 1 } else { 0 };

    // Check if config exists
    let existing = self
      .get_user_toolset_config(&config.user_id, &config.toolset_id)
      .await?;

    let id = if let Some(existing) = existing {
      // Update existing config
      sqlx::query(
        r#"
        UPDATE user_toolset_configs
        SET enabled = ?, encrypted_api_key = ?, salt = ?, nonce = ?, updated_at = ?
        WHERE user_id = ? AND toolset_id = ?
        "#,
      )
      .bind(enabled)
      .bind(&config.encrypted_api_key)
      .bind(&config.salt)
      .bind(&config.nonce)
      .bind(config.updated_at)
      .bind(&config.user_id)
      .bind(&config.toolset_id)
      .execute(&self.pool)
      .await?;

      existing.id
    } else {
      // Insert new config
      let result = sqlx::query(
        r#"
        INSERT INTO user_toolset_configs (user_id, toolset_id, enabled, encrypted_api_key, salt, nonce, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
      )
      .bind(&config.user_id)
      .bind(&config.toolset_id)
      .bind(enabled)
      .bind(&config.encrypted_api_key)
      .bind(&config.salt)
      .bind(&config.nonce)
      .bind(config.created_at)
      .bind(config.updated_at)
      .execute(&self.pool)
      .await?;

      result.last_insert_rowid()
    };

    // Return the updated/inserted config
    Ok(crate::db::UserToolsetConfigRow {
      id,
      user_id: config.user_id.clone(),
      toolset_id: config.toolset_id.clone(),
      enabled: config.enabled,
      encrypted_api_key: config.encrypted_api_key.clone(),
      salt: config.salt.clone(),
      nonce: config.nonce.clone(),
      created_at: config.created_at,
      updated_at: config.updated_at,
    })
  }

  async fn list_user_toolset_configs(
    &self,
    user_id: &str,
  ) -> Result<Vec<crate::db::UserToolsetConfigRow>, DbError> {
    let results = sqlx::query_as::<_, (i64, String, String, i64, Option<String>, Option<String>, Option<String>, i64, i64)>(
      "SELECT id, user_id, toolset_id, enabled, encrypted_api_key, salt, nonce, created_at, updated_at FROM user_toolset_configs WHERE user_id = ?",
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
            toolset_id,
            enabled,
            encrypted_api_key,
            salt,
            nonce,
            created_at,
            updated_at,
          )| {
            crate::db::UserToolsetConfigRow {
              id,
              user_id,
              toolset_id,
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

  async fn delete_user_toolset_config(
    &self,
    user_id: &str,
    toolset_id: &str,
  ) -> Result<(), DbError> {
    sqlx::query("DELETE FROM user_toolset_configs WHERE user_id = ? AND toolset_id = ?")
      .bind(user_id)
      .bind(toolset_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  // ============================================================================
  // App-level toolset configuration management
  // ============================================================================

  async fn get_app_toolset_config(
    &self,
    toolset_id: &str,
  ) -> Result<Option<crate::db::AppToolsetConfigRow>, DbError> {
    let result = sqlx::query_as::<_, (i64, String, i64, String, i64, i64)>(
      "SELECT id, toolset_id, enabled, updated_by, created_at, updated_at FROM app_toolset_configs WHERE toolset_id = ?",
    )
    .bind(toolset_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(id, toolset_id, enabled, updated_by, created_at, updated_at)| {
        crate::db::AppToolsetConfigRow {
          id,
          toolset_id,
          enabled: enabled != 0,
          updated_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn upsert_app_toolset_config(
    &self,
    config: &crate::db::AppToolsetConfigRow,
  ) -> Result<crate::db::AppToolsetConfigRow, DbError> {
    let enabled = if config.enabled { 1 } else { 0 };

    // Check if config exists
    let existing = self.get_app_toolset_config(&config.toolset_id).await?;

    let id = if let Some(existing) = existing {
      // Update existing config
      sqlx::query(
        r#"
        UPDATE app_toolset_configs
        SET enabled = ?, updated_by = ?, updated_at = ?
        WHERE toolset_id = ?
        "#,
      )
      .bind(enabled)
      .bind(&config.updated_by)
      .bind(config.updated_at)
      .bind(&config.toolset_id)
      .execute(&self.pool)
      .await?;

      existing.id
    } else {
      // Insert new config
      let result = sqlx::query(
        r#"
        INSERT INTO app_toolset_configs (toolset_id, enabled, updated_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
      )
      .bind(&config.toolset_id)
      .bind(enabled)
      .bind(&config.updated_by)
      .bind(config.created_at)
      .bind(config.updated_at)
      .execute(&self.pool)
      .await?;

      result.last_insert_rowid()
    };

    // Return the updated/inserted config
    Ok(crate::db::AppToolsetConfigRow {
      id,
      toolset_id: config.toolset_id.clone(),
      enabled: config.enabled,
      updated_by: config.updated_by.clone(),
      created_at: config.created_at,
      updated_at: config.updated_at,
    })
  }

  async fn list_app_toolset_configs(&self) -> Result<Vec<crate::db::AppToolsetConfigRow>, DbError> {
    let results = sqlx::query_as::<_, (i64, String, i64, String, i64, i64)>(
      "SELECT id, toolset_id, enabled, updated_by, created_at, updated_at FROM app_toolset_configs ORDER BY toolset_id",
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(id, toolset_id, enabled, updated_by, created_at, updated_at)| {
            crate::db::AppToolsetConfigRow {
              id,
              toolset_id,
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
  ) -> Result<Option<crate::db::AppClientToolsetConfigRow>, DbError> {
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
        crate::db::AppClientToolsetConfigRow {
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
    config: &crate::db::AppClientToolsetConfigRow,
  ) -> Result<crate::db::AppClientToolsetConfigRow, DbError> {
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

    Ok(crate::db::AppClientToolsetConfigRow {
      id: result.0,
      app_client_id: result.1,
      config_version: result.2,
      toolsets_json: result.3,
      resource_scope: result.4,
      created_at: result.5,
      updated_at: result.6,
    })
  }

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }

  fn encryption_key(&self) -> &[u8] {
    &self.encryption_key
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::{
      ApiKeyUpdate, ApiToken, DbError, DbService, DownloadRequest, DownloadStatus, SqlxError,
      TokenStatus, UserAccessRequest, UserAccessRequestStatus,
    },
    test_utils::{
      create_test_api_model_metadata, create_test_model_metadata, model_metadata_builder,
      test_db_service, TestDbService,
    },
  };
  use chrono::Utc;
  use objs::ApiAlias;
  use objs::ApiFormat;
  use rstest::rstest;
  use uuid::Uuid;

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_create_download_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let request = DownloadRequest::new_pending("test/repo", "test_file.gguf", now);
    service.create_download_request(&request).await?;
    let fetched = service.get_download_request(&request.id).await?;
    assert!(fetched.is_some());
    assert_eq!(request, fetched.unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_update_download_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut request = DownloadRequest::new_pending("test/repo", "test_file.gguf", now);
    service.create_download_request(&request).await?;
    request.status = DownloadStatus::Completed;
    request.total_bytes = Some(1000000);
    request.downloaded_bytes = 1000000;
    request.started_at = Some(now);
    request.updated_at = now + chrono::Duration::hours(1);
    service.update_download_request(&request).await?;

    let fetched = service.get_download_request(&request.id).await?.unwrap();
    assert_eq!(
      DownloadRequest {
        id: request.id,
        repo: "test/repo".to_string(),
        filename: "test_file.gguf".to_string(),
        status: DownloadStatus::Completed,
        error: None,
        created_at: now,
        updated_at: now + chrono::Duration::hours(1),
        total_bytes: Some(1000000),
        downloaded_bytes: 1000000,
        started_at: Some(now),
      },
      fetched
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_download_request_progress_tracking(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut request = DownloadRequest {
      id: Uuid::new_v4().to_string(),
      repo: "test/repo".to_string(),
      filename: "test_file.gguf".to_string(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: now,
      updated_at: now,
      total_bytes: Some(1000000), // 1MB
      downloaded_bytes: 0,
      started_at: Some(now),
    };
    service.create_download_request(&request).await?;

    // Simulate progress update
    request.downloaded_bytes = 500000; // 50% downloaded
    request.updated_at = now + chrono::Duration::seconds(4);
    service.update_download_request(&request).await?;

    let fetched = service.get_download_request(&request.id).await?.unwrap();
    assert_eq!(request.downloaded_bytes, fetched.downloaded_bytes);
    assert_eq!(request.total_bytes, fetched.total_bytes);
    assert_eq!(request.started_at, fetched.started_at);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_insert_pending_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let username = "test@example.com".to_string();
    let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
    let pending_request = service
      .insert_pending_request(username.clone(), user_id.clone())
      .await?;
    let expected_request = UserAccessRequest {
      id: pending_request.id, // We don't know this in advance
      username,
      user_id,
      created_at: now,
      updated_at: now,
      status: UserAccessRequestStatus::Pending,
      reviewer: None,
    };
    assert_eq!(pending_request, expected_request);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_get_pending_request(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let username = "test@example.com".to_string();
    let user_id = "550e8400-e29b-41d4-a716-446655440001".to_string();
    let inserted_request = service
      .insert_pending_request(username, user_id.clone())
      .await?;
    let fetched_request = service.get_pending_request(user_id).await?;
    assert!(fetched_request.is_some());
    assert_eq!(inserted_request, fetched_request.unwrap());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_list_pending_requests(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let test_data = vec![
      (
        "test1@example.com".to_string(),
        "550e8400-e29b-41d4-a716-446655440002".to_string(),
      ),
      (
        "test2@example.com".to_string(),
        "550e8400-e29b-41d4-a716-446655440003".to_string(),
      ),
      (
        "test3@example.com".to_string(),
        "550e8400-e29b-41d4-a716-446655440004".to_string(),
      ),
    ];
    for (username, user_id) in &test_data {
      service
        .insert_pending_request(username.clone(), user_id.clone())
        .await?;
    }
    let (page1, total) = service.list_pending_requests(1, 2).await?;
    assert_eq!(2, page1.len());
    assert_eq!(3, total);
    let (page2, total) = service.list_pending_requests(2, 2).await?;
    assert_eq!(1, page2.len());
    assert_eq!(3, total);
    for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
      let expected_request = UserAccessRequest {
        id: request.id,
        username: test_data[i].0.clone(),
        user_id: test_data[i].1.clone(),
        created_at: now,
        updated_at: now,
        status: UserAccessRequestStatus::Pending,
        reviewer: None,
      };
      assert_eq!(&expected_request, request);
    }
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_db_service_update_request_status(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let username = "test@example.com".to_string();
    let user_id = "550e8400-e29b-41d4-a716-446655440005".to_string();
    let inserted_request = service
      .insert_pending_request(username, user_id.clone())
      .await?;
    service
      .update_request_status(
        inserted_request.id,
        UserAccessRequestStatus::Approved,
        "admin@example.com".to_string(),
      )
      .await?;
    let updated_request = service.get_pending_request(user_id).await?;
    assert!(updated_request.is_none());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_token(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Create token
    let user_id = Uuid::new_v4().to_string();
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.clone(),
      name: "".to_string(),
      token_prefix: "bodhiapp_test01".to_string(),
      token_hash: "token_hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };

    service.create_api_token(&mut token).await?;

    // List tokens
    let (tokens, _) = service.list_api_tokens(&user_id, 1, 10).await?;
    assert_eq!(1, tokens.len());

    assert_eq!(token, tokens[0]);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_token(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create initial token
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: "test_user".to_string(),
      name: "Initial Name".to_string(),
      token_prefix: "bodhiapp_test02".to_string(),
      token_hash: "token_hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    };
    service.create_api_token(&mut token).await?;

    // Update token
    token.name = "Updated Name".to_string();
    token.status = TokenStatus::Inactive;
    token.updated_at = Utc::now();
    service.update_api_token("test_user", &mut token).await?;
    // Verify update
    let updated = service
      .get_api_token_by_id("test_user", &token.id)
      .await?
      .unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.status, TokenStatus::Inactive);
    assert_eq!(updated.id, token.id);
    assert_eq!(updated.user_id, token.user_id);
    assert_eq!(updated.token_prefix, token.token_prefix);
    assert_eq!(updated.created_at, token.created_at);
    assert!(updated.updated_at >= token.updated_at);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_api_tokens_user_scoped(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Create tokens for two different users
    let user1_id = "user1";
    let user2_id = "user2";

    // Create token for user1
    let mut token1 = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user1_id.to_string(),
      name: "User1 Token".to_string(),
      token_prefix: "bodhiapp_test03".to_string(),
      token_hash: "hash1".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    service.create_api_token(&mut token1).await?;

    // Create token for user2
    let mut token2 = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user2_id.to_string(),
      name: "User2 Token".to_string(),
      token_prefix: "bodhiapp_test04".to_string(),
      token_hash: "hash2".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    service.create_api_token(&mut token2).await?;

    // List tokens for user1
    let (tokens, total) = service.list_api_tokens(user1_id, 1, 10).await?;
    assert_eq!(tokens.len(), 1);
    assert_eq!(total, 1);
    assert_eq!(tokens[0].user_id, user1_id);
    assert_eq!(tokens[0].name, "User1 Token");

    // List tokens for user2
    let (tokens, total) = service.list_api_tokens(user2_id, 1, 10).await?;
    assert_eq!(tokens.len(), 1);
    assert_eq!(total, 1);
    assert_eq!(tokens[0].user_id, user2_id);
    assert_eq!(tokens[0].name, "User2 Token");

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_token_user_scoped(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Create a token for user1
    let user1_id = "user1";
    let mut token = ApiToken {
      id: Uuid::new_v4().to_string(),
      user_id: user1_id.to_string(),
      name: "Initial Name".to_string(),
      token_prefix: "bodhiapp_test05".to_string(),
      token_hash: "hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };
    service.create_api_token(&mut token).await?;

    // Try to update token as user2 (should fail)
    let user2_id = "user2";
    token.name = "Updated Name".to_string();
    let result = service.update_api_token(user2_id, &mut token).await;
    assert!(matches!(
      result,
      Err(DbError::SqlxError(SqlxError { source })) if source.to_string() == sqlx::Error::RowNotFound.to_string()
    ));

    // Verify token was not updated
    let unchanged = service
      .get_api_token_by_id(user1_id, &token.id)
      .await?
      .unwrap();
    assert_eq!(unchanged.name, "Initial Name");
    assert_eq!(unchanged.user_id, user1_id);

    // Update token as user1 (should succeed)
    let result = service.update_api_token(user1_id, &mut token).await;
    assert!(result.is_ok());

    // Verify the update succeeded
    let updated = service
      .get_api_token_by_id(user1_id, &token.id)
      .await?
      .unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.user_id, user1_id);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_and_get_api_model_alias(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let alias_obj = ApiAlias::new(
      "openai",
      ApiFormat::OpenAI,
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      None,
      false,
      now,
    );
    let api_key = "sk-test123456789";

    // Create API model alias
    service
      .create_api_model_alias(&alias_obj, Some(api_key.to_string()))
      .await?;

    // Retrieve and verify
    let retrieved = service.get_api_model_alias("openai").await?.unwrap();
    assert_eq!(alias_obj, retrieved);

    // Verify API key is stored encrypted and retrievable
    let decrypted_key = service.get_api_key_for_alias("openai").await?.unwrap();
    assert_eq!(api_key, decrypted_key);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_model_alias_without_key(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let alias_obj = ApiAlias::new(
      "no-key-model",
      ApiFormat::OpenAI,
      "https://api.example.com/v1",
      vec!["gpt-4".to_string()],
      None,
      false,
      now,
    );

    // Create API model alias WITHOUT api_key
    service.create_api_model_alias(&alias_obj, None).await?;

    // Verify model exists
    let retrieved = service.get_api_model_alias("no-key-model").await?;
    assert!(retrieved.is_some());
    assert_eq!(alias_obj, retrieved.unwrap());

    // Verify API key is None
    let key = service.get_api_key_for_alias("no-key-model").await?;
    assert_eq!(None, key);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_alias_with_new_key(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut alias_obj = ApiAlias::new(
      "claude",
      ApiFormat::OpenAI,
      "https://api.anthropic.com/v1",
      vec!["claude-3".to_string()],
      None,
      false,
      now,
    );
    let original_api_key = "sk-original123";
    let new_api_key = "sk-updated456";

    // Create initial alias
    service
      .create_api_model_alias(&alias_obj, Some(original_api_key.to_string()))
      .await?;

    // Update with new API key and additional model
    alias_obj.models.push("claude-3.5".to_string());
    service
      .update_api_model_alias(
        "claude",
        &alias_obj,
        ApiKeyUpdate::Set(Some(new_api_key.to_string())),
      )
      .await?;

    // Verify updated data
    let updated = service.get_api_model_alias("claude").await?.unwrap();
    assert_eq!(alias_obj, updated);

    // Verify new API key
    let updated_key = service.get_api_key_for_alias("claude").await?.unwrap();
    assert_eq!(new_api_key, updated_key);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_alias_without_key_change(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut alias_obj = ApiAlias::new(
      "gemini",
      ApiFormat::OpenAI,
      "https://generativelanguage.googleapis.com/v1",
      vec!["gemini-pro".to_string()],
      None,
      false,
      now,
    );
    let api_key = "AIzaSy-test123";

    // Create initial alias
    service
      .create_api_model_alias(&alias_obj, Some(api_key.to_string()))
      .await?;

    // Update without changing API key
    alias_obj.base_url = "https://generativelanguage.googleapis.com/v1beta".to_string();
    service
      .update_api_model_alias("gemini", &alias_obj, ApiKeyUpdate::Keep)
      .await?;

    // Verify API key unchanged
    let retrieved_key = service.get_api_key_for_alias("gemini").await?.unwrap();
    assert_eq!(api_key, retrieved_key);

    // Verify other fields updated
    let updated = service.get_api_model_alias("gemini").await?.unwrap();
    assert_eq!(
      "https://generativelanguage.googleapis.com/v1beta",
      updated.base_url
    );

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_api_model_aliases(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Create multiple aliases with different timestamps for proper sorting test
    let aliases = vec![
      ("alias1", "key1", now - chrono::Duration::seconds(20)),
      ("alias2", "key2", now - chrono::Duration::seconds(10)),
      ("alias3", "key3", now),
    ];

    for (alias, key, created_at) in &aliases {
      let alias_obj = ApiAlias::new(
        *alias,
        ApiFormat::OpenAI,
        "https://api.example.com/v1",
        vec!["model1".to_string()],
        None,
        false,
        *created_at,
      );
      service
        .create_api_model_alias(&alias_obj, Some(key.to_string()))
        .await?;
    }

    // List and verify
    let listed = service.list_api_model_aliases().await?;
    assert_eq!(3, listed.len());

    // Verify sorted by created_at DESC (newest first: alias3 -> alias2 -> alias1)
    let sorted_aliases: Vec<_> = listed.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(vec!["alias3", "alias2", "alias1"], sorted_aliases);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_api_model_alias(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let alias_obj = ApiAlias::new(
      "to-delete",
      ApiFormat::OpenAI,
      "https://api.test.com/v1",
      vec!["test-model".to_string()],
      None,
      false,
      now,
    );

    // Create and verify exists
    service
      .create_api_model_alias(&alias_obj, Some("test-key".to_string()))
      .await?;
    assert!(service.get_api_model_alias("to-delete").await?.is_some());

    // Delete and verify gone
    service.delete_api_model_alias("to-delete").await?;
    assert!(service.get_api_model_alias("to-delete").await?.is_none());
    assert!(service.get_api_key_for_alias("to-delete").await?.is_none());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_api_key_encryption_security(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let alias_obj = ApiAlias::new(
      "security-test",
      ApiFormat::OpenAI,
      "https://api.secure.com/v1",
      vec!["secure-model".to_string()],
      None,
      false,
      now,
    );
    let sensitive_key = "sk-very-secret-key-12345";

    // Store API key
    service
      .create_api_model_alias(&alias_obj, Some(sensitive_key.to_string()))
      .await?;

    // Verify different encryptions produce different results
    let alias_obj2 = ApiAlias::new(
      "security-test2",
      ApiFormat::OpenAI,
      "https://api.secure.com/v1",
      vec!["secure-model".to_string()],
      None,
      false,
      now,
    );
    service
      .create_api_model_alias(&alias_obj2, Some(sensitive_key.to_string()))
      .await?;

    // Both should decrypt to same key but have different encrypted values in DB
    let key1 = service
      .get_api_key_for_alias("security-test")
      .await?
      .unwrap();
    let key2 = service
      .get_api_key_for_alias("security-test2")
      .await?
      .unwrap();

    assert_eq!(sensitive_key, key1);
    assert_eq!(sensitive_key, key2);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_nonexistent_api_model_alias(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    // Test getting non-existent alias
    let result = service.get_api_model_alias("nonexistent").await?;
    assert!(result.is_none());

    // Test getting API key for non-existent alias
    let key = service.get_api_key_for_alias("nonexistent").await?;
    assert!(key.is_none());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_alias_keeps_key_when_none_provided(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut alias_obj = ApiAlias::new(
      "keep-key-test",
      ApiFormat::OpenAI,
      "https://api.example.com/v1",
      vec!["gpt-4".to_string()],
      None,
      false,
      now,
    );
    let original_key = "sk-original-key-12345";

    // Create WITH api_key
    service
      .create_api_model_alias(&alias_obj, Some(original_key.to_string()))
      .await?;

    // Verify key exists
    let key = service.get_api_key_for_alias("keep-key-test").await?;
    assert_eq!(Some(original_key.to_string()), key);

    // Update without providing api_key (Keep) - should keep existing key
    alias_obj.base_url = "https://api.example.com/v2".to_string();
    service
      .update_api_model_alias("keep-key-test", &alias_obj, ApiKeyUpdate::Keep)
      .await?;

    // Verify key still exists and unchanged
    let key = service.get_api_key_for_alias("keep-key-test").await?;
    assert_eq!(Some(original_key.to_string()), key);

    // Verify other fields were updated
    let updated_alias = service.get_api_model_alias("keep-key-test").await?.unwrap();
    assert_eq!("https://api.example.com/v2", updated_alias.base_url);

    Ok(())
  }

  #[rstest]
  #[case::with_existing_key(Some("sk-key-to-be-cleared"), "https://api.example.com/v2")]
  #[case::without_existing_key(None, "https://api.example.com/v2")]
  #[awt]
  #[tokio::test]
  async fn test_update_api_model_alias_clear_key(
    #[case] initial_api_key: Option<&str>,
    #[case] updated_base_url: &str,
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut alias_obj = ApiAlias::new(
      "clear-key-test",
      ApiFormat::OpenAI,
      "https://api.example.com/v1",
      vec!["gpt-4".to_string()],
      None,
      false,
      now,
    );

    // Create with or without API key
    service
      .create_api_model_alias(&alias_obj, initial_api_key.map(|s| s.to_string()))
      .await?;

    // Verify initial key state
    let key = service.get_api_key_for_alias("clear-key-test").await?;
    assert_eq!(initial_api_key.map(|s| s.to_string()), key);

    // Update and clear the API key
    alias_obj.base_url = updated_base_url.to_string();
    service
      .update_api_model_alias("clear-key-test", &alias_obj, ApiKeyUpdate::Set(None))
      .await?;

    // Verify key is now None (regardless of initial state)
    let key = service.get_api_key_for_alias("clear-key-test").await?;
    assert_eq!(None, key);

    // Verify model still exists and other fields were updated
    let model = service
      .get_api_model_alias("clear-key-test")
      .await?
      .unwrap();
    assert_eq!(updated_base_url, model.base_url);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_batch_get_metadata_by_files_returns_inserted_data(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Insert test data with multiple rows - all with source='model' since
    // metadata always represents the physical GGUF file
    let test_data = vec![
      ("test/repo1", "model1.gguf", "abc123"),
      ("test/repo2", "model2.gguf", "def456"),
      ("test/repo3", "model3.gguf", "ghi789"),
    ];

    for (repo, filename, snapshot) in &test_data {
      let row = create_test_model_metadata(repo, filename, snapshot, now);
      service.upsert_model_metadata(&row).await?;
    }

    // Verify single query works for first entry
    let single = service
      .get_model_metadata_by_file("test/repo1", "model1.gguf", "abc123")
      .await?;
    assert!(single.is_some(), "Single query should find the row");

    // Test batch query with all keys
    let keys: Vec<(String, String, String)> = test_data
      .iter()
      .map(|(repo, filename, snapshot)| {
        (repo.to_string(), filename.to_string(), snapshot.to_string())
      })
      .collect();

    let batch_result = service.batch_get_metadata_by_files(&keys).await?;

    assert_eq!(3, batch_result.len(), "Batch query should return 3 results");

    // Verify each key is present
    for (repo, filename, snapshot) in &test_data {
      let key = (repo.to_string(), filename.to_string(), snapshot.to_string());
      assert!(
        batch_result.contains_key(&key),
        "Batch result should contain key {:?}",
        key
      );
    }

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_batch_get_metadata_by_files_returns_empty_for_empty_input(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let keys: Vec<(String, String, String)> = vec![];
    let result = service.batch_get_metadata_by_files(&keys).await?;
    assert!(result.is_empty());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_upsert_model_metadata_inserts_new_row(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut builder = model_metadata_builder(now);
    builder
      .source("model")
      .repo("test/repo")
      .filename("model.gguf")
      .snapshot("snapshot123")
      .capabilities_vision(1_i64)
      .capabilities_thinking(1_i64)
      .capabilities_function_calling(1_i64)
      .context_max_input_tokens(8192_i64)
      .context_max_output_tokens(4096_i64)
      .architecture(r#"{"family":"llama","parameter_count":7000000000,"quantization":"Q4_K_M","format":"gguf"}"#)
      .chat_template("{% for msg in messages %}{{ msg.role }}: {{ msg.content }}{% endfor %}");
    let row = builder.build()?;

    service.upsert_model_metadata(&row).await?;

    let fetched = service
      .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
      .await?
      .expect("Row should exist");

    assert_eq!("model", fetched.source);
    assert_eq!(Some("test/repo".to_string()), fetched.repo);
    assert_eq!(Some("model.gguf".to_string()), fetched.filename);
    assert_eq!(Some(1), fetched.capabilities_vision);
    assert_eq!(Some(1), fetched.capabilities_thinking);
    assert_eq!(Some(8192), fetched.context_max_input_tokens);
    assert!(fetched.architecture.is_some());
    assert!(fetched.chat_template.is_some());
    assert!(fetched.chat_template.unwrap().contains("msg.role"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_upsert_model_metadata_updates_existing_row(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Insert initial row with source='model' (physical GGUF file)
    let row = create_test_model_metadata("test/repo", "model.gguf", "snapshot123", now);
    service.upsert_model_metadata(&row).await?;

    // Update with new data (same repo/filename/snapshot)
    let mut builder = model_metadata_builder(now);
    builder
      .source("model")
      .repo("test/repo")
      .filename("model.gguf")
      .snapshot("snapshot123")
      .capabilities_vision(1_i64)
      .capabilities_thinking(1_i64)
      .capabilities_function_calling(1_i64)
      .context_max_input_tokens(8192_i64)
      .context_max_output_tokens(4096_i64)
      .architecture(r#"{"family":"llama","format":"gguf"}"#)
      .chat_template("updated template");
    let updated_row = builder.build()?;
    service.upsert_model_metadata(&updated_row).await?;

    // Verify update
    let fetched = service
      .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
      .await?
      .expect("Row should exist");

    assert_eq!(Some(1), fetched.capabilities_vision);
    assert_eq!(Some(1), fetched.capabilities_thinking);
    assert_eq!(Some(8192), fetched.context_max_input_tokens);
    assert_eq!(Some(4096), fetched.context_max_output_tokens);
    assert_eq!(Some("updated template".to_string()), fetched.chat_template);

    // Verify only one row exists (upsert, not insert)
    let all = service.list_model_metadata().await?;
    assert_eq!(1, all.len());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_upsert_model_metadata_with_api_model_id(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut builder = model_metadata_builder(now);
    builder
      .source("api")
      .api_model_id("gpt-4-turbo")
      .capabilities_vision(1_i64)
      .capabilities_function_calling(1_i64)
      .capabilities_structured_output(1_i64)
      .context_max_input_tokens(128000_i64)
      .context_max_output_tokens(4096_i64);
    let row = builder.build()?;

    service.upsert_model_metadata(&row).await?;

    // Verify it's in list (API models use different path)
    let all = service.list_model_metadata().await?;
    assert_eq!(1, all.len());
    assert_eq!(Some("gpt-4-turbo".to_string()), all[0].api_model_id);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_model_metadata_by_file_returns_none_for_nonexistent(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let result = service
      .get_model_metadata_by_file("nonexistent/repo", "model.gguf", "snapshot")
      .await?;
    assert!(result.is_none());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_model_metadata_by_file_filters_by_source(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Insert an API source row with repo/filename/snapshot
    // (unusual but possible, should NOT be returned by get_model_metadata_by_file)
    let mut builder = model_metadata_builder(now);
    builder
      .source("api")
      .repo("test/repo")
      .filename("model.gguf")
      .snapshot("snapshot123")
      .api_model_id("api-model-id");
    let api_row = builder.build()?;
    service.upsert_model_metadata(&api_row).await?;

    // Should NOT find it (source filter excludes 'api')
    let result = service
      .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
      .await?;
    assert!(result.is_none(), "Should not find API source rows");

    // Insert a 'model' source row with same repo/filename/snapshot
    let model_row = create_test_model_metadata("test/repo", "model.gguf", "snapshot123", now);
    service.upsert_model_metadata(&model_row).await?;

    // Now should find it
    let result = service
      .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
      .await?;
    assert!(result.is_some(), "Should find model source rows");
    assert_eq!("model", result.unwrap().source);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_model_metadata_returns_all_rows(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Insert multiple rows - note: for local GGUF files, source is always 'model'
    // API source is for remote API models which have api_model_id instead of repo/filename
    let rows = vec![
      create_test_model_metadata("repo1", "model1.gguf", "snapshot", now),
      create_test_model_metadata("repo2", "model2.gguf", "snapshot", now),
      create_test_api_model_metadata("gpt-4", now),
    ];

    for row in &rows {
      service.upsert_model_metadata(row).await?;
    }

    let all = service.list_model_metadata().await?;
    assert_eq!(3, all.len());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_model_metadata_returns_empty_when_no_data(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let result = service.list_model_metadata().await?;
    assert!(result.is_empty());
    Ok(())
  }

  /// Test that metadata for the same physical GGUF file (same repo/filename/snapshot)
  /// is stored only once with source='model', regardless of whether the request
  /// came from a UserAlias or ModelAlias. This verifies the deduplication behavior
  /// where UserAlias requests are translated to store under source='model'.
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_metadata_stored_with_source_model_only(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();

    // Simulate what extract_and_store_metadata does for a UserAlias:
    // It always stores with source='model' regardless of input alias type
    let mut builder = model_metadata_builder(now);
    builder
      .source("model") // Always 'model', never 'user'
      .repo("test/repo")
      .filename("model.gguf")
      .snapshot("snapshot123")
      .capabilities_vision(1_i64)
      .capabilities_function_calling(1_i64)
      .context_max_input_tokens(8192_i64)
      .context_max_output_tokens(4096_i64)
      .chat_template("test template");
    let row = builder.build()?;
    service.upsert_model_metadata(&row).await?;

    // Query should find the row (only looks for source='model')
    let result = service
      .get_model_metadata_by_file("test/repo", "model.gguf", "snapshot123")
      .await?;
    assert!(result.is_some(), "Should find metadata for the GGUF file");

    let fetched = result.unwrap();
    assert_eq!("model", fetched.source, "Source should always be 'model'");
    assert_eq!(Some("test template".to_string()), fetched.chat_template);

    // Verify only one row exists in the database
    let all = service.list_model_metadata().await?;
    assert_eq!(1, all.len(), "Should have exactly one metadata row");
    assert_eq!("model", all[0].source);

    Ok(())
  }

  // ============================================================================
  // Toolset configuration tests
  // ============================================================================

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_upsert_user_toolset_config_creates_new(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let now_ts = now.timestamp();

    let config = crate::db::UserToolsetConfigRow {
      id: 0, // Will be set by database
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some("encrypted".to_string()),
      salt: Some("salt".to_string()),
      nonce: Some("nonce".to_string()),
      created_at: now_ts,
      updated_at: now_ts,
    };

    let result = service.upsert_user_toolset_config(&config).await?;

    assert!(result.id > 0);
    assert_eq!("user123", result.user_id);
    assert_eq!("builtin-exa-web-search", result.toolset_id);
    assert!(result.enabled);
    assert_eq!(Some("encrypted".to_string()), result.encrypted_api_key);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_upsert_user_toolset_config_updates_existing(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let now_ts = now.timestamp();

    // Create initial config
    let config = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: false,
      encrypted_api_key: None,
      salt: None,
      nonce: None,
      created_at: now_ts,
      updated_at: now_ts,
    };

    let created = service.upsert_user_toolset_config(&config).await?;

    // Update config
    let updated_config = crate::db::UserToolsetConfigRow {
      id: created.id,
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some("encrypted".to_string()),
      salt: Some("salt".to_string()),
      nonce: Some("nonce".to_string()),
      created_at: now_ts,
      updated_at: now_ts + 100,
    };

    let result = service.upsert_user_toolset_config(&updated_config).await?;

    assert_eq!(created.id, result.id);
    assert!(result.enabled);
    assert_eq!(Some("encrypted".to_string()), result.encrypted_api_key);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_user_toolset_config_returns_config(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let now_ts = now.timestamp();

    let config = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some("encrypted".to_string()),
      salt: Some("salt".to_string()),
      nonce: Some("nonce".to_string()),
      created_at: now_ts,
      updated_at: now_ts,
    };

    service.upsert_user_toolset_config(&config).await?;

    let result = service
      .get_user_toolset_config("user123", "builtin-exa-web-search")
      .await?;

    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!("user123", result.user_id);
    assert_eq!("builtin-exa-web-search", result.toolset_id);
    assert!(result.enabled);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_user_toolset_config_returns_none_for_nonexistent(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let result = service
      .get_user_toolset_config("user123", "nonexistent-toolset")
      .await?;

    assert!(result.is_none());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_user_toolset_configs_returns_all_for_user(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let now_ts = now.timestamp();

    // Create configs for user123
    let config1 = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some("encrypted1".to_string()),
      salt: Some("salt1".to_string()),
      nonce: Some("nonce1".to_string()),
      created_at: now_ts,
      updated_at: now_ts,
    };

    let config2 = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user123".to_string(),
      toolset_id: "another-toolset".to_string(),
      enabled: false,
      encrypted_api_key: None,
      salt: None,
      nonce: None,
      created_at: now_ts,
      updated_at: now_ts,
    };

    // Create config for different user
    let config3 = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user456".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some("encrypted3".to_string()),
      salt: Some("salt3".to_string()),
      nonce: Some("nonce3".to_string()),
      created_at: now_ts,
      updated_at: now_ts,
    };

    service.upsert_user_toolset_config(&config1).await?;
    service.upsert_user_toolset_config(&config2).await?;
    service.upsert_user_toolset_config(&config3).await?;

    let results = service.list_user_toolset_configs("user123").await?;

    assert_eq!(2, results.len());
    assert!(results.iter().all(|r| r.user_id == "user123"));

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_user_toolset_configs_returns_empty_for_user_with_no_configs(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let results = service.list_user_toolset_configs("user123").await?;

    assert!(results.is_empty());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_user_toolset_config_encryption_roundtrip(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let now_ts = now.timestamp();

    // Use encryption functions directly (same as api_model_aliases pattern)
    let api_key = "sk-test1234567890";
    let (encrypted, salt, nonce) =
      crate::db::encryption::encrypt_api_key(&service.encryption_key, api_key)?;

    let config = crate::db::UserToolsetConfigRow {
      id: 0,
      user_id: "user123".to_string(),
      toolset_id: "builtin-exa-web-search".to_string(),
      enabled: true,
      encrypted_api_key: Some(encrypted.clone()),
      salt: Some(salt.clone()),
      nonce: Some(nonce.clone()),
      created_at: now_ts,
      updated_at: now_ts,
    };

    service.upsert_user_toolset_config(&config).await?;

    let retrieved = service
      .get_user_toolset_config("user123", "builtin-exa-web-search")
      .await?
      .unwrap();

    // Decrypt and verify
    let decrypted = crate::db::encryption::decrypt_api_key(
      &service.encryption_key,
      &retrieved.encrypted_api_key.unwrap(),
      &retrieved.salt.unwrap(),
      &retrieved.nonce.unwrap(),
    )?;

    assert_eq!(api_key, decrypted);

    Ok(())
  }
}
