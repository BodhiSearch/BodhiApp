use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiModelsRouteError {
  #[error("API model '{0}' not found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}
