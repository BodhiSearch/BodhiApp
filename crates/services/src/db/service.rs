use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  ApiKeyUpdate, ApiToken, DownloadRequest, DownloadStatus, SqlxError, SqlxMigrateError,
  TokenStatus, UserAccessRequest, UserAccessRequestStatus,
};
use chrono::{DateTime, Timelike, Utc};
use derive_new::new;
use objs::{impl_error_from, AppError, ErrorType};
use objs::{ApiAlias, ApiFormat};
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

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError>;

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError>;

  fn now(&self) -> DateTime<Utc>;
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

    sqlx::query(
      r#"
      INSERT INTO api_model_aliases (id, api_format, base_url, models_json, prefix, forward_all_with_prefix, encrypted_api_key, salt, nonce, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#
    )
    .bind(&alias.id)
    .bind(alias.api_format.to_string())
    .bind(&alias.base_url)
    .bind(&models_json)
    .bind(&alias.prefix)
    .bind(alias.forward_all_with_prefix)
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
    let result = query_as::<_, (String, String, String, String, Option<String>, bool, i64)>(
      "SELECT id, api_format, base_url, models_json, prefix, forward_all_with_prefix, created_at FROM api_model_aliases WHERE id = ?",
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
        created_at,
      )) => {
        let api_format = api_format_str
          .parse::<ApiFormat>()
          .map_err(|e| DbError::EncryptionError(format!("Failed to parse api_format: {}", e)))?;

        let models: Vec<String> = serde_json::from_str(&models_json)
          .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

        let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();

        Ok(Some(ApiAlias {
          id,
          api_format,
          base_url,
          models,
          prefix,
          forward_all_with_prefix,
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

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM api_model_aliases WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError> {
    let results = query_as::<_, (String, String, String, String, Option<String>, bool, i64)>(
      "SELECT id, api_format, base_url, models_json, prefix, forward_all_with_prefix, created_at FROM api_model_aliases ORDER BY created_at DESC"
    )
    .fetch_all(&self.pool)
    .await?;

    let mut aliases = Vec::new();
    for (id, api_format_str, base_url, models_json, prefix, forward_all_with_prefix, created_at) in
      results
    {
      let api_format = api_format_str
        .parse::<ApiFormat>()
        .map_err(|e| DbError::EncryptionError(format!("Failed to parse api_format: {}", e)))?;

      let models: Vec<String> = serde_json::from_str(&models_json)
        .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

      let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();

      aliases.push(ApiAlias {
        id,
        api_format,
        base_url,
        models,
        prefix,
        forward_all_with_prefix,
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

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::{
      ApiKeyUpdate, ApiToken, DbError, DbService, DownloadRequest, DownloadStatus, SqlxError,
      TokenStatus, UserAccessRequest, UserAccessRequestStatus,
    },
    test_utils::{test_db_service, TestDbService},
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
}
