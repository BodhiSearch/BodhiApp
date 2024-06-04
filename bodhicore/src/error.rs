use crate::{
  oai::OpenAIApiError,
  objs::ObjError,
  service::{DataServiceError, HubServiceError},
  shared_rw::ContextError,
  Command,
};
use async_openai::error::OpenAIError;
use std::{io, sync::Arc};
use thiserror::Error;
use validator::ValidationErrors;

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

  #[error("$HOME directory not found, set home directory using $HOME")]
  HomeDirectory,

  #[error(transparent)]
  Common(#[from] Common),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  // TODO: replace when async-openai is internal crate
  #[error(transparent)]
  BuildError(#[from] OpenAIError),
  #[error(transparent)]
  OpenAIApiError(#[from] OpenAIApiError),
  #[error(transparent)]
  AxumHttp(#[from] axum::http::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum Common {
  #[error("io_file: {source}\npath='{path}'")]
  IoFile {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io_error_dir_create: {source}\npath='{path}'")]
  IoDir {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error("serde_yaml_serialize: {source}\nfilename='{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
  #[error("serde_json_serialize: {source}\nvalue: {value}")]
  SerdeJsonSerialize {
    #[source]
    source: serde_json::Error,
    value: String,
  },
  #[error("serde_json_deserialize: {0}")]
  SerdeJsonDeserialize(#[from] serde_json::Error),
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error("stderr: {0}")]
  Stdlib(#[from] Arc<dyn std::error::Error + Send + Sync>),
}
