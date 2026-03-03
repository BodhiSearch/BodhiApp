use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestAuthError {
  #[error("Authentication required.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,

  #[error("Entity not found in request path.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  EntityNotFound,

  #[error("Access request {access_request_id} not found.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotFound { access_request_id: String },

  #[error("Access request {access_request_id} has status '{status}'. Only approved requests can access resources.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotApproved {
    access_request_id: String,
    status: String,
  },

  #[error("Access request {access_request_id} is invalid: {reason}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestInvalid {
    access_request_id: String,
    reason: String,
  },

  #[error("Entity {entity_id} is not included in your approved resources for this app.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  EntityNotApproved { entity_id: String },

  #[error("Access request app client ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AppClientMismatch { expected: String, found: String },

  #[error("Access request user ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  UserMismatch { expected: String, found: String },

  #[error("Invalid approved JSON in access request: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidApprovedJson { error: String },

  #[error(transparent)]
  DbError(#[from] services::DbError),
}
