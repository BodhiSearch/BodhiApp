use crate::{oai::OpenAIApiError, shared_rw::ContextError};
use async_openai::error::OpenAIError;
use objs::{Common, ObjError};
use services::{db::DbError, AuthServiceError, DataServiceError, HubServiceError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BodhiError {
  #[error(
    r#"model alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error("model alias '{0}' already exists. Use --force to overwrite the model alias config")]
  AliasExists(String),
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
  #[error(transparent)]
  OAuthError(#[from] AuthServiceError),
  #[error(transparent)]
  Db(#[from] DbError),
}

pub type Result<T> = std::result::Result<T, BodhiError>;
