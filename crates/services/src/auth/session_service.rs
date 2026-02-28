use super::{SessionResult, SessionServiceError, SessionStoreBackend};
use async_trait::async_trait;
use cookie::SameSite;
use sqlx::{
  any::AnyRow,
  sqlite::{SqliteConnectOptions, SqlitePoolOptions},
  AnyPool, PgPool, Row,
};
use std::{fmt, str::FromStr};
use tower_sessions::SessionManagerLayer;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait SessionService: Send + Sync + std::fmt::Debug {
  fn session_layer(&self) -> SessionManagerLayer<SessionStoreBackend>;
  async fn clear_sessions_for_user(&self, user_id: &str) -> SessionResult<usize>;
  async fn clear_all_sessions(&self) -> SessionResult<usize>;
  async fn count_sessions_for_user(&self, user_id: &str) -> SessionResult<i32>;
  fn get_session_store(&self) -> &SessionStoreBackend;
}

#[async_trait]
pub trait AppSessionStoreExt: Send + Sync {
  async fn migrate_custom(&self, url: &str) -> SessionResult<()>;
  async fn clear_sessions_for_user(&self, user_id: &str) -> SessionResult<usize>;
  async fn clear_all_sessions(&self) -> SessionResult<usize>;
  async fn count_sessions_for_user(&self, user_id: &str) -> SessionResult<i32>;
  async fn get_session_ids_for_user(&self, user_id: &str) -> SessionResult<Vec<String>>;
  async fn dump_all_sessions(&self) -> SessionResult<Vec<(String, Option<String>)>>;
}

pub struct DefaultSessionService {
  store_backend: SessionStoreBackend,
  url: String,
}

impl fmt::Debug for DefaultSessionService {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DefaultSessionService")
      .field("backend", &self.store_backend)
      .finish()
  }
}

fn map_sqlx_err(e: sqlx::Error) -> SessionServiceError {
  SessionServiceError::SqlxError(crate::db::SqlxError::new(e))
}

impl DefaultSessionService {
  pub fn new(store_backend: SessionStoreBackend, url: String) -> Self {
    Self { store_backend, url }
  }

  pub async fn connect_sqlite(url: &str) -> std::result::Result<Self, SessionServiceError> {
    sqlx::any::install_default_drivers();
    let opts = SqliteConnectOptions::from_str(url)
      .map_err(map_sqlx_err)?
      .create_if_missing(true);
    let sqlite_pool = SqlitePoolOptions::new()
      .connect_with(opts)
      .await
      .map_err(map_sqlx_err)?;
    let any_url = if url.contains('?') {
      format!("{url}&mode=rwc")
    } else {
      format!("{url}?mode=rwc")
    };
    let any_pool = AnyPool::connect(&any_url).await.map_err(map_sqlx_err)?;
    let store = super::sqlite::create_sqlite_store(sqlite_pool);
    store.migrate().await?;
    let backend = SessionStoreBackend::new_sqlite(store, any_pool);
    let mut service = Self::new(backend, url.to_string());
    service.run_custom_migration().await?;
    Ok(service)
  }

  pub async fn connect_postgres(url: &str) -> std::result::Result<Self, SessionServiceError> {
    sqlx::any::install_default_drivers();
    let pg_pool = PgPool::connect(url).await.map_err(map_sqlx_err)?;
    let any_pool = AnyPool::connect(url).await.map_err(map_sqlx_err)?;
    let store = super::postgres::create_postgres_store(pg_pool)?;
    store.migrate().await?;
    let backend = SessionStoreBackend::new_postgres(store, any_pool);
    let mut service = Self::new(backend, url.to_string());
    service.run_custom_migration().await?;
    Ok(service)
  }

  pub async fn connect(url: &str) -> std::result::Result<Self, SessionServiceError> {
    if super::session_store::is_postgres_url(url) {
      Self::connect_postgres(url).await
    } else {
      Self::connect_sqlite(url).await
    }
  }

  async fn run_custom_migration(&mut self) -> SessionResult<()> {
    AppSessionStoreExt::migrate_custom(self, &self.url.clone()).await
  }
}

#[async_trait]
impl AppSessionStoreExt for DefaultSessionService {
  async fn migrate_custom(&self, url: &str) -> SessionResult<()> {
    let pool = self.store_backend.any_pool();
    if super::session_store::is_postgres_url(url) {
      sqlx::query("ALTER TABLE tower_sessions ADD COLUMN IF NOT EXISTS user_id TEXT")
        .execute(pool)
        .await?;
    } else {
      let row: AnyRow = sqlx::query(
        "SELECT COUNT(*) as cnt FROM pragma_table_info('tower_sessions') WHERE name = 'user_id'",
      )
      .fetch_one(pool)
      .await?;
      let column_exists: i32 = row.get("cnt");

      if column_exists == 0 {
        sqlx::query("ALTER TABLE tower_sessions ADD COLUMN user_id TEXT")
          .execute(pool)
          .await?;
      }
    }

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tower_sessions_user_id ON tower_sessions(user_id)")
      .execute(pool)
      .await?;

    Ok(())
  }

  async fn clear_sessions_for_user(&self, user_id: &str) -> SessionResult<usize> {
    // Note: $1 positional parameters work with SQLite via AnyPool because
    // sqlx's AnyPool translates PostgreSQL-style positional params to ?-style for SQLite.
    // Parameterized tests (test_session_service.rs) verify both backends.
    let result = sqlx::query("DELETE FROM tower_sessions WHERE user_id = $1")
      .bind(user_id)
      .execute(self.store_backend.any_pool())
      .await?;
    Ok(result.rows_affected() as usize)
  }

  async fn clear_all_sessions(&self) -> SessionResult<usize> {
    let result = sqlx::query("DELETE FROM tower_sessions")
      .execute(self.store_backend.any_pool())
      .await?;
    Ok(result.rows_affected() as usize)
  }

  async fn count_sessions_for_user(&self, user_id: &str) -> SessionResult<i32> {
    let row: AnyRow = sqlx::query("SELECT COUNT(*) as cnt FROM tower_sessions WHERE user_id = $1")
      .bind(user_id)
      .fetch_one(self.store_backend.any_pool())
      .await?;
    let count: i32 = row.get("cnt");
    Ok(count)
  }

  async fn get_session_ids_for_user(&self, user_id: &str) -> SessionResult<Vec<String>> {
    let rows: Vec<AnyRow> = sqlx::query("SELECT id FROM tower_sessions WHERE user_id = $1")
      .bind(user_id)
      .fetch_all(self.store_backend.any_pool())
      .await?;
    Ok(rows.iter().map(|r| r.get::<String, _>("id")).collect())
  }

  async fn dump_all_sessions(&self) -> SessionResult<Vec<(String, Option<String>)>> {
    let rows: Vec<AnyRow> = sqlx::query("SELECT id, user_id FROM tower_sessions")
      .fetch_all(self.store_backend.any_pool())
      .await?;
    Ok(
      rows
        .iter()
        .map(|r| {
          let id: String = r.get("id");
          let user_id: Option<String> = r.get("user_id");
          (id, user_id)
        })
        .collect(),
    )
  }
}

#[async_trait]
impl SessionService for DefaultSessionService {
  fn session_layer(&self) -> SessionManagerLayer<SessionStoreBackend> {
    SessionManagerLayer::new(self.store_backend.clone())
      .with_secure(false)
      .with_same_site(SameSite::Strict)
      .with_name("bodhiapp_session_id")
  }

  async fn clear_sessions_for_user(&self, user_id: &str) -> SessionResult<usize> {
    AppSessionStoreExt::clear_sessions_for_user(self, user_id).await
  }

  async fn clear_all_sessions(&self) -> SessionResult<usize> {
    AppSessionStoreExt::clear_all_sessions(self).await
  }

  async fn count_sessions_for_user(&self, user_id: &str) -> SessionResult<i32> {
    AppSessionStoreExt::count_sessions_for_user(self, user_id).await
  }

  fn get_session_store(&self) -> &SessionStoreBackend {
    &self.store_backend
  }
}

#[cfg(test)]
#[path = "test_session_service.rs"]
mod test_session_service;
