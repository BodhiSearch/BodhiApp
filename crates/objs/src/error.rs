use crate::AppError;
use axum::{extract::rejection::JsonRejection, http::StatusCode};
use derive_builder::UninitializedFieldError;
use validator::{ValidationError, ValidationErrors};

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}

// https://help.openai.com/en/articles/6897213-openai-library-error-types-guidance
#[derive(Debug, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  #[strum(serialize = "validation_error")]
  Validation,
  #[strum(serialize = "invalid_request_error")]
  BadRequest,
  #[strum(serialize = "invalid_app_state")]
  InvalidAppState,
  #[strum(serialize = "internal_server_error")]
  InternalServer,
  #[strum(serialize = "authentication_error")]
  Authentication,
  #[strum(serialize = "not_found_error")]
  NotFound,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ObjValidationError {
  #[error("validation_errors")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  ValidationErrors(#[from] ValidationErrors),

  #[error("file_pattern_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 400)]
  FilePatternMismatch(String),
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("bad_request_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest, status = 400)]
pub struct BadRequestError {
  reason: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("internal_server_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct InternalServerError {
  reason: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoError {
  #[from]
  source: std::io::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoWithPathError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_dir_create_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoDirCreateError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_read_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileReadError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_write_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileWriteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("io_file_delete_failed")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct IoFileDeleteError {
  #[source]
  source: std::io::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonError {
  #[from]
  source: serde_json::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_json_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeJsonWithPathError {
  #[source]
  source: serde_json::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlError {
  #[from]
  source: serde_yaml::Error,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("serde_yaml_with_path_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct SerdeYamlWithPathError {
  #[source]
  source: serde_yaml::Error,
  path: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("internal_network_error")]
#[error_meta(trait_to_impl = AppError,
  error_type = ErrorType::InternalServer,
  status = 500,
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[error_meta(trait_to_impl = AppError)]
#[non_exhaustive]
pub enum BuilderError {
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  UninitializedField(&'static str),
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
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
#[error("json_rejection_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest, status = StatusCode::BAD_REQUEST.as_u16())]
pub struct JsonRejectionError {
  #[from]
  source: JsonRejection,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    test_utils::{assert_error_message, parse, setup_l10n},
    ApiError, FluentLocalizationService, Repo,
  };
  use axum::{body::Body, response::Response, routing::get, Json, Router};
  use axum_extra::extract::WithRejection;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Error as StdIoError, ErrorKind},
    sync::Arc,
  };
  use tower::ServiceExt;
  use validator::ValidationErrorsKind;

  #[rstest]
  #[case(&Repo::try_from("invalid-repo").unwrap_err(), "validation_error: value: does not match the huggingface repo pattern 'username/repo'")]
  #[case(&IoWithPathError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: path: test.txt, file not found")]
  #[case(&IoDirCreateError::new(StdIoError::new(ErrorKind::AlreadyExists, "already exists"), "model-home".to_string()), "io_error: failed to create directory $BODHI_HOME/model-home, error: already exists")]
  #[case(&IoFileWriteError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to update file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&IoFileReadError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to read file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&IoFileDeleteError::new(StdIoError::new(ErrorKind::NotFound, "file not found"), "test.txt".to_string()), "io_error: failed to delete file $BODHI_HOME/test.txt, error: file not found")]
  #[case(&SerdeJsonError::new(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err()), "error serializing/deserializing json: expected value at line 1 column 1")]
  #[case(&SerdeJsonWithPathError::new(serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err(), "test.json".to_string()), "error serializing/deserializing json: path: test.json, expected value at line 1 column 1")]
  #[case(&SerdeYamlError::new(serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err()), "error serializing/deserializing yaml: found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10")]
  #[case(&SerdeYamlWithPathError::new(serde_yaml::from_str::<serde_yaml::Value>("invalid: foo\n\tbar").unwrap_err(), "test.yaml".to_string()), "error serializing/deserializing yaml: path: test.yaml, found a tab character that violates indentation at line 2 column 1, while scanning a plain scalar at line 1 column 10")]
  #[case(&BadRequestError::new("invalid input".to_string()), "invalid request, reason: invalid input")]
  #[case(&InternalServerError::new("unexpected server error".to_string()), "internal_server_error: unexpected server error")]
  #[case(&IoError::new(StdIoError::new(ErrorKind::PermissionDenied, "test io error")), "io_error: test io error")]
  #[case(&ObjValidationError::ValidationErrors(ValidationErrors(HashMap::from([("field", ValidationErrorsKind::Field(vec![validator::ValidationError::new("value").with_message(Cow::Borrowed("validation failed"))]))]))), "validation_error: field: validation failed")]
  #[case(&ObjValidationError::FilePatternMismatch("huggingface/hub/models--invalid-repo/snapshots/model.gguf".to_string()), "file pattern does not match huggingface repo pattern, path: huggingface/hub/models--invalid-repo/snapshots/model.gguf")]
  #[case(&ReqwestError {
    error: "error sending request for url (http://foobar.nohost/)".to_string(),
  }, "error connecting to internal service: error sending request for url (http://foobar.nohost/)")]
  #[case::uninitialized_field(
    &BuilderError::UninitializedField("field_name"),
    "builder_error: uninitialized field: field_name"
  )]
  #[case::validation_error(&BuilderError::ValidationError("validation failed".to_string()), "builder_error: validation error: validation failed")]
  #[case::file_pattern_mismatch(&ObjValidationError::FilePatternMismatch("test.txt".to_string()), "file pattern does not match huggingface repo pattern, path: test.txt")]
  fn test_error_messages_objs(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  #[rstest]
  #[tokio::test]
  async fn test_json_rejection_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) {
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
      response,
      json! {{
        "error": {
          "message": "failed to parse the request body as JSON, error: \u{2068}Expected request with `Content-Type: application/json`\u{2069}",
          "type": "invalid_request_error",
          "code": "json_rejection_error"
        }
      }}
    );
  }
}
