use crate::{
  objs::ObjError,
  service::{DataServiceError, HubServiceError},
  Command,
};
use std::io;
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
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum Common {
  #[error("io_error: {source}\npath='{path}'")]
  Io {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error("serde_yaml_serialize: {source}\nfilename='{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
}
