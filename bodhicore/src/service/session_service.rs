use sqlx::{Pool, Sqlite};
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Debug, thiserror::Error)]
pub enum SessionServiceError {
  #[error(transparent)]
  SqlxError(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, SessionServiceError>;

#[cfg_attr(test, mockall::automock)]
pub trait SessionService: std::fmt::Debug {
  fn session_layer(&self) -> SessionManagerLayer<SqliteStore>;
}

#[derive(Debug)]
pub struct SqliteSessionService {
  session_store: SqliteStore,
}

impl SqliteSessionService {
  pub fn new(pool: Pool<Sqlite>) -> Self {
    let session_store = SqliteStore::new(pool);
    Self { session_store }
  }

  pub async fn migrate(&self) -> Result<()> {
    self.session_store.migrate().await?;
    Ok(())
  }
}

impl SessionService for SqliteSessionService {
  fn session_layer(&self) -> SessionManagerLayer<SqliteStore> {
    SessionManagerLayer::new(self.session_store.clone())
      .with_secure(true)
      .with_name("bodhiapp_session_id")
  }
}
