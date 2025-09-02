use crate::{
  db::{
    encryption::{decrypt_api_key, encrypt_api_key},
    AccessRequest, ApiToken, DownloadRequest, DownloadStatus, RequestStatus, SqlxError,
    SqlxMigrateError, TokenStatus,
  },
  extract_claims,
};
use chrono::{DateTime, Timelike, Utc};
use derive_new::new;
use objs::{impl_error_from, AppError, ErrorType};
use objs::{AliasSource, ApiModelAlias};
use sqlx::{query_as, SqlitePool};
use std::{fs, path::Path, str::FromStr, sync::Arc, time::UNIX_EPOCH};
use uuid::Uuid;

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

  async fn insert_pending_request(&self, email: String) -> Result<AccessRequest, DbError>;

  async fn get_pending_request(&self, email: String) -> Result<Option<AccessRequest>, DbError>;

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError>;

  async fn update_request_status(&self, id: i64, status: RequestStatus) -> Result<(), DbError>;

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError>;

  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError>;

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError>;

  async fn get_api_token_by_id(&self, user_id: &str, id: &str)
    -> Result<Option<ApiToken>, DbError>;

  async fn get_api_token_by_token_id(&self, token: &str) -> Result<Option<ApiToken>, DbError>;

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError>;

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError>;

  async fn create_api_model_alias(
    &self,
    alias: &ApiModelAlias,
    api_key: &str,
  ) -> Result<(), DbError>;

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiModelAlias>, DbError>;

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiModelAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError>;

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError>;

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiModelAlias>, DbError>;

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError>;

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
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(query)
    .bind(user_id)
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((id, user_id, name, token_id, token_hash, status, created_at, updated_at)) => {
        let Ok(status) = TokenStatus::from_str(&status) else {
          tracing::warn!("unknown token status: {status} for id: {id}");
          return Ok(None);
        };

        let result = ApiToken {
          id,
          user_id,
          name,
          token_id,
          token_hash,
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

  async fn insert_pending_request(&self, email: String) -> Result<AccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let result = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "INSERT INTO access_requests (email, created_at, updated_at, status)
         VALUES (?, ?, ?, ?)
         RETURNING id, email, created_at, updated_at, status",
    )
    .bind(&email)
    .bind(now)
    .bind(now)
    .bind(RequestStatus::Pending.to_string())
    .fetch_one(&self.pool)
    .await?;

    Ok(AccessRequest {
      id: result.0,
      email: result.1,
      created_at: result.2,
      updated_at: result.3,
      status: RequestStatus::from_str(&result.4)?,
    })
  }

  async fn get_pending_request(&self, email: String) -> Result<Option<AccessRequest>, DbError> {
    let result = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "SELECT id, email, created_at, updated_at, status
         FROM access_requests
         WHERE email = ? AND status = ?",
    )
    .bind(&email)
    .bind(RequestStatus::Pending.to_string())
    .fetch_optional(&self.pool)
    .await?;

    let result = result
      .map(|(id, email, created_at, updated_at, status)| {
        let Ok(status) = RequestStatus::from_str(&status) else {
          tracing::warn!("unknown request status: {} for id: {}", status, id);
          return None;
        };
        let result = AccessRequest {
          id,
          email,
          created_at,
          updated_at,
          status,
        };
        Some(result)
      })
      .unwrap_or(None);
    Ok(result)
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<Vec<AccessRequest>, DbError> {
    let offset = (page - 1) * per_page;
    let results = query_as::<_, (i64, String, DateTime<Utc>, DateTime<Utc>, String)>(
      "SELECT id, email, created_at, updated_at, status
         FROM access_requests
         WHERE status = ?
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(RequestStatus::Pending.to_string())
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(|(id, email, created_at, updated_at, status)| {
        let Ok(status) = RequestStatus::from_str(&status) else {
          tracing::warn!("unknown request status: {} for id: {}", status, id);
          return None;
        };
        let result = AccessRequest {
          id,
          email,
          created_at,
          updated_at,
          status,
        };
        Some(result)
      })
      .collect::<Vec<AccessRequest>>();
    Ok(results)
  }

  async fn update_request_status(&self, id: i64, status: RequestStatus) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    sqlx::query(
      "UPDATE access_requests
         SET status = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(status.to_string())
    .bind(now)
    .bind(id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    token.created_at = now;
    token.updated_at = now;

    sqlx::query(
      r#"
      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&token.id)
    .bind(&token.user_id)
    .bind(&token.name)
    .bind(&token.token_id)
    .bind(&token.token_hash)
    .bind(token.status.to_string())
    .bind(token.created_at.timestamp())
    .bind(token.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError> {
    use crate::IdClaims;
    use sha2::{Digest, Sha256};

    let claims =
      extract_claims::<IdClaims>(token).map_err(|e| DbError::TokenValidation(e.to_string()))?;

    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let token_hash = token_hash[..12].to_string();

    let now = self.time_service.utc_now();
    let id = Uuid::new_v4().to_string();

    let api_token = ApiToken {
      id,
      user_id: claims.sub,
      name: name.to_string(),
      token_id: claims.jti,
      token_hash,
      status: TokenStatus::Active,
      created_at: now,
      updated_at: now,
    };

    sqlx::query(
      r#"
      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&api_token.id)
    .bind(&api_token.user_id)
    .bind(&api_token.name)
    .bind(&api_token.token_id)
    .bind(&api_token.token_hash)
    .bind(api_token.status.to_string())
    .bind(api_token.created_at.timestamp())
    .bind(api_token.updated_at.timestamp())
    .execute(&self.pool)
    .await?;

    Ok(api_token)
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
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      r#"
      SELECT
        id,
        user_id,
        name,
        token_id,
        token_hash,
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
        |(id, user_id, name, token_id, token_hash, status, created_at, updated_at)| {
          let Ok(status) = TokenStatus::from_str(&status) else {
            tracing::warn!("unknown token status: {} for id: {}", status, id);
            return None;
          };

          Some(ApiToken {
            id,
            user_id,
            name,
            token_id,
            token_hash,
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
        token_id,
        token_hash,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE user_id = ? AND id = ?
      "#;
    self.get_by_col(query, user_id, id).await
  }

  async fn get_api_token_by_token_id(&self, token: &str) -> Result<Option<ApiToken>, DbError> {
    use crate::IdClaims;
    use sha2::{Digest, Sha256};
    let claims =
      extract_claims::<IdClaims>(token).map_err(|e| DbError::TokenValidation(e.to_string()))?;
    let query = r#"
      SELECT
        id,
        user_id,
        name,
        token_id,
        token_hash,
        status,
        created_at,
        updated_at
      FROM api_tokens
      WHERE user_id = ? AND token_id = ?
      "#;
    let api_token = self.get_by_col(query, &claims.sub, &claims.jti).await?;
    match api_token {
      None => Ok(None),
      Some(api_token) => {
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let token_hash = token_hash[..12].to_string();
        if api_token.token_hash == token_hash {
          Ok(Some(api_token))
        } else {
          Ok(None)
        }
      }
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
    alias: &ApiModelAlias,
    api_key: &str,
  ) -> Result<(), DbError> {
    let (encrypted_api_key, salt, nonce) = encrypt_api_key(&self.encryption_key, api_key)
      .map_err(|e| DbError::EncryptionError(e.to_string()))?;

    let models_json = serde_json::to_string(&alias.models)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models: {}", e)))?;

    sqlx::query(
      r#"
      INSERT INTO api_model_aliases (id, provider, base_url, models_json, encrypted_api_key, salt, nonce, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#
    )
    .bind(&alias.id)
    .bind(&alias.provider)
    .bind(&alias.base_url)
    .bind(&models_json)
    .bind(&encrypted_api_key)
    .bind(&salt)
    .bind(&nonce)
    .bind(alias.created_at.timestamp())
    .bind(alias.created_at.timestamp()) // updated_at = created_at initially
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiModelAlias>, DbError> {
    let result = query_as::<_, (String, String, String, String, i64)>(
      "SELECT id, provider, base_url, models_json, created_at FROM api_model_aliases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((id, provider, base_url, models_json, created_at)) => {
        let models: Vec<String> = serde_json::from_str(&models_json)
          .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

        let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();

        Ok(Some(ApiModelAlias {
          id,
          source: AliasSource::RemoteApi,
          provider,
          base_url,
          models,
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
    model: &ApiModelAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    let models_json = serde_json::to_string(&model.models)
      .map_err(|e| DbError::EncryptionError(format!("Failed to serialize models: {}", e)))?;

    let now = self.time_service.utc_now();

    if let Some(api_key) = api_key {
      // Update with new API key
      let (encrypted_api_key, salt, nonce) = encrypt_api_key(&self.encryption_key, &api_key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;

      sqlx::query(
        r#"
        UPDATE api_model_aliases 
        SET provider = ?, base_url = ?, models_json = ?, encrypted_api_key = ?, salt = ?, nonce = ?, updated_at = ?
        WHERE id = ?
        "#
      )
      .bind(&model.provider)
      .bind(&model.base_url)
      .bind(&models_json)
      .bind(&encrypted_api_key)
      .bind(&salt)
      .bind(&nonce)
      .bind(now.timestamp())
      .bind(id)
      .execute(&self.pool)
      .await?;
    } else {
      // Update without changing API key
      sqlx::query(
        r#"
        UPDATE api_model_aliases 
        SET provider = ?, base_url = ?, models_json = ?, updated_at = ?
        WHERE id = ?
        "#,
      )
      .bind(&model.provider)
      .bind(&model.base_url)
      .bind(&models_json)
      .bind(now.timestamp())
      .bind(id)
      .execute(&self.pool)
      .await?;
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

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiModelAlias>, DbError> {
    let results = query_as::<_, (String, String, String, String, i64)>(
      "SELECT id, provider, base_url, models_json, created_at FROM api_model_aliases ORDER BY created_at DESC"
    )
    .fetch_all(&self.pool)
    .await?;

    let mut aliases = Vec::new();
    for (id, provider, base_url, models_json, created_at) in results {
      let models: Vec<String> = serde_json::from_str(&models_json)
        .map_err(|e| DbError::EncryptionError(format!("Failed to deserialize models: {}", e)))?;

      let created_at = chrono::DateTime::<Utc>::from_timestamp(created_at, 0).unwrap_or_default();

      aliases.push(ApiModelAlias {
        id,
        source: AliasSource::RemoteApi,
        provider,
        base_url,
        models,
        created_at,
        updated_at: created_at,
      });
    }

    Ok(aliases)
  }

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError> {
    let result = query_as::<_, (String, String, String)>(
      "SELECT encrypted_api_key, salt, nonce FROM api_model_aliases WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((encrypted_api_key, salt, nonce)) => {
        let api_key = decrypt_api_key(&self.encryption_key, &encrypted_api_key, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some(api_key))
      }
      None => Ok(None),
    }
  }

  fn now(&self) -> DateTime<Utc> {
    self.time_service.utc_now()
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::{
      AccessRequest, ApiToken, DbError, DbService, DownloadRequest, DownloadStatus, RequestStatus,
      SqlxError, TokenStatus,
    },
    test_utils::{build_token, test_db_service, TestDbService},
  };
  use chrono::Utc;
  use objs::{AliasSource, ApiModelAlias};
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
    let email = "test@example.com".to_string();
    let pending_request = service.insert_pending_request(email.clone()).await?;
    let expected_request = AccessRequest {
      id: pending_request.id, // We don't know this in advance
      email,
      created_at: now,
      updated_at: now,
      status: RequestStatus::Pending,
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
    let email = "test@example.com".to_string();
    let inserted_request = service.insert_pending_request(email.clone()).await?;
    let fetched_request = service.get_pending_request(email).await?;
    assert!(fetched_request.is_some());
    assert_eq!(fetched_request.unwrap(), inserted_request);
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
    let emails = vec![
      "test1@example.com".to_string(),
      "test2@example.com".to_string(),
      "test3@example.com".to_string(),
    ];
    for email in &emails {
      service.insert_pending_request(email.clone()).await?;
    }
    let page1 = service.list_pending_requests(1, 2).await?;
    assert_eq!(2, page1.len());
    let page2 = service.list_pending_requests(2, 2).await?;
    assert_eq!(1, page2.len());
    for (i, request) in page1.iter().chain(page2.iter()).enumerate() {
      let expected_request = AccessRequest {
        id: request.id,
        email: emails[i].clone(),
        created_at: now,
        updated_at: now,
        status: RequestStatus::Pending,
      };
      assert_eq!(request, &expected_request);
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
    let email = "test@example.com".to_string();
    let inserted_request = service.insert_pending_request(email.clone()).await?;
    service
      .update_request_status(inserted_request.id, RequestStatus::Approved)
      .await?;
    let updated_request = service.get_pending_request(email).await?;
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
      token_id: Uuid::new_v4().to_string(),
      token_hash: "token_hash".to_string(),
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
      token_id: "token123".to_string(),
      token_hash: "token_hash".to_string(),
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
    assert_eq!(updated.token_id, token.token_id);
    assert_eq!(updated.created_at, token.created_at);
    assert!(updated.updated_at >= token.updated_at);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_api_token_from(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    // Create a test token with known claims
    let test_jti = Uuid::new_v4().to_string();
    let test_sub = Uuid::new_v4().to_string();
    let (token, _) = build_token(serde_json::json!({
      "jti": test_jti,
      "sub": test_sub,
    }))?;

    // Create API token
    let name = "Test Token";
    let api_token = db_service.create_api_token_from(name, &token).await?;

    // Verify the created token
    assert_eq!(api_token.name, name);
    assert_eq!(api_token.token_id, test_jti);
    assert_eq!(api_token.user_id, test_sub);
    assert_eq!(api_token.status, TokenStatus::Active);

    // Verify we can retrieve it
    let retrieved = db_service
      .get_api_token_by_id(&test_sub, &api_token.id)
      .await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.token_id, test_jti);
    assert_eq!(retrieved.user_id, test_sub);
    assert_eq!(retrieved.token_hash, api_token.token_hash);

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
      token_id: Uuid::new_v4().to_string(),
      token_hash: "hash1".to_string(),
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
      token_id: Uuid::new_v4().to_string(),
      token_hash: "hash2".to_string(),
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
      token_id: Uuid::new_v4().to_string(),
      token_hash: "hash".to_string(),
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
    let alias_obj = ApiModelAlias::new(
      "openai",
      AliasSource::RemoteApi,
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      now,
    );
    let api_key = "sk-test123456789";

    // Create API model alias
    service.create_api_model_alias(&alias_obj, api_key).await?;

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
  async fn test_update_api_model_alias_with_new_key(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
  ) -> anyhow::Result<()> {
    let now = service.now();
    let mut alias_obj = ApiModelAlias::new(
      "claude",
      AliasSource::RemoteApi,
      "anthropic",
      "https://api.anthropic.com/v1",
      vec!["claude-3".to_string()],
      now,
    );
    let original_api_key = "sk-original123";
    let new_api_key = "sk-updated456";

    // Create initial alias
    service
      .create_api_model_alias(&alias_obj, original_api_key)
      .await?;

    // Update with new API key and additional model
    alias_obj.models.push("claude-3.5".to_string());
    service
      .update_api_model_alias("claude", &alias_obj, Some(new_api_key.to_string()))
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
    let mut alias_obj = ApiModelAlias::new(
      "gemini",
      AliasSource::RemoteApi,
      "google",
      "https://generativelanguage.googleapis.com/v1",
      vec!["gemini-pro".to_string()],
      now,
    );
    let api_key = "AIzaSy-test123";

    // Create initial alias
    service.create_api_model_alias(&alias_obj, api_key).await?;

    // Update without changing API key
    alias_obj.base_url = "https://generativelanguage.googleapis.com/v1beta".to_string();
    service
      .update_api_model_alias("gemini", &alias_obj, None)
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
      (
        "alias1",
        "provider1",
        "key1",
        now - chrono::Duration::seconds(20),
      ),
      (
        "alias2",
        "provider2",
        "key2",
        now - chrono::Duration::seconds(10),
      ),
      ("alias3", "provider3", "key3", now),
    ];

    for (alias, provider, key, created_at) in &aliases {
      let alias_obj = ApiModelAlias::new(
        *alias,
        AliasSource::RemoteApi,
        *provider,
        "https://api.example.com/v1",
        vec!["model1".to_string()],
        *created_at,
      );
      service.create_api_model_alias(&alias_obj, key).await?;
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
    let alias_obj = ApiModelAlias::new(
      "to-delete",
      AliasSource::RemoteApi,
      "test-provider",
      "https://api.test.com/v1",
      vec!["test-model".to_string()],
      now,
    );

    // Create and verify exists
    service
      .create_api_model_alias(&alias_obj, "test-key")
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
    let alias_obj = ApiModelAlias::new(
      "security-test",
      AliasSource::RemoteApi,
      "secure-provider",
      "https://api.secure.com/v1",
      vec!["secure-model".to_string()],
      now,
    );
    let sensitive_key = "sk-very-secret-key-12345";

    // Store API key
    service
      .create_api_model_alias(&alias_obj, sensitive_key)
      .await?;

    // Verify different encryptions produce different results
    let alias_obj2 = ApiModelAlias::new(
      "security-test2",
      AliasSource::RemoteApi,
      "secure-provider",
      "https://api.secure.com/v1",
      vec!["secure-model".to_string()],
      now,
    );
    service
      .create_api_model_alias(&alias_obj2, sensitive_key)
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
}
