use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use jsonwebtoken::{DecodingKey, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use services::db::DbError;
use thiserror::Error;

// TODO - have internal log message, and external user message
#[derive(Debug, Error)]
pub enum ApiError {
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

pub fn generate_random_string(length: usize) -> String {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let idx = rng.gen_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub jti: String,
  pub exp: u64,
  pub email: String,
}

pub fn decode_access_token(
  access_token: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
  let mut validation = Validation::default();
  validation.insecure_disable_signature_validation();
  validation.validate_exp = false;
  let token_data = jsonwebtoken::decode::<Claims>(
    access_token,
    &DecodingKey::from_secret(&[]), // dummy key for parsing
    &validation,
  )?;
  Ok(token_data)
}