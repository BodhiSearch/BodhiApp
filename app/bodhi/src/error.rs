use commands::{
  AliasCommandError, CreateCommandError, EnvCommandError, ListCommandError, PullCommandError,
};
use objs::BuilderError;
use server::{ContextError, RunCommandError};
use services::{db::DbError, DataServiceError, SessionServiceError};
use std::io;

use crate::convert::ConvertError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{0}")]
  Unreachable(String),
  #[error(transparent)]
  BodhiError(#[from] server::BodhiError),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
  #[error(transparent)]
  Db(#[from] DbError),
  #[error(transparent)]
  BuilderError(#[from] BuilderError),
  #[error(transparent)]
  SessionServiceError(#[from] SessionServiceError),
  #[error(transparent)]
  AliasCommandError(#[from] AliasCommandError),
  #[error(transparent)]
  PullCommandError(#[from] PullCommandError),
  #[error(transparent)]
  RunCommandError(#[from] RunCommandError),
  #[error(transparent)]
  CreateCommandError(#[from] CreateCommandError),
  #[error(transparent)]
  ListCommandError(#[from] ListCommandError),
  #[error(transparent)]
  EnvCommandError(#[from] EnvCommandError),
  #[error(transparent)]
  ConvertError(#[from] ConvertError),
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;
