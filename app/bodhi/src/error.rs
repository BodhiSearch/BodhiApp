use bodhicore::{db::DbError, CliError};
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{0}")]
  Unreachable(String),
  #[error("{0}")]
  Any(String),
  #[error(transparent)]
  BodhiError(#[from] bodhicore::BodhiError),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  Cli(#[from] CliError),
  #[error(transparent)]
  Db(#[from] DbError),
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;
