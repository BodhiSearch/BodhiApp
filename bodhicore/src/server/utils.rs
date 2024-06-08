use crate::{
  db::DbError,
  error::{BodhiError, Common},
};
use axum::{
  body::Body,
  http::{header::CONTENT_TYPE, request::Builder, Request, StatusCode},
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

pub static DEFAULT_PORT: u16 = 1135;
// TODO: see if can use lazy_static to not duplicate port
pub static DEFAULT_PORT_STR: &str = "1135";
pub static DEFAULT_HOST: &str = "127.0.0.1";
pub static BODHI_HOME: &str = "BODHI_HOME";
pub static HF_HOME: &str = "HF_HOME";

pub trait AxumRequestExt {
  #[allow(clippy::result_large_err)]
  fn json<T: serde::Serialize>(self, value: T) -> Result<Request<Body>, BodhiError>;
}

impl AxumRequestExt for Builder {
  fn json<T: serde::Serialize>(self, value: T) -> std::result::Result<Request<Body>, BodhiError> {
    let this = self.header(CONTENT_TYPE, "application/json");
    let content = serde_json::to_string(&value).map_err(Common::SerdeJsonDeserialize)?;
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
  #[error(transparent)]
  Axum(#[from] axum::http::Error),
}

impl From<DbError> for ApiError {
  fn from(value: DbError) -> Self {
    match value {
      DbError::Sqlx { source, table } => match source {
        sqlx::Error::RowNotFound => {
          ApiError::NotFound(format!("given record not found in {}", table))
        }
        err => ApiError::ServerError(err.to_string()),
      },
      DbError::SqlxConnect { source, url } => ApiError::ServerError(format!(
        "not able to connect to database at {url}, error: {source}",
      )),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorResponse {
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
      ApiError::Axum(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorResponse {
          error: err.to_string(),
        }),
      )
        .into_response(),
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
