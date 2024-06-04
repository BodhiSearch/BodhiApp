use crate::objs::ObjError;
use hf_hub::api::sync::ApiError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataServiceError {
  #[error(transparent)]
  ApiError(#[from] ApiError),
  #[error(
    r#"{source}
huggingface repo '{repo}' is requires requesting for access from website.
Go to https://huggingface.co/{repo} to request access to the model and try again.
"#
  )]
  GatedAccess {
    #[source]
    source: ApiError,
    repo: String,
  },
  #[error(
    r#"{source}
You are not logged in to huggingface using CLI `huggingface-cli login`.
So either the huggingface repo '{repo}' does not exists, or is private, or requires request access.
Go to https://huggingface.co/{repo} to request access, login via CLI, and then try again.
"#
  )]
  MayBeNotExists {
    #[source]
    source: ApiError,
    repo: String,
  },
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error("{source}\nerror while serializing from file: '{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error(
    r#"directory '{dirname}' not found in $BODHI_HOME.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#
  )]
  DirMissing { dirname: String },
  #[error(
    r#"file '{filename}' not found in $BODHI_HOME/{dirname}.
$BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME."#
  )]
  FileMissing { filename: String, dirname: String },
  #[error("only files from refs/main supported")]
  OnlyRefsMainSupported,
  #[error(transparent)]
  ObjError(#[from] ObjError),
}

pub type Result<T> = std::result::Result<T, DataServiceError>;
