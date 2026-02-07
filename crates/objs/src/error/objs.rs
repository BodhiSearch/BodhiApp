use axum::extract::rejection::JsonRejection;
use derive_builder::UninitializedFieldError;
use validator::{ValidationError, ValidationErrors};

use crate::{AppError, ErrorType};

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum EntityError {
  #[error("{0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error("Application registration information is missing.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct AppRegInfoMissingError;

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ObjValidationError {
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ValidationErrors(#[from] ValidationErrors),

  #[error("Invalid repository format '{0}'. Expected 'username/repo'.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FilePatternMismatch(String),

  #[error("Prefix is required when forwarding all requests.")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "obj_validation_error-forward_all_requires_prefix")]
  ForwardAllRequiresPrefix,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Service unavailable: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::ServiceUnavailable)]
pub struct ServiceUnavailableError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Invalid request: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct BadRequestError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("{reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound)]
pub struct NotFoundError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Internal error: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct InternalServerError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Access denied: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::Unauthorized)]
pub struct UnauthorizedError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Resource conflict: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::Conflict)]
pub struct ConflictError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Cannot process request: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::UnprocessableEntity)]
pub struct UnprocessableEntityError {
  reason: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum IoError {
  #[error("File operation failed: {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Io {
    #[from]
    source: std::io::Error,
  },

  #[error("File operation failed for '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  WithPath {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to create folder '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to read file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileRead {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to write file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileWrite {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to delete file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileDelete {
    #[source]
    source: std::io::Error,
    path: String,
  },
}

impl IoError {
  pub fn with_path(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::WithPath {
      source,
      path: path.into(),
    }
  }

  pub fn dir_create(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::DirCreate {
      source,
      path: path.into(),
    }
  }

  pub fn file_read(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileRead {
      source,
      path: path.into(),
    }
  }

  pub fn file_write(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileWrite {
      source,
      path: path.into(),
    }
  }

  pub fn file_delete(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileDelete {
      source,
      path: path.into(),
    }
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Failed to process JSON data: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct SerdeJsonError {
  #[from]
  source: serde_json::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Failed to process JSON file '{path}': {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct SerdeJsonWithPathError {
  #[source]
  source: serde_json::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Failed to process YAML data: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct SerdeYamlError {
  #[from]
  source: serde_yaml::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Failed to process YAML file '{path}': {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct SerdeYamlWithPathError {
  #[source]
  source: serde_yaml::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Network error: {error}.")]
#[error_meta(trait_to_impl = AppError,
  error_type = ErrorType::InternalServer,
)]
pub struct ReqwestError {
  error: String,
}

impl From<reqwest::Error> for ReqwestError {
  fn from(source: reqwest::Error) -> Self {
    Self {
      error: source.to_string(),
    }
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
#[non_exhaustive]
pub enum BuilderError {
  #[error("Configuration incomplete: missing {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  UninitializedField(&'static str),
  #[error("Configuration invalid: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ValidationError(String),
}

impl From<UninitializedFieldError> for BuilderError {
  fn from(s: UninitializedFieldError) -> Self {
    Self::UninitializedField(s.field_name())
  }
}

impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Invalid JSON in request: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct JsonRejectionError {
  #[from]
  source: JsonRejection,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Concurrent access error: {reason}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct RwLockReadError {
  reason: String,
}

#[cfg(test)]
mod tests {
  use crate::{test_utils::parse, ApiError};
  use axum::{body::Body, http::StatusCode, response::Response, routing::get, Json, Router};
  use axum_extra::extract::WithRejection;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use tower::ServiceExt;

  #[rstest]
  #[tokio::test]
  async fn test_json_rejection_error() {
    #[derive(Debug, Serialize, Deserialize)]
    struct Input {
      source: String,
    }

    async fn with_json_rejection(
      WithRejection(Json(value), _): WithRejection<Json<Input>, ApiError>,
    ) -> Result<Response, ApiError> {
      let input = value.source;
      Ok(
        Response::builder()
          .status(418)
          .body(Body::from(format!("{{\"message\": \"ok - {input}\"}}")))
          .unwrap(),
      )
    }

    let router = Router::new().route("/", get(with_json_rejection));
    let response = router
      .oneshot(
        axum::http::Request::builder()
          .uri("/")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = parse::<Value>(response).await;
    assert_eq!(
      json! {{
        "error": {
          "message": "Invalid JSON in request: Expected request with `Content-Type: application/json`.",
          "type": "invalid_request_error",
          "code": "json_rejection_error",
          "param": {"source": "Expected request with `Content-Type: application/json`"}
        }
      }},
      response
    );
  }
}
