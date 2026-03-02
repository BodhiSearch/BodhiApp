use crate::auth::AuthContextError;
use crate::db::DbError;
use crate::{EntityError, ErrorType};
use errmeta_derive::ErrorMeta;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = crate::AppError)]
pub enum TokenServiceError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  Auth(#[from] AuthContextError),

  #[error(transparent)]
  Entity(#[from] EntityError),
}
