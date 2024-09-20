use crate::BodhiError;
use axum::{
  body::Body,
  http::{header::CONTENT_TYPE, request::Builder, Request, StatusCode},
  response::{IntoResponse, Response},
  Json,
};
use objs::Common;
use rand::Rng;
use serde::{Deserialize, Serialize};
use services::db::DbError;
use thiserror::Error;

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
      DbError::Migrate(err) => ApiError::ServerError(err.to_string()),
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

pub(crate) fn generate_random_string(length: usize) -> String {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let idx = rng.gen_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}
