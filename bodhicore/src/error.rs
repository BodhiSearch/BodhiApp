use crate::{service::DataServiceError, Command};
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum AppError {
  #[error(
    r#"model alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error("model alias '{0}' already exists. Use --force to overwrite the model alias config")]
  AliasExists(String),
  #[error("{0}")]
  BadRequest(String),
  #[error("Command '{0}' cannot be converted into command '{1}'")]
  ConvertCommand(Command, String),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(
    r#"model files for model alias '{alias}' not found in huggingface cache directory. Check if file in the expected filepath exists.
filepath: {filepath}
"#
  )]
  AliasModelFilesNotFound { alias: String, filepath: String },
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),
  #[error(transparent)]
  Minijina(#[from] minijinja::Error),
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
  // #[error(transparent)]
  // ContextError(#[from] ContextError),
}

pub type Result<T> = std::result::Result<T, AppError>;
