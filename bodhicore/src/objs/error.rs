use std::{io, path::PathBuf};

use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ObjError {
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error("cannot convert '{from}' to '{to}', error: \"{error}\"")]
  Conversion {
    from: String,
    to: String,
    error: String,
  },
  #[error("io error: {source}\npath: {path}")]
  IoWithDetail {
    #[source]
    source: io::Error,
    path: PathBuf,
  },
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ObjError>;
