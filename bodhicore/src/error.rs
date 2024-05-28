use thiserror::Error;

use crate::service::DataServiceError;

#[derive(Debug, Error)]
pub enum AppError {
  #[error(
    r#"alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error("alias '{0}' already exists. Use --force to overwrite the alias config")]
  AliasExists(String),
  #[error("{0}")]
  BadRequest(String),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
}

pub type Result<T> = std::result::Result<T, AppError>;
