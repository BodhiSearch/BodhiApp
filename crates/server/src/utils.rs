use std::{str::FromStr, sync::Arc};

use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use jsonwebtoken::{DecodingKey, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use services::{db::DbError, AppStatus, SecretService, KEY_APP_STATUS};
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
      DbError::StrumParse(err) => ApiError::ServerError(err.to_string()),
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

pub fn app_status_or_default(secret_service: &Arc<dyn SecretService>) -> AppStatus {
  let value = secret_service.get_secret_string(KEY_APP_STATUS);
  match value {
    Ok(Some(value)) => AppStatus::from_str(&value).unwrap_or(AppStatus::default()),
    Ok(None) => AppStatus::default(),
    Err(_) => AppStatus::default(),
  }
}

#[cfg(test)]
mod tests {
  use crate::app_status_or_default;
  use rstest::rstest;
  use services::{
    test_utils::SecretServiceStub, AppStatus, SecretService, APP_STATUS_READY, APP_STATUS_SETUP,
    KEY_APP_STATUS,
  };
  use std::sync::Arc;

  #[rstest]
  #[case(APP_STATUS_SETUP, AppStatus::Setup)]
  #[case(APP_STATUS_READY, AppStatus::Ready)]
  #[case("resource-admin", AppStatus::ResourceAdmin)]
  fn test_app_status_or_default(
    #[case] status: &str,
    #[case] expected: AppStatus,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => status.to_string(),
    });
    assert_eq!(
      expected,
      app_status_or_default(&(Arc::new(secret_service) as Arc<dyn SecretService>))
    );
    Ok(())
  }

  #[rstest]
  fn test_app_status_or_default_not_found() -> anyhow::Result<()> {
    let secret_service: &Arc<dyn SecretService> =
      &(Arc::new(SecretServiceStub::new()) as Arc<dyn SecretService>);
    assert_eq!(AppStatus::default(), app_status_or_default(secret_service));
    assert_eq!(AppStatus::Setup, app_status_or_default(secret_service));
    Ok(())
  }
}
