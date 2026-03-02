use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingsRouteError {
  #[error("Setting '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("BODHI_HOME can only be changed via environment variable.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  BodhiHome,

  #[error("Updating setting '{0}' is not supported yet.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Unsupported(String),
}
