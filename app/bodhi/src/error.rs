use bodhicore::{
  db::DbError,
  service::{DataServiceError, SessionServiceError},
  CliError, ContextError,
};
use objs::BuilderError;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{0}")]
  Unreachable(String),
  #[error(transparent)]
  BodhiError(#[from] bodhicore::BodhiError),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  Cli(#[from] CliError),
  #[error(transparent)]
  Db(#[from] DbError),
  #[error(transparent)]
  BuilderError(#[from] BuilderError),
  #[error(transparent)]
  SessionServiceError(#[from] SessionServiceError),
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;
