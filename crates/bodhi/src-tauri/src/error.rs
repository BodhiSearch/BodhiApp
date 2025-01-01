use commands::{
  AliasCommandError, CreateCommandError, EnvCommandError, ListCommandError, PullCommandError,
};
use objs::{impl_error_from, AppError, BuilderError, ErrorType, IoError, LocalizationSetupError};
use server_app::{RunCommandError, ServeError};
use server_core::ContextError;
use services::{
  db::DbError, DataServiceError, KeyringError, SecretServiceError, SessionServiceError,
};

use crate::convert::ConvertError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum BodhiError {
  #[error("unreachable")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  Unreachable(String),
  #[error("native_not_supported")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  NativeNotSupported,
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  Io(#[from] IoError),
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
  SecretService(#[from] SecretServiceError),
  #[error(transparent)]
  KeyringError(#[from] KeyringError),
  #[error(transparent)]
  Serve(#[from] ServeError),
  #[cfg(feature = "native")]
  #[error(transparent)]
  Native(#[from] crate::native::NativeError),
}

impl_error_from!(::std::io::Error, BodhiError::Io, objs::IoError);

pub(crate) type Result<T> = std::result::Result<T, BodhiError>;
