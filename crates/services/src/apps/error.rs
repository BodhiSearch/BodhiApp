use crate::db::DbError;
use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppInstanceError {
  #[error("Application instance not found.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  NotFound,
  #[error(transparent)]
  Db(#[from] DbError),
}

pub(crate) type Result<T> = std::result::Result<T, AppInstanceError>;
