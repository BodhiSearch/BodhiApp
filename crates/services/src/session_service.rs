use crate::db::SqlxError;
use cookie::SameSite;
use objs::{impl_error_from, AppError};
use sqlx::{Pool, Sqlite};
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SessionServiceError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
}

impl_error_from!(
  ::sqlx::Error,
  SessionServiceError::SqlxError,
  crate::db::SqlxError
);

type Result<T> = std::result::Result<T, SessionServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SessionService: Send + Sync + std::fmt::Debug {
  fn session_layer(&self) -> SessionManagerLayer<SqliteStore>;

  #[cfg(any(test, feature = "test-utils"))]
  fn get_session_store(&self) -> &SqliteStore;
}

#[derive(Debug)]
pub struct SqliteSessionService {
  pub session_store: SqliteStore,
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
      .with_secure(false) // TODO: change this when https is supported
      .with_same_site(SameSite::Strict) // The cookie is now only sent for same-site/origin requests, blocking cross-site XHR usage
      .with_name("bodhiapp_session_id")
  }

  #[cfg(any(test, feature = "test-utils"))]
  fn get_session_store(&self) -> &SqliteStore {
    &self.session_store
  }
}
