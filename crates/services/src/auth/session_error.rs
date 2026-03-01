use crate::db::SqlxError;
use errmeta::{impl_error_from, AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SessionServiceError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
  #[error("Session store error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SessionStoreError(tower_sessions::session_store::Error),
  #[error("Session DB setup error: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DbSetup(String),
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

pub type SessionResult<T> = std::result::Result<T, SessionServiceError>;
