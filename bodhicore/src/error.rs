use crate::{service::DataServiceError, Command};
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
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
  #[error("Command '{0}' cannot be converted into command '{1}'")]
  ConvertCommand(Command, String),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(
    r#"alias model files not found in local huggingface hub cache directory.
Ensure repo: {repo}, contains GGUF model file: {filename}, snapshot: {snapshot}"#
  )]
  AliasModelNotFound {
    repo: String,
    filename: String,
    snapshot: String,
  },
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
