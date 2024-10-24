use commands::{
  AliasCommandError, CreateCommandError, EnvCommandError, ListCommandError, PullCommandError,
};
use objs::{impl_error_from, AppError, BuilderError, ErrorType, IoError, LocalizationSetupError};
use server_app::{RunCommandError, ServeError};
use server_core::ContextError;
use services::{db::DbError, DataServiceError, SessionServiceError};

use crate::convert::ConvertError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum BodhiError {
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  Io(#[from] IoError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "tauri_error", args_delegate = false)]
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
  #[error(transparent)]
  LocalizationSetup(#[from] LocalizationSetupError),
  #[error(transparent)]
  Serve(#[from] ServeError),
}

impl_error_from!(::std::io::Error, BodhiError::Io, objs::IoError);

pub(crate) type Result<T> = std::result::Result<T, BodhiError>;
