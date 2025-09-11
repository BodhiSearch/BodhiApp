use crate::db::SqlxError;
use async_trait::async_trait;
use cookie::SameSite;
use objs::{impl_error_from, AppError, ErrorType};
use sqlx::{Pool, Sqlite};
use std::fmt;
use tower_sessions::{
  session::{Id, Record},
  SessionManagerLayer, SessionStore,
};
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SessionServiceError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
  #[error("Session store error: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SessionStoreError(tower_sessions::session_store::Error),
}

impl From<tower_sessions::session_store::Error> for SessionServiceError {
  fn from(error: tower_sessions::session_store::Error) -> Self {
    SessionServiceError::SessionStoreError(error)
  }
}

impl_error_from!(
  ::sqlx::Error,
  SessionServiceError::SqlxError,
  crate::db::SqlxError
);

type Result<T> = std::result::Result<T, SessionServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait SessionService: Send + Sync + std::fmt::Debug {
  fn session_layer(&self) -> SessionManagerLayer<AppSessionStore>;
  async fn clear_sessions_for_user(&self, user_id: &str) -> Result<usize>;
  fn get_session_store(&self) -> &AppSessionStore;
}

/// Custom SessionStore wrapper that adds user_id tracking to tower_sessions
#[derive(Clone)]
pub struct AppSessionStore {
  inner: SqliteStore,
  pool: Pool<Sqlite>,
}

impl fmt::Debug for AppSessionStore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AppSessionStore")
      .field("inner", &"SqliteStore")
      .finish()
  }
}

#[derive(Debug)]
pub struct SqliteSessionService {
  pub session_store: AppSessionStore,
}

impl AppSessionStore {
  pub fn new(pool: Pool<Sqlite>) -> Self {
    Self {
      inner: SqliteStore::new(pool.clone()),
      pool,
    }
  }

  pub async fn migrate(&self) -> Result<()> {
    // First run the standard tower_sessions migration
    self.inner.migrate().await?;

    // Check if user_id column exists, and add it if it doesn't
    let column_exists = sqlx::query_scalar::<_, i32>(
      "SELECT COUNT(*) FROM pragma_table_info('tower_sessions') WHERE name = 'user_id'",
    )
    .fetch_one(&self.pool)
    .await?
      > 0;

    if !column_exists {
      sqlx::query("ALTER TABLE tower_sessions ADD COLUMN user_id TEXT")
        .execute(&self.pool)
        .await?;
    }

    // Create index on user_id for efficient lookups
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tower_sessions_user_id ON tower_sessions(user_id)")
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  pub async fn clear_sessions_for_user(&self, user_id: &str) -> Result<usize> {
    let result = sqlx::query("DELETE FROM tower_sessions WHERE user_id = ?")
      .bind(user_id)
      .execute(&self.pool)
      .await?;

    Ok(result.rows_affected() as usize)
  }

  pub async fn count_sessions_for_user(&self, user_id: &str) -> Result<i32> {
    let count =
      sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM tower_sessions WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

    Ok(count)
  }

  pub async fn get_session_ids_for_user(&self, user_id: &str) -> Result<Vec<String>> {
    let session_ids =
      sqlx::query_scalar::<_, String>("SELECT id FROM tower_sessions WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

    Ok(session_ids)
  }

  pub async fn dump_all_sessions(&self) -> Result<Vec<(String, Option<String>)>> {
    let sessions =
      sqlx::query_as::<_, (String, Option<String>)>("SELECT id, user_id FROM tower_sessions")
        .fetch_all(&self.pool)
        .await?;

    Ok(sessions)
  }
}

impl SqliteSessionService {
  pub fn new(pool: Pool<Sqlite>) -> Self {
    let session_store = AppSessionStore::new(pool);
    Self { session_store }
  }

  pub async fn migrate(&self) -> Result<()> {
    self.session_store.migrate().await?;
    Ok(())
  }
}

#[async_trait]
impl SessionStore for AppSessionStore {
  async fn save(
    &self,
    record: &Record,
  ) -> std::result::Result<(), tower_sessions::session_store::Error> {
    // Extract user_id from session data if present
    let user_id = record.data.get("user_id").and_then(|v| v.as_str());

    // Save the record using the inner store first
    self.inner.save(record).await?;

    // Update the user_id column if we have a user_id
    if let Some(user_id) = user_id {
      sqlx::query("UPDATE tower_sessions SET user_id = ? WHERE id = ?")
        .bind(user_id)
        .bind(record.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| tower_sessions::session_store::Error::Backend(e.to_string()))?;
    }

    Ok(())
  }

  async fn load(
    &self,
    session_id: &Id,
  ) -> std::result::Result<Option<Record>, tower_sessions::session_store::Error> {
    self.inner.load(session_id).await
  }

  async fn delete(
    &self,
    session_id: &Id,
  ) -> std::result::Result<(), tower_sessions::session_store::Error> {
    self.inner.delete(session_id).await
  }
}

#[async_trait]
impl SessionService for SqliteSessionService {
  fn session_layer(&self) -> SessionManagerLayer<AppSessionStore> {
    SessionManagerLayer::new(self.session_store.clone())
      .with_secure(false) // TODO: change this when https is supported
      .with_same_site(SameSite::Strict) // The cookie is now only sent for same-site/origin requests, blocking cross-site XHR usage
      .with_name("bodhiapp_session_id")
  }

  async fn clear_sessions_for_user(&self, user_id: &str) -> Result<usize> {
    self.session_store.clear_sessions_for_user(user_id).await
  }

  fn get_session_store(&self) -> &AppSessionStore {
    &self.session_store
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sqlx::SqlitePool;
  use std::collections::HashMap;
  use tempfile::TempDir;
  use time::OffsetDateTime;
  use tower_sessions::session::{Id, Record};

  async fn create_test_session_service() -> (SqliteSessionService, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_sessions.sqlite");

    // Create the database file first
    std::fs::File::create(&db_path).unwrap();

    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
      .await
      .unwrap();
    let service = SqliteSessionService::new(pool);
    service.migrate().await.unwrap();
    (service, temp_dir)
  }

  #[tokio::test]
  async fn test_app_session_store_migration() {
    let (service, _temp_dir) = create_test_session_service().await;

    // Migration should succeed without error
    service.migrate().await.unwrap();

    // Check that user_id column exists
    let pool = &service.session_store.pool;
    let column_exists = sqlx::query_scalar::<_, i32>(
      "SELECT COUNT(*) FROM pragma_table_info('tower_sessions') WHERE name = 'user_id'",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(
      column_exists, 1,
      "user_id column should exist after migration"
    );

    // Check that index exists
    let index_exists = sqlx::query_scalar::<_, i32>(
      "SELECT COUNT(*) FROM pragma_index_list('tower_sessions') WHERE name = 'idx_tower_sessions_user_id'",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(
      index_exists, 1,
      "user_id index should exist after migration"
    );
  }

  #[tokio::test]
  async fn test_session_save_with_user_id() {
    let (service, _temp_dir) = create_test_session_service().await;
    let store = &service.session_store;

    // Create a session record with user_id
    let session_id = Id::default();
    let mut data = HashMap::new();
    data.insert(
      "user_id".to_string(),
      serde_json::Value::String("user123".to_string()),
    );
    data.insert(
      "test_key".to_string(),
      serde_json::Value::String("test_value".to_string()),
    );

    let record = Record {
      id: session_id.clone(),
      data,
      expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
    };

    // Save the record
    store.save(&record).await.unwrap();

    // Check that user_id was stored in the database
    let stored_user_id =
      sqlx::query_scalar::<_, Option<String>>("SELECT user_id FROM tower_sessions WHERE id = ?")
        .bind(session_id.to_string())
        .fetch_one(&store.pool)
        .await
        .unwrap();

    assert_eq!(stored_user_id, Some("user123".to_string()));
  }

  #[tokio::test]
  async fn test_session_save_without_user_id() {
    let (service, _temp_dir) = create_test_session_service().await;
    let store = &service.session_store;

    // Create a session record without user_id
    let session_id = Id::default();
    let mut data = HashMap::new();
    data.insert(
      "test_key".to_string(),
      serde_json::Value::String("test_value".to_string()),
    );

    let record = Record {
      id: session_id.clone(),
      data,
      expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
    };

    // Save the record
    store.save(&record).await.unwrap();

    // Check that user_id is null in the database
    let stored_user_id =
      sqlx::query_scalar::<_, Option<String>>("SELECT user_id FROM tower_sessions WHERE id = ?")
        .bind(session_id.to_string())
        .fetch_one(&store.pool)
        .await
        .unwrap();

    assert_eq!(stored_user_id, None);
  }

  #[tokio::test]
  async fn test_clear_sessions_for_user() {
    let (service, _temp_dir) = create_test_session_service().await;
    let store = &service.session_store;

    // Create multiple sessions for the same user
    let user_id = "user123";
    for i in 0..3 {
      let session_id = Id::default();
      let mut data = HashMap::new();
      data.insert(
        "user_id".to_string(),
        serde_json::Value::String(user_id.to_string()),
      );
      data.insert(
        "session_num".to_string(),
        serde_json::Value::Number(i.into()),
      );

      let record = Record {
        id: session_id,
        data,
        expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
      };

      store.save(&record).await.unwrap();
    }

    // Create a session for a different user
    let session_id = Id::default();
    let mut data = HashMap::new();
    data.insert(
      "user_id".to_string(),
      serde_json::Value::String("user456".to_string()),
    );
    let record = Record {
      id: session_id,
      data,
      expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
    };
    store.save(&record).await.unwrap();

    // Clear sessions for user123
    let cleared_count = service.clear_sessions_for_user(user_id).await.unwrap();
    assert_eq!(cleared_count, 3);

    // Check that user123's sessions are gone
    let remaining_user123_sessions =
      sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM tower_sessions WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&store.pool)
        .await
        .unwrap();
    assert_eq!(remaining_user123_sessions, 0);

    // Check that user456's session is still there
    let remaining_user456_sessions =
      sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM tower_sessions WHERE user_id = ?")
        .bind("user456")
        .fetch_one(&store.pool)
        .await
        .unwrap();
    assert_eq!(remaining_user456_sessions, 1);
  }

  #[tokio::test]
  async fn test_clear_sessions_for_nonexistent_user() {
    let (service, _temp_dir) = create_test_session_service().await;

    // Try to clear sessions for a user that doesn't exist
    let cleared_count = service
      .clear_sessions_for_user("nonexistent")
      .await
      .unwrap();
    assert_eq!(cleared_count, 0);
  }
}
