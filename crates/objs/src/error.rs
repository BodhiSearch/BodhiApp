use crate::builder::BuilderError;
use llama_server_bindings::GptParamsBuilderError;
use std::{io, path::PathBuf, sync::Arc};
use thiserror::Error;
use tokio::task::JoinError;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug, Error)]
pub enum ObjError {
  #[error("Validation failed: {0}")]
  Validation(#[from] ValidationErrors),

  #[error("Cannot convert '{from}' to '{to}'. Error: {error}")]
  Conversion {
    from: String,
    to: String,
    error: String,
  },

  #[error("IO error occurred while accessing '{path}': {source}")]
  IoWithDetail {
    #[source]
    source: io::Error,
    path: PathBuf,
  },

  #[error("JSON serialization/deserialization error: {0}")]
  SerdeJson(#[from] serde_json::Error),

  #[error("Builder error: {0}")]
  Builder(#[from] BuilderError),

  #[error("GPT parameters builder error: {0}")]
  GptBuilder(#[from] GptParamsBuilderError),
}

#[allow(unused)]
pub type Result<T> = std::result::Result<T, ObjError>;

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}

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
  #[error("sender_err: error sending signal using channel for '{0}'")]
  Sender(String),
  #[error(transparent)]
  Join(JoinError),
}
