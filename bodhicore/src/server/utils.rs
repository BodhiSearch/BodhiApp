use super::routes_ui::ChatError;
use axum::http::header::CONTENT_TYPE;
use axum::{
  body::Body,
  http::{request::Builder, Request, StatusCode},
  response::{IntoResponse, Response},
  Json,
};
use serde::Serialize;
use std::{env, fs, io, path::PathBuf};
use thiserror::Error;

pub static DEFAULT_PORT: u16 = 1135;
// TODO: see if can use lazy_static to not duplicate port
pub static DEFAULT_PORT_STR: &str = "1135";
pub static DEFAULT_HOST: &str = "127.0.0.1";
pub static BODHI_HOME: &str = "BODHI_HOME";

pub trait AxumRequestExt {
  fn json<T: serde::Serialize>(self, value: T) -> Result<Request<Body>, anyhow::Error>;
}

impl AxumRequestExt for Builder {
  fn json<T: serde::Serialize>(
    self,
    value: T,
  ) -> std::result::Result<Request<Body>, anyhow::Error> {
    let this = self.header(CONTENT_TYPE, "application/json");
    let content = serde_json::to_string(&value)?;
    let result = this.body(Body::from(content))?;
    Ok(result)
  }
}

// TODO - have internal log message, and external user message
#[derive(Debug, Error)]
pub(crate) enum ApiError {
  #[error("{0}")]
  ServerError(String),
  #[error("{0}")]
  NotFound(String),
}

#[derive(Serialize)]
pub(crate) struct ApiErrorResponse {
  error: String,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    match self {
      ApiError::ServerError(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorResponse { error }),
      )
        .into_response(),
      ApiError::NotFound(error) => {
        (StatusCode::NOT_FOUND, Json(ApiErrorResponse { error })).into_response()
      }
    }
  }
}

#[allow(unused)]
pub fn port_from_env_vars(port: Result<String, env::VarError>) -> u16 {
  match port {
    Ok(port) => match port.parse::<u16>() {
      Ok(port) => port,
      Err(err) => {
        tracing::debug!(
          err = ?err,
          port = port,
          default_port = DEFAULT_PORT,
          "error parsing port set in environment variable, using default port",
        );
        DEFAULT_PORT
      }
    },
    Err(err) => {
      tracing::debug!(
        err = ?err,
        default_port = DEFAULT_PORT,
        "error reading port from environment variable, using default port",
      );
      DEFAULT_PORT
    }
  }
}

#[derive(Debug, Error)]
pub(crate) enum HomeDirError {
  #[error("Failed to get user home directory")]
  HomeDirErr,
  #[error("Failed to create app home directory")]
  HomeDirCreateErr,
  #[error("Failed to get chats directory")]
  ChatDirErr,
  #[error("Chats directory is not a directory")]
  ChatNotDirErr,
  #[error("Failed to read directory")]
  ReadDirErr,
  #[error(transparent)]
  IOError(#[from] io::Error),
}

impl PartialEq for HomeDirError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (HomeDirError::HomeDirErr, HomeDirError::HomeDirErr) => true,
      (HomeDirError::HomeDirCreateErr, HomeDirError::HomeDirCreateErr) => true,
      (HomeDirError::ChatDirErr, HomeDirError::ChatDirErr) => true,
      (HomeDirError::ReadDirErr, HomeDirError::ReadDirErr) => true,
      (HomeDirError::IOError(e1), HomeDirError::IOError(e2)) => e1.kind() == e2.kind(),
      _ => false,
    }
  }
}

// TODO - change from exposing internal error message, to having internal log message and external user message
impl From<HomeDirError> for ApiError {
  fn from(value: HomeDirError) -> Self {
    ApiError::ServerError(format!("{value}"))
  }
}

impl From<ChatError> for ApiError {
  fn from(value: ChatError) -> Self {
    match value {
      ChatError::HomeDirError(err) => ApiError::ServerError(format!("{err}")),
      ChatError::ChatNotFoundErr(err) => ApiError::NotFound(err.to_string()),
      ChatError::IOError(err) => ApiError::ServerError(format!("{err}")),
      ChatError::JsonError(err) => ApiError::ServerError(format!("{err}")),
    }
  }
}

// TODO: use methods in home.rs
pub(crate) fn get_bodhi_home_dir() -> Result<PathBuf, HomeDirError> {
  if let Ok(bodhi_home) = env::var(BODHI_HOME) {
    let home_dir = PathBuf::from(bodhi_home);
    if home_dir.exists() {
      if home_dir.is_dir() {
        Ok(home_dir)
      } else {
        Err(HomeDirError::ChatNotDirErr)
      }
    } else {
      Err(HomeDirError::ChatDirErr)
    }
  } else {
    let home_dir = dirs::home_dir().ok_or(HomeDirError::HomeDirErr)?;
    let bodhi_home = home_dir.join("chats");
    if !bodhi_home.exists() {
      fs::create_dir_all(&bodhi_home).map_err(|_| HomeDirError::HomeDirCreateErr)?;
    }
    Ok(bodhi_home)
  }
}

// TODO: use methods in home.rs
pub(crate) fn get_chats_dir() -> Result<PathBuf, HomeDirError> {
  let bodhi_home = get_bodhi_home_dir()?;
  let chats_dir = bodhi_home.join("chats");
  if chats_dir.exists() {
    if chats_dir.is_dir() {
      Ok(chats_dir)
    } else {
      Err(HomeDirError::ChatNotDirErr)
    }
  } else {
    fs::create_dir_all(&chats_dir)?;
    Ok(chats_dir)
  }
}

#[cfg(test)]
mod tests {
  use super::{port_from_env_vars, DEFAULT_PORT};
  use rstest::rstest;

  #[test]
  pub fn test_port_from_env_vars_not_present() {
    let port = port_from_env_vars(Err(std::env::VarError::NotPresent));
    assert_eq!(port, DEFAULT_PORT);
  }

  #[test]
  pub fn test_port_from_env_vars_valid() {
    let port = port_from_env_vars(Ok("8055".to_string()));
    assert_eq!(port, 8055);
  }

  #[rstest]
  #[case("notu16")]
  #[case("65536")]
  #[case("-1")]
  pub fn test_port_from_env_vars_malformed(#[case] input: &str) {
    let port = port_from_env_vars(Ok(input.to_string()));
    assert_eq!(port, DEFAULT_PORT);
  }
}
